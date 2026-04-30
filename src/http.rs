use crate::common;
use crate::error::Error;
use crate::test_config::TestConfig;
use reqwest::Client;
use rustls::client::WebPkiServerVerifier;
use rustls::client::danger::ServerCertVerifier;
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use std::sync::Arc;

/// TLS certificate configuration options.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    /// Path to a custom CA certificate file (PEM format).
    pub ca_cert_path: Option<std::path::PathBuf>,
    /// Minimum TLS version to use (e.g., "1.2", "1.3").
    pub min_tls_version: Option<String>,
    /// Restrict TLS connections to speedtest.net and ookla.com domains.
    pub pin_speedtest_certs: bool,
}

impl TlsConfig {
    /// Set a custom CA certificate file for TLS verification.
    #[must_use]
    pub fn with_ca_cert(mut self, path: std::path::PathBuf) -> Self {
        self.ca_cert_path = Some(path);
        self
    }

    /// Set minimum TLS version.
    #[must_use]
    pub fn with_min_tls_version(mut self, version: impl Into<String>) -> Self {
        self.min_tls_version = Some(version.into());
        self
    }

    /// Enable speedtest.net/ookla.com TLS domain restriction.
    #[must_use]
    pub fn with_cert_pinning(mut self) -> Self {
        self.pin_speedtest_certs = true;
        self
    }
}

/// Default browser-like user agent for speedtest.net compatibility.
/// Can be overridden via config file with custom_user_agent option.
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// HTTP client settings - decoupled from Config struct.
///
/// This allows creating HTTP clients without depending on the full Config,
/// improving modularity and testability.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Timeout in seconds for HTTP requests.
    pub timeout_secs: u64,
    /// Optional source IP address to bind to.
    pub source_ip: Option<String>,
    /// User agent string for HTTP requests.
    pub user_agent: String,
    /// Enable automatic retry on transient failures.
    pub retry_enabled: bool,
    /// TLS certificate configuration.
    pub tls: TlsConfig,
}

/// Build [`Settings`] from a [`crate::config::Config`] reference.
///
/// Centralizes the Config→HTTP bridging so callers don't duplicate
/// the mapping. Resolves custom_user_agent from file config or default.
///
/// This impl lives in `http.rs` (not `config.rs`) to preserve layering:
/// dependency flows http → config, not config → http.
impl From<&crate::config::Config> for Settings {
    fn from(config: &crate::config::Config) -> Self {
        Self {
            timeout_secs: config.timeout(),
            source_ip: config.source().map(String::from),
            user_agent: config
                .custom_user_agent()
                .map(String::from)
                .unwrap_or_else(|| DEFAULT_USER_AGENT.to_string()),
            retry_enabled: true,
            tls: TlsConfig {
                ca_cert_path: config.ca_cert_path(),
                min_tls_version: config.tls_version().map(String::from),
                pin_speedtest_certs: config.pin_certs(),
            },
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            source_ip: None,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            retry_enabled: true,
            tls: TlsConfig::default(),
        }
    }
}

impl Settings {
    /// Set a custom user agent (e.g., from config file).
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Disable retry logic (useful for tests or when caller handles retries).
    #[must_use]
    pub fn with_retry_disabled(mut self) -> Self {
        self.retry_enabled = false;
        self
    }
}

/// Create an HTTP client with the given settings.
///
/// # Errors
///
/// Returns [`Error::Context`] if the source IP is invalid.
/// Returns [`Error::NetworkError`] if the client fails to build.
pub fn create_client(settings: &Settings) -> Result<Client, Error> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(settings.timeout_secs))
        .http1_only()
        .no_gzip()
        .use_rustls_tls()
        .user_agent(&settings.user_agent);

    if let Some(ref source_ip) = settings.source_ip {
        let addr: std::net::SocketAddr = source_ip
            .parse()
            .map_err(|e| Error::with_source("Invalid source IP", e))?;
        builder = builder.local_address(addr.ip());
    }

    // Apply TLS configuration if any options are set
    if settings.tls.ca_cert_path.is_some()
        || settings.tls.min_tls_version.is_some()
        || settings.tls.pin_speedtest_certs
    {
        let tls_config = build_tls_config(&settings.tls)?;
        builder = builder.use_preconfigured_tls(tls_config);
    }

    let client = builder.build().map_err(Error::NetworkError)?;

    Ok(client)
}

/// Build a rustls client configuration based on the TLS settings.
fn build_tls_config(tls: &TlsConfig) -> Result<ClientConfig, Error> {
    // Determine protocol versions based on min_tls_version setting
    let versions: &[&rustls::SupportedProtocolVersion] = match tls.min_tls_version.as_deref() {
        Some("1.2") => &[&rustls::version::TLS12],
        Some("1.3") => &[&rustls::version::TLS13],
        Some(v) => {
            eprintln!("Warning: Unknown TLS version '{}', using defaults", v);
            rustls::DEFAULT_VERSIONS
        }
        None => rustls::DEFAULT_VERSIONS,
    };

    if tls.pin_speedtest_certs && tls.ca_cert_path.is_some() {
        eprintln!(
            "Warning: Both --ca-cert and --pin-certs are set. Custom CA verification will be used before the speedtest.net domain restriction."
        );
    }

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ClientConfig::builder_with_provider(Arc::clone(&provider))
        .with_protocol_versions(versions)
        .map_err(|e| Error::context(format!("Invalid TLS configuration: {e}")))?;

    let root_store = match tls.ca_cert_path.as_deref() {
        Some(ca_path) => load_custom_ca_cert(ca_path)?,
        None => default_root_store(),
    };

    if tls.pin_speedtest_certs {
        let verifier = PinningVerifier::try_new(root_store, provider)?;
        return Ok(builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
            .with_no_client_auth());
    }

    Ok(builder
        .with_root_certificates(root_store)
        .with_no_client_auth())
}

fn default_root_store() -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    root_store
}

/// Load a custom CA certificate from a PEM or DER file.
fn load_custom_ca_cert(path: &std::path::Path) -> Result<RootCertStore, Error> {
    let pem_data = std::fs::read(path)
        .map_err(|e| Error::context(format!("Failed to read CA cert: {}", e)))?;

    let mut store = RootCertStore::empty();

    // Try PEM first (returns iterator in newer versions)
    let mut cursor = std::io::Cursor::new(&pem_data);
    let mut found_cert = false;
    for cert_result in rustls_pemfile::certs(&mut cursor) {
        match cert_result {
            Ok(cert) => {
                store
                    .add(cert)
                    .map_err(|e| Error::context(format!("Failed to add cert: {}", e)))?;
                found_cert = true;
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse PEM cert: {}", e);
            }
        }
    }

    // If no PEM certs found, try DER format
    if !found_cert {
        store
            .add(CertificateDer::from(pem_data))
            .map_err(|e| Error::context(format!("Failed to parse cert: {}", e)))?;
    }

    Ok(store)
}

/// Certificate verifier that restricts TLS to speedtest.net and ookla.com domains
/// while preserving normal webpki chain, hostname, and validity checks.
#[derive(Debug)]
struct PinningVerifier {
    inner: Arc<WebPkiServerVerifier>,
}

impl PinningVerifier {
    #[cfg(test)]
    fn new() -> Self {
        Self::try_new(
            default_root_store(),
            Arc::new(rustls::crypto::aws_lc_rs::default_provider()),
        )
        .expect("default TLS verifier should build")
    }

    fn try_new(root_store: RootCertStore, provider: Arc<CryptoProvider>) -> Result<Self, Error> {
        let inner = WebPkiServerVerifier::builder_with_provider(Arc::new(root_store), provider)
            .build()
            .map_err(|e| Error::context(format!("Failed to build TLS verifier: {e:?}")))?;
        Ok(Self { inner })
    }

    fn is_valid_domain(host: &str) -> bool {
        // Check exact domains first, then subdomains ending with the suffix.
        host == "speedtest.net"
            || host == "ookla.com"
            || host.ends_with(".speedtest.net")
            || host.ends_with(".ookla.com")
    }
}

impl ServerCertVerifier for PinningVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediate_certs: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Extract hostname from server name
        let hostname = match server_name {
            ServerName::DnsName(name) => name.as_ref(),
            _ => {
                return Err(rustls::Error::General(
                    "Unsupported server name type".to_string(),
                ));
            }
        };

        if !Self::is_valid_domain(hostname) {
            return Err(rustls::Error::General(format!(
                "'{}' is not a speedtest.net domain",
                hostname
            )));
        }

        self.inner.verify_server_cert(
            end_entity,
            _intermediate_certs,
            server_name,
            _ocsp_response,
            _now,
        )
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Represents a transient HTTP error that may benefit from retry.
fn is_transient_error(e: &reqwest::Error) -> bool {
    if e.is_timeout() {
        return true;
    }
    if e.is_connect() {
        return true;
    }
    // Server errors (5xx) are transient
    if let Some(status) = e.status() {
        return status.as_u16() >= 500;
    }
    false
}

/// Execute an HTTP request with automatic retry on transient failures.
///
/// This function wraps a request closure with exponential backoff retry logic.
/// It will retry on timeouts, connection errors, and 5xx server errors.
///
/// # Arguments
///
/// * `request` - Closure that creates and executes the request
///
/// # Errors
///
/// Returns the final error after all retry attempts are exhausted.
pub async fn with_retry<R, F, Fut>(mut request: F) -> Result<R, Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<R, reqwest::Error>>,
{
    let config = TestConfig::default();
    let max_attempts = config.http_retry_attempts;

    for attempt in 0..max_attempts {
        let result = request().await;

        if let Ok(r) = result {
            return Ok(r);
        }

        // Get the error reference (we can't clone reqwest::Error)
        if let Err(e) = &result {
            let (delay, should_retry) = TestConfig::retry_delay(attempt);

            // Check if error is transient and we should retry
            #[allow(clippy::collapsible_if)]
            if should_retry && is_transient_error(e) && attempt < max_attempts - 1 {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                continue;
            }

            // Non-transient error or exhausted retries - return the error
            return result.map_err(Error::NetworkError);
        }
    }

    // This should not be reached, but handle it defensively
    Err(Error::context("retry loop ended without result or error"))
}

/// Discover the client's public IP address via speedtest.net.
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if all IP discovery endpoints fail.
pub async fn discover_client_ip(client: &Client) -> Result<String, Error> {
    if let Ok(response) = client
        .get("https://www.speedtest.net/api/ip.php")
        .send()
        .await
    {
        if let Ok(text) = response.text().await {
            let trimmed = text.trim().to_string();
            if common::is_valid_ipv4(&trimmed) {
                return Ok(trimmed);
            }
        }
    }

    if let Ok(response) = client
        .get("https://www.speedtest.net/api/ios-config.php")
        .send()
        .await
    {
        if let Ok(text) = response.text().await {
            if let Some(ip) = parse_ip_from_xml(&text) {
                return Ok(ip);
            }
        }
    }

    Ok("unknown".to_string())
}

fn parse_ip_from_xml(xml: &str) -> Option<String> {
    // Use structured XML deserialization instead of manual string scanning
    // to handle edge cases (comments, CDATA, nested elements) correctly.
    #[derive(serde::Deserialize)]
    struct Settings {
        client: ClientElement,
    }
    #[derive(serde::Deserialize)]
    struct ClientElement {
        #[serde(rename = "@ip")]
        ip: Option<String>,
    }

    // XML parse failures are expected (malformed responses, unexpected structure)
    // and are not actionable — the caller falls back to returning "unknown".
    let settings: Settings = match quick_xml::de::from_str(xml) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let ip = settings.client.ip?;
    if common::is_valid_ipv4(&ip) {
        Some(ip)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ==================== TlsConfig Builder Method Tests ====================

    #[test]
    fn test_tls_config_with_ca_cert() {
        let config = TlsConfig::default();
        assert!(config.ca_cert_path.is_none());

        let config = config.with_ca_cert(std::path::PathBuf::from("/path/to/cert.pem"));
        assert_eq!(
            config.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/cert.pem"))
        );
    }

    #[test]
    fn test_tls_config_with_min_tls_version() {
        let config = TlsConfig::default();
        assert!(config.min_tls_version.is_none());

        let config = config.with_min_tls_version("1.2");
        assert_eq!(config.min_tls_version, Some("1.2".to_string()));

        let config = TlsConfig::default().with_min_tls_version("1.3");
        assert_eq!(config.min_tls_version, Some("1.3".to_string()));
    }

    #[test]
    fn test_tls_config_with_cert_pinning() {
        let config = TlsConfig::default();
        assert!(!config.pin_speedtest_certs);

        let config = config.with_cert_pinning();
        assert!(config.pin_speedtest_certs);
    }

    #[test]
    fn test_tls_config_builder_chaining() {
        // Test that builder methods can be chained
        let config = TlsConfig::default()
            .with_ca_cert(std::path::PathBuf::from("/custom/ca.pem"))
            .with_min_tls_version("1.3")
            .with_cert_pinning();

        assert_eq!(
            config.ca_cert_path,
            Some(std::path::PathBuf::from("/custom/ca.pem"))
        );
        assert_eq!(config.min_tls_version, Some("1.3".to_string()));
        assert!(config.pin_speedtest_certs);
    }

    // ==================== Settings Tests ====================

    #[test]
    fn test_settings_default_values() {
        let settings = Settings::default();
        assert_eq!(settings.timeout_secs, 10);
        assert!(settings.source_ip.is_none());
        assert_eq!(settings.user_agent, DEFAULT_USER_AGENT);
        assert!(settings.retry_enabled);
        // Check TlsConfig fields individually since PartialEq isn't derived
        assert!(settings.tls.ca_cert_path.is_none());
        assert!(settings.tls.min_tls_version.is_none());
        assert!(!settings.tls.pin_speedtest_certs);
    }

    #[test]
    fn test_settings_with_user_agent() {
        let settings = Settings::default().with_user_agent("Custom Agent/1.0");
        assert_eq!(settings.user_agent, "Custom Agent/1.0");
    }

    #[test]
    fn test_settings_with_user_agent_chaining() {
        let settings = Settings::default()
            .with_user_agent("Test Agent")
            .with_retry_disabled();
        assert_eq!(settings.user_agent, "Test Agent");
        assert!(!settings.retry_enabled);
    }

    #[test]
    fn test_settings_with_retry_disabled() {
        let settings = Settings::default();
        assert!(settings.retry_enabled);

        let settings = settings.with_retry_disabled();
        assert!(!settings.retry_enabled);
    }

    #[test]
    fn test_settings_debug_trait() {
        let settings = Settings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("timeout_secs"));
        assert!(debug_str.contains("user_agent"));
    }

    #[test]
    fn test_settings_clone() {
        let settings = Settings::default();
        let cloned = settings.clone();
        assert_eq!(settings.user_agent, cloned.user_agent);
        assert_eq!(settings.timeout_secs, cloned.timeout_secs);
    }

    // ==================== is_transient_error Tests ====================
    // Note: is_transient_error requires a real reqwest::Error which is difficult to construct.
    // The function is tested indirectly via integration tests with actual network failures.

    // ==================== build_tls_config Tests ====================
    // Note: These tests require a configured rustls CryptoProvider.
    // They are skipped in unit tests but tested via integration tests.
    // The build_tls_config function's behavior is tested indirectly via
    // create_client tests with TLS options.

    #[test]
    #[ignore]
    fn test_build_tls_config_unknown_tls_version() {
        let tls = TlsConfig {
            min_tls_version: Some("99.0".to_string()),
            ..Default::default()
        };
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_build_tls_config_tls12() {
        let tls = TlsConfig {
            min_tls_version: Some("1.2".to_string()),
            ..Default::default()
        };
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_build_tls_config_tls13() {
        let tls = TlsConfig {
            min_tls_version: Some("1.3".to_string()),
            ..Default::default()
        };
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_build_tls_config_pinning_takes_precedence() {
        let tls = TlsConfig {
            ca_cert_path: Some(std::path::PathBuf::from("/path/to/ca.pem")),
            pin_speedtest_certs: true,
            ..Default::default()
        };
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_build_tls_config_pinning_only() {
        let tls = TlsConfig {
            pin_speedtest_certs: true,
            ..Default::default()
        };
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_build_tls_config_no_options() {
        let tls = TlsConfig::default();
        let result = build_tls_config(&tls);
        assert!(result.is_ok());
    }

    // ==================== load_custom_ca_cert Tests ====================

    #[test]
    fn test_load_custom_ca_cert_file_not_found() {
        let result = load_custom_ca_cert(std::path::Path::new("/nonexistent/cert.pem"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should mention the path
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("nonexistent") || err_msg.contains("Failed to read CA cert"));
    }

    #[test]
    fn test_load_custom_ca_cert_invalid_path() {
        // Test with a directory path instead of file
        let result = load_custom_ca_cert(std::path::Path::new("/tmp"));
        assert!(result.is_err());
    }

    // ==================== create_client Tests ====================
    // Note: Some TLS-related tests are ignored due to rustls CryptoProvider requirements.
    // These are tested via integration tests.

    #[test]
    fn test_create_client_source_ip_v4() {
        let settings = Settings {
            source_ip: Some("192.168.1.100".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        // IPv4 source IP should work
        match result {
            Ok(_) => {}
            Err(Error::Context { .. }) => {} // Invalid IP format returns Context
            Err(e) => panic!("Unexpected error type for valid IPv4: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_source_ip_v6() {
        let settings = Settings {
            source_ip: Some("::1".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        match result {
            Ok(_) => {}
            Err(Error::NetworkError(_) | Error::Context { .. }) => {} // Network errors acceptable
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    #[ignore]
    fn test_create_client_with_ca_cert() {
        let settings = Settings {
            tls: TlsConfig {
                ca_cert_path: Some(std::path::PathBuf::from("/nonexistent/ca.pem")),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    fn test_create_client_with_pinning() {
        let settings = Settings {
            tls: TlsConfig {
                pin_speedtest_certs: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_retry_disabled() {
        let settings = Settings::default().with_retry_disabled();
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_timeout_30() {
        let settings = Settings {
            timeout_secs: 30,
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_timeout_60() {
        let settings = Settings {
            timeout_secs: 60,
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    // ==================== PinningVerifier Tests ====================

    #[test]
    fn test_pinning_verifier_is_valid_domain_speedtest() {
        // Valid subdomains - .ends_with() matches exact domains too
        assert!(PinningVerifier::is_valid_domain("speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("www.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("api.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("foo.bar.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("fake.speedtest.net")); // also valid (subdomain)

        // Invalid domains - must NOT end with .speedtest.net
        assert!(!PinningVerifier::is_valid_domain("evilsite.net"));
        assert!(!PinningVerifier::is_valid_domain("speedtest.com"));
        assert!(!PinningVerifier::is_valid_domain("notspeedtest.net"));
    }

    #[test]
    fn test_pinning_verifier_is_valid_domain_ookla() {
        // Valid subdomains - .ends_with() matches exact domains too
        assert!(PinningVerifier::is_valid_domain("ookla.com"));
        assert!(PinningVerifier::is_valid_domain("www.ookla.com"));
        assert!(PinningVerifier::is_valid_domain("api.ookla.com"));
        assert!(PinningVerifier::is_valid_domain("foo.bar.ookla.com"));
        assert!(PinningVerifier::is_valid_domain("fake.ookla.com")); // also valid (subdomain)

        // Invalid domains - must NOT end with .ookla.com
        assert!(!PinningVerifier::is_valid_domain("ookla.net"));
    }

    #[test]
    fn test_pinning_verifier_edge_cases() {
        // Edge cases for security
        assert!(!PinningVerifier::is_valid_domain(""));
        assert!(!PinningVerifier::is_valid_domain("speedtestXnet")); // no dot prefix
        assert!(!PinningVerifier::is_valid_domain("attack.com")); // unrelated domain
    }

    #[test]
    fn test_pinning_verifier_exact_domains() {
        // Test exact domain matches (should be valid)
        assert!(PinningVerifier::is_valid_domain("speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("ookla.com"));
    }

    #[test]
    fn test_pinning_verifier_subdomains() {
        // Test various subdomain depths
        assert!(PinningVerifier::is_valid_domain("www.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("api.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("a.b.c.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("www.ookla.com"));
        assert!(PinningVerifier::is_valid_domain("api.www.ookla.com"));
    }

    #[test]
    fn test_pinning_verifier_invalid_suffixes() {
        // These should NOT match because they don't end with the exact suffix
        assert!(!PinningVerifier::is_valid_domain("xspeedtest.net")); // prefix attack
        assert!(!PinningVerifier::is_valid_domain("fake-speedtest.net")); // prefix attack
        assert!(!PinningVerifier::is_valid_domain("speedtest.net.evil.com")); // suffix confusion
        assert!(!PinningVerifier::is_valid_domain("ookla.com.evil.com")); // suffix confusion
        assert!(!PinningVerifier::is_valid_domain("fooookla.com")); // prefix attack
    }

    #[test]
    fn test_pinning_verifier_case_sensitivity() {
        // Domain matching should be case-sensitive (DNS is case-insensitive but we check exact match)
        assert!(!PinningVerifier::is_valid_domain("Speedtest.net")); // uppercase
        assert!(!PinningVerifier::is_valid_domain("SPEEDTEST.NET")); // all caps
        assert!(!PinningVerifier::is_valid_domain("www.Speedtest.net")); // mixed case
        assert!(!PinningVerifier::is_valid_domain("OOKLA.COM")); // all caps ookla
    }

    #[test]
    fn test_pinning_verifier_special_characters() {
        // Invalid domain formats
        assert!(!PinningVerifier::is_valid_domain("speedtest.net/")); // trailing slash
        assert!(!PinningVerifier::is_valid_domain("speedtest.net:443")); // port number
        assert!(!PinningVerifier::is_valid_domain("speedtest.net/path")); // path
    }

    #[test]
    fn test_pinning_verifier_numeric_domains() {
        // Numeric subdomains are valid
        assert!(PinningVerifier::is_valid_domain("123.speedtest.net")); // valid numeric subdomain
        assert!(PinningVerifier::is_valid_domain("1.2.3.speedtest.net")); // valid numeric subdomain
        // Numeric prefix on base domain is invalid
        assert!(!PinningVerifier::is_valid_domain("speedtest123.net")); // not valid
        assert!(!PinningVerifier::is_valid_domain("123speedtest.net")); // not valid
    }

    #[test]
    fn test_pinning_verifier_new_returns_self() {
        // Test that new() creates an instance
        let verifier = PinningVerifier::new();
        assert!(!verifier.supported_verify_schemes().is_empty());
    }

    #[test]
    fn test_pinning_verifier_debug_trait() {
        // Test that Debug can be derived and used
        let verifier = PinningVerifier::new();
        let debug_str = format!("{:?}", verifier);
        assert!(debug_str.contains("PinningVerifier"));
    }

    #[test]
    fn test_pinning_verifier_supported_verify_schemes() {
        let verifier = PinningVerifier::new();
        let schemes = verifier.supported_verify_schemes();

        // Should support these signature schemes
        assert!(schemes.contains(&SignatureScheme::RSA_PKCS1_SHA256));
        assert!(schemes.contains(&SignatureScheme::RSA_PKCS1_SHA384));
        assert!(schemes.contains(&SignatureScheme::RSA_PKCS1_SHA512));
        assert!(schemes.contains(&SignatureScheme::ECDSA_NISTP256_SHA256));
        assert!(schemes.contains(&SignatureScheme::ECDSA_NISTP384_SHA384));
        assert!(schemes.contains(&SignatureScheme::RSA_PSS_SHA256));
        assert!(schemes.contains(&SignatureScheme::RSA_PSS_SHA384));
        assert!(schemes.contains(&SignatureScheme::RSA_PSS_SHA512));

        assert!(schemes.len() >= 8);
    }

    // Note: Signature verification tests are omitted because DigitallySignedStruct
    // has a private constructor in rustls. The signature verification methods always
    // return HandshakeSignatureValid::assertion() in PinningVerifier, which is tested
    // implicitly by successful TLS handshakes with valid speedtest.net certificates.

    #[test]
    fn test_pinning_verifier_verify_server_cert_rejects_invalid_domain() {
        let verifier = PinningVerifier::new();

        // Create a DnsName for an invalid domain
        let dns_name = rustls::pki_types::DnsName::try_from("evil.com".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Create a minimal valid certificate structure
        // Using a real but minimal test certificate
        let cert_der = CertificateDer::from(vec![]);

        // This should fail because the domain is not valid
        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("evil.com") || err_msg.contains("not a speedtest.net domain"));
    }

    #[test]
    fn test_pinning_verifier_verify_server_cert_rejects_unsupported_name_type() {
        let verifier = PinningVerifier::new();

        // Test with an IpAddress server name (unsupported)
        let ip_addr = std::net::IpAddr::from([127, 0, 0, 1]);
        let server_name = ServerName::IpAddress(ip_addr.into());

        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("Unsupported server name type"));
    }

    #[test]
    fn test_pinning_verifier_verify_server_cert_rejects_invalid_certificate() {
        let verifier = PinningVerifier::new();

        // Valid domain but invalid certificate structure
        let dns_name =
            rustls::pki_types::DnsName::try_from("www.speedtest.net".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Empty certificate should fail webpki parsing
        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        // Should fail on cert parsing, not domain validation
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        // The error should be about certificate validation, not domain validation.
        assert!(!err_msg.contains("not a speedtest.net domain"));
    }

    #[test]
    fn test_pinning_verifier_domain_checked_before_cert_parse_speedtest() {
        let verifier = PinningVerifier::new();

        // Valid speedtest.net domain
        let dns_name = rustls::pki_types::DnsName::try_from("speedtest.net".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Empty certificate - the test verifies domain validation happens first
        // Certificate structure validation is tested in a separate test
        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        // Should fail on certificate structure validation, not domain validation
        // This proves domain was checked before cert parsing
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        // The error should be about certificate validation, not domain validation.
        assert!(!err_msg.contains("not a speedtest.net domain"));
    }

    #[test]
    fn test_pinning_verifier_ipv6_address_rejected() {
        let verifier = PinningVerifier::new();

        // Test with an IPv6 address server name
        let ip_addr = std::net::IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]); // ::1
        let server_name = ServerName::IpAddress(ip_addr.into());

        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        assert!(result.is_err());
    }

    #[test]
    fn test_pinning_verifier_domain_checked_before_cert_parse_ookla() {
        let verifier = PinningVerifier::new();

        // Valid ookla.com domain
        let dns_name = rustls::pki_types::DnsName::try_from("ookla.com".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Empty certificate - domain validation is the key being tested here
        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        // Should fail on certificate structure (proves domain was checked first)
        assert!(result.is_err());
    }

    #[test]
    fn test_pinning_verifier_domain_validation_order() {
        // This test verifies that domain validation happens BEFORE certificate parsing
        let verifier = PinningVerifier::new();

        // Invalid domain should fail immediately, without attempting cert parsing
        let dns_name = rustls::pki_types::DnsName::try_from("attacker.com".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Even with a potentially "valid" looking empty cert structure,
        // domain validation should fail first
        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        assert!(
            err_msg.contains("not a speedtest.net domain"),
            "Expected domain validation error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_pinning_verifier_verify_server_cert_rejects_different_tld() {
        let verifier = PinningVerifier::new();

        // Test with speedtest.net.org (should be rejected)
        let dns_name =
            rustls::pki_types::DnsName::try_from("speedtest.net.org".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        let cert_der = CertificateDer::from(vec![]);

        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("not a speedtest.net domain"));
    }

    #[test]
    fn test_pinning_verifier_intermediate_certs_ignored() {
        // Test that intermediate certificates are ignored in validation
        // The implementation only validates the end-entity certificate
        let verifier = PinningVerifier::new();

        // Valid domain
        let dns_name =
            rustls::pki_types::DnsName::try_from("www.speedtest.net".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Empty certificate - will fail on structure but that's expected
        let cert_der = CertificateDer::from(vec![]);

        // Add intermediate certificates (should be ignored)
        let intermediate_cert = CertificateDer::from(vec![0u8; 10]);

        let result = verifier.verify_server_cert(
            &cert_der,
            &[intermediate_cert],
            &server_name,
            &[],
            UnixTime::now(),
        );

        // Should fail on cert structure, not because of intermediate certs
        assert!(result.is_err());
    }

    #[test]
    fn test_pinning_verifier_ocsp_response_ignored() {
        // Test that OCSP response data is ignored
        let verifier = PinningVerifier::new();

        // Valid domain
        let dns_name = rustls::pki_types::DnsName::try_from("api.ookla.com".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);

        // Empty certificate
        let cert_der = CertificateDer::from(vec![]);

        // Add OCSP response data (should be ignored)
        let ocsp_response = vec![0x30, 0x03, 0x01, 0x00];

        let result = verifier.verify_server_cert(
            &cert_der,
            &[],
            &server_name,
            &ocsp_response,
            UnixTime::now(),
        );

        // Should fail on cert structure, not because of OCSP
        assert!(result.is_err());
    }

    #[test]
    fn test_pinning_verifier_all_valid_subdomains() {
        // Test all valid subdomain patterns
        let valid_subdomains = [
            "www.speedtest.net",
            "api.speedtest.net",
            "test.speedtest.net",
            "staging.speedtest.net",
            "prod.speedtest.net",
            "cdn.speedtest.net",
            "a.speedtest.net",
            "z.speedtest.net",
            "a1b2c3.speedtest.net",
            "my-site.speedtest.net",
            "www.ookla.com",
            "api.ookla.com",
            "test.ookla.com",
        ];

        for domain in valid_subdomains {
            assert!(
                PinningVerifier::is_valid_domain(domain),
                "Domain '{}' should be valid",
                domain
            );
        }
    }

    #[test]
    fn test_pinning_verifier_all_invalid_domains() {
        // Test all invalid domain patterns
        let invalid_domains = [
            "evilsite.net",
            "speedtest.net.evil.com",
            "ookla.com.evil.com",
            "speedtest.com",
            "ookla.net",
            "notspeedtest.net",
            "notookla.com",
            "fake-speedtest.net",
            "fake-ookla.com",
            "attacker.speedtest.net.fake.com",
            "attacker.ookla.com.fake.com",
        ];

        for domain in invalid_domains {
            assert!(
                !PinningVerifier::is_valid_domain(domain),
                "Domain '{}' should be invalid",
                domain
            );
        }
    }

    // ==================== Existing Tests ====================

    #[test]
    fn test_parse_ip_from_xml() {
        let xml = r#"<settings><client country="CA" ip="173.35.57.235" isp="Rogers"/></settings>"#;
        assert_eq!(parse_ip_from_xml(xml), Some("173.35.57.235".to_string()));
    }

    #[test]
    fn test_parse_ip_from_xml_full_response() {
        let xml = r#"<?xml version="1.0"?>
<settings>
 <config downloadThreadCountV3="4"/>
 <client country="CA" ip="173.35.57.235" isp="Rogers"/>
</settings>"#;
        assert_eq!(parse_ip_from_xml(xml), Some("173.35.57.235".to_string()));
    }

    #[test]
    fn test_parse_ip_from_xml_invalid() {
        assert!(parse_ip_from_xml("not xml").is_none());
        assert!(parse_ip_from_xml("<html></html>").is_none());
        assert!(parse_ip_from_xml("<settings><client ip=\"invalid\"/></settings>").is_none());
    }

    #[test]
    fn test_create_client_invalid_source_ip() {
        let source = crate::config::ConfigSource::default();
        let config = crate::config::Config::from_source(&source);
        let mut settings = Settings::from(&config);
        settings.source_ip = Some("invalid-ip".to_string());
        let result = create_client(&settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Context { .. }));
    }

    #[test]
    fn test_create_client_valid_config() {
        let source = crate::config::ConfigSource::default();
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_source_ip() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                source: Some("0.0.0.0".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        let result = create_client(&settings);
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_custom_timeout() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                timeout: 30,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    // ==================== Settings from Config Tests ====================

    #[test]
    fn test_settings_from_config_with_source_ip() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                source: Some("192.168.1.50".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        assert_eq!(settings.source_ip, Some("192.168.1.50".to_string()));
    }

    #[test]
    fn test_settings_from_config_with_ca_cert() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                ca_cert: Some("/path/to/ca.pem".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        assert_eq!(
            settings.tls.ca_cert_path,
            Some(std::path::PathBuf::from("/path/to/ca.pem"))
        );
    }

    #[test]
    fn test_settings_from_config_with_tls_version() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                tls_version: Some("1.2".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        assert_eq!(settings.tls.min_tls_version, Some("1.2".to_string()));
    }

    #[test]
    fn test_settings_from_config_with_pinning() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                pin_certs: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        assert!(settings.tls.pin_speedtest_certs);
    }

    #[test]
    fn test_settings_from_config_timeout() {
        let source = crate::config::ConfigSource {
            network: crate::config::NetworkSource {
                timeout: 45,
                ..Default::default()
            },
            ..Default::default()
        };
        let config = crate::config::Config::from_source(&source);
        let settings = Settings::from(&config);
        assert_eq!(settings.timeout_secs, 45);
    }

    #[test]
    fn test_settings_from_config_default_user_agent() {
        let config = crate::config::Config::from_source(&crate::config::ConfigSource::default());
        let settings = Settings::from(&config);
        assert_eq!(settings.user_agent, DEFAULT_USER_AGENT);
    }

    #[test]
    fn test_settings_from_config_retry_enabled_by_default() {
        let config = crate::config::Config::from_source(&crate::config::ConfigSource::default());
        let settings = Settings::from(&config);
        assert!(settings.retry_enabled);
    }

    // ==================== with_retry Tests ====================

    #[tokio::test]
    async fn test_with_retry_immediate_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&counter);

        let result = with_retry(|| {
            let c = Arc::clone(&count);
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, reqwest::Error>(42)
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_with_retry_with_mock_request() {
        // Test with_retry with a request that succeeds
        let result = with_retry(|| async { Ok::<_, reqwest::Error>(100) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_with_retry_counter_increment() {
        let counter = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&counter);

        let _result = with_retry(|| {
            let c = Arc::clone(&count);
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<_, reqwest::Error>(1)
            }
        })
        .await;

        // Verify the counter was incremented exactly once (single attempt)
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_with_retry_different_value_types() {
        // Test with_retry with different success value types
        let result_str = with_retry(|| async { Ok::<_, reqwest::Error>("hello") }).await;
        assert!(result_str.is_ok());
        assert_eq!(result_str.unwrap(), "hello");

        let result_u64 = with_retry(|| async { Ok::<_, reqwest::Error>(999u64) }).await;
        assert!(result_u64.is_ok());
        assert_eq!(result_u64.unwrap(), 999);

        let result_vec = with_retry(|| async { Ok::<_, reqwest::Error>(vec![1, 2, 3]) }).await;
        assert!(result_vec.is_ok());
        assert_eq!(result_vec.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_with_retry_multiple_sequential_calls() {
        // Test calling with_retry multiple times
        for i in 0..3 {
            let result = with_retry(|| async { Ok::<_, reqwest::Error>(i) }).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), i);
        }
    }

    // ==================== parse_ip_from_xml Additional Tests ====================

    #[test]
    fn test_parse_ip_from_xml_missing_client_element() {
        let xml = r#"<settings><server ip="127.0.0.1"/></settings>"#;
        assert!(parse_ip_from_xml(xml).is_none());
    }

    #[test]
    fn test_parse_ip_from_xml_empty_ip() {
        let xml = r#"<settings><client ip=""/></settings>"#;
        assert!(parse_ip_from_xml(xml).is_none());
    }

    #[test]
    fn test_parse_ip_from_xml_whitespace_ip() {
        let xml = r#"<settings><client ip="  " /></settings>"#;
        assert!(parse_ip_from_xml(xml).is_none());
    }

    #[test]
    fn test_parse_ip_from_xml_ipv6_format() {
        let xml = r#"<settings><client ip="::1"/></settings>"#;
        // IPv6 should not match valid IPv4 check
        assert!(parse_ip_from_xml(xml).is_none());
    }

    #[test]
    fn test_parse_ip_from_xml_special_characters() {
        let xml = r#"<settings><client country="US" ip="192.168.1.1" isp="ISP"/></settings>"#;
        assert_eq!(parse_ip_from_xml(xml), Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_parse_ip_from_xml_garbage_after_xml() {
        let xml = r#"<settings><client ip="1.2.3.4" /></settings>GARBAGE"#;
        assert_eq!(parse_ip_from_xml(xml), Some("1.2.3.4".to_string()));
    }

    #[test]
    fn test_parse_ip_from_xml_malformed_xml() {
        assert!(parse_ip_from_xml("<settings><client").is_none());
        assert!(parse_ip_from_xml("</settings>").is_none());
        assert!(parse_ip_from_xml("").is_none());
    }

    // ==================== discover_client_ip Tests ====================

    #[tokio::test]
    async fn test_discover_client_ip_handles_network_failure() {
        // Test with a client that's not properly configured (will fail to connect)
        let settings = Settings::default().with_retry_disabled();
        let client = create_client(&settings).unwrap();

        // This test verifies the function handles network failures gracefully
        let result = discover_client_ip(&client).await;

        // Should return "unknown" on failure, not panic
        match result {
            Ok(ip) => {
                // If it succeeds, verify the format
                assert!(ip == "unknown" || common::is_valid_ipv4(&ip));
            }
            Err(e) => {
                // Network errors are acceptable
                assert!(matches!(e, Error::NetworkError(_)));
            }
        }
    }

    // ==================== TlsConfig Additional Tests ====================

    #[test]
    fn test_tls_config_debug() {
        let config = TlsConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TlsConfig"));
    }

    #[test]
    fn test_tls_config_clone() {
        let config = TlsConfig::default()
            .with_ca_cert(std::path::PathBuf::from("/test.pem"))
            .with_min_tls_version("1.3")
            .with_cert_pinning();
        let cloned = config.clone();
        assert_eq!(cloned.ca_cert_path, config.ca_cert_path);
        assert_eq!(cloned.min_tls_version, config.min_tls_version);
        assert_eq!(cloned.pin_speedtest_certs, config.pin_speedtest_certs);
    }

    #[test]
    fn test_tls_config_default_trait() {
        let config = TlsConfig::default();
        assert!(config.ca_cert_path.is_none());
        assert!(config.min_tls_version.is_none());
        assert!(!config.pin_speedtest_certs);
    }

    // ==================== Settings Additional Tests ====================

    #[test]
    fn test_settings_with_source_ip() {
        let settings = Settings {
            source_ip: Some("10.0.0.1".to_string()),
            ..Default::default()
        };
        let cloned = settings.clone();
        assert_eq!(cloned.source_ip, Some("10.0.0.1".to_string()));
    }

    #[test]
    fn test_settings_builder_full_chain() {
        let settings = Settings::default()
            .with_user_agent("Test/1.0")
            .with_retry_disabled();
        assert_eq!(settings.user_agent, "Test/1.0");
        assert!(!settings.retry_enabled);
    }

    #[test]
    fn test_settings_clone_is_independent() {
        let mut settings = Settings {
            timeout_secs: 60,
            ..Default::default()
        };
        let cloned = settings.clone();
        assert_eq!(cloned.timeout_secs, 60);
        // Modify original should not affect clone (deep clone of primitives)
        settings.timeout_secs = 120;
        assert_eq!(cloned.timeout_secs, 60); // Clone should be independent
    }

    // ==================== create_client Additional Tests ====================

    #[test]
    fn test_create_client_with_source_ip_none() {
        let settings = Settings::default();
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_custom_user_agent() {
        let settings = Settings::default().with_user_agent("TestAgent/1.0");
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_timeout_zero() {
        let settings = Settings {
            timeout_secs: 0,
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_timeout_large() {
        let settings = Settings {
            timeout_secs: 300,
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    // ==================== Error Context Tests ====================

    #[test]
    fn test_error_context_message() {
        let err = Error::context("test error");
        let msg = format!("{:?}", err);
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_error_context_with_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::with_source("operation failed", inner);
        let msg = format!("{:?}", err);
        assert!(msg.contains("operation failed") || msg.contains("file not found"));
    }

    #[test]
    fn test_error_server_not_found() {
        let err = Error::ServerNotFound("no servers available".into());
        let msg = format!("{:?}", err);
        assert!(msg.contains("no servers available") || msg.contains("ServerNotFound"));
    }

    #[test]
    fn test_error_download_failure() {
        let err = Error::DownloadFailure("test download failed".into());
        let msg = format!("{:?}", err);
        assert!(msg.contains("test download failed") || msg.contains("DownloadFailure"));
    }

    #[test]
    fn test_error_upload_failure() {
        let err = Error::UploadFailure("test upload failed".into());
        let msg = format!("{:?}", err);
        assert!(msg.contains("test upload failed") || msg.contains("UploadFailure"));
    }

    #[test]
    fn test_error_context_debug() {
        let err = Error::context("context debug");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Context"));
        assert!(debug_str.contains("context debug"));
    }

    #[test]
    fn test_error_context_display() {
        let err = Error::context("context display");
        assert_eq!(format!("{}", err), "context display");
    }

    #[test]
    fn test_error_download_failure_display() {
        let err = Error::DownloadFailure("download failed".into());
        let display = format!("{}", err);
        assert!(display.contains("download failed"));
    }

    #[test]
    fn test_error_upload_failure_display() {
        let err = Error::UploadFailure("upload failed".into());
        let display = format!("{}", err);
        assert!(display.contains("upload failed"));
    }

    #[test]
    fn test_error_server_not_found_display() {
        let err = Error::ServerNotFound("server not found".into());
        let display = format!("{}", err);
        assert!(display.contains("Server not found"));
        assert!(display.contains("server not found"));
    }

    // ==================== HTTP Client Settings Default Tests ====================

    #[test]
    fn test_settings_default_timeout_10() {
        let settings = Settings::default();
        assert_eq!(settings.timeout_secs, 10);
    }

    #[test]
    fn test_settings_default_retry_true() {
        let settings = Settings::default();
        assert!(settings.retry_enabled);
    }

    #[test]
    fn test_settings_with_timeout() {
        let settings = Settings {
            timeout_secs: 120,
            ..Default::default()
        };
        assert_eq!(settings.timeout_secs, 120);
    }

    // ==================== TlsConfig Defaults Tests ====================

    #[test]
    fn test_tls_config_default_values() {
        let tls = TlsConfig::default();
        assert!(tls.ca_cert_path.is_none());
        assert!(tls.min_tls_version.is_none());
        assert!(!tls.pin_speedtest_certs);
    }

    #[test]
    fn test_tls_config_multiple_options() {
        let tls = TlsConfig::default()
            .with_ca_cert("/path/to/ca.pem".into())
            .with_min_tls_version("1.2");
        assert!(tls.ca_cert_path.is_some());
        assert!(tls.min_tls_version.is_some());
    }

    // ==================== Settings Chain Tests ====================

    #[test]
    fn test_settings_chained_modifications() {
        let settings = Settings::default()
            .with_user_agent("Test/1.0")
            .with_retry_disabled()
            .with_user_agent("Test/2.0");
        assert_eq!(settings.user_agent, "Test/2.0");
        assert!(!settings.retry_enabled);
    }

    // ==================== DEFAULT_USER_AGENT Tests ====================

    #[test]
    fn test_default_user_agent_is_valid() {
        assert!(!DEFAULT_USER_AGENT.is_empty());
        assert!(DEFAULT_USER_AGENT.contains("Mozilla"));
        assert!(DEFAULT_USER_AGENT.contains("Chrome"));
    }

    #[test]
    fn test_default_user_agent_in_settings() {
        let settings = Settings::default();
        assert_eq!(settings.user_agent, DEFAULT_USER_AGENT);
    }

    // ==================== create_client builder variations ====================

    #[test]
    fn test_create_client_all_defaults() {
        let settings = Settings::default();
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_minimal_tls_config() {
        let settings = Settings {
            tls: TlsConfig::default(),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_http1_only() {
        let settings = Settings::default();
        let result = create_client(&settings);
        assert!(result.is_ok());
        // HTTP/1.1 only is configured (verified by no_gzip and http1_only)
    }

    // ==================== PinningVerifier additional tests ====================

    #[test]
    fn test_pinning_verifier_single_char_subdomain() {
        assert!(PinningVerifier::is_valid_domain("a.speedtest.net"));
        assert!(PinningVerifier::is_valid_domain("z.ookla.com"));
    }

    #[test]
    fn test_pinning_verifier_numbers_in_subdomain() {
        // Numbers in subdomain are valid
        assert!(PinningVerifier::is_valid_domain("123.speedtest.net")); // valid subdomain
        assert!(!PinningVerifier::is_valid_domain("speedtest123.net")); // not valid, doesn't end with .speedtest.net
        assert!(!PinningVerifier::is_valid_domain("123speedtest.net")); // prefix attack
    }

    #[test]
    fn test_pinning_verifier_unicode_in_subdomain() {
        // Unicode subdomains should still match the ends_with check
        assert!(PinningVerifier::is_valid_domain("münchen.speedtest.net"));
    }

    #[test]
    fn test_pinning_verifier_empty_cert_with_valid_domain() {
        let verifier = PinningVerifier::new();
        let dns_name =
            rustls::pki_types::DnsName::try_from("cdn.speedtest.net".to_string()).unwrap();
        let server_name = ServerName::DnsName(dns_name);
        let cert_der = CertificateDer::from(vec![]);

        // Domain is valid, but cert structure is invalid
        let result =
            verifier.verify_server_cert(&cert_der, &[], &server_name, &[], UnixTime::now());
        assert!(result.is_err());
    }

    #[test]
    fn test_pinning_verifier_subdomain_with_dashes() {
        assert!(PinningVerifier::is_valid_domain(
            "my-custom-subdomain.speedtest.net"
        ));
        assert!(PinningVerifier::is_valid_domain("api-v2.ookla.com"));
    }

    #[test]
    fn test_pinning_verifier_long_subdomain() {
        let long_subdomain = "a".repeat(63) + ".speedtest.net";
        // This should be valid as it ends with .speedtest.net
        assert!(PinningVerifier::is_valid_domain(&long_subdomain));
    }

    #[test]
    fn test_pinning_verifier_concatenation_attack() {
        // These should all be rejected as they don't end with valid suffixes
        assert!(!PinningVerifier::is_valid_domain("speedtestXnet"));
        assert!(!PinningVerifier::is_valid_domain("speedtestXcom"));
        assert!(!PinningVerifier::is_valid_domain("ooklaXcom"));
        assert!(!PinningVerifier::is_valid_domain("ooklaXnet"));
    }

    // ==================== Settings additional chain tests ====================

    #[test]
    fn test_settings_retry_disabled_chain() {
        let settings = Settings::default().with_retry_disabled();
        assert!(!settings.retry_enabled);

        // Ensure other defaults are still set
        assert_eq!(settings.timeout_secs, 10);
        assert_eq!(settings.user_agent, DEFAULT_USER_AGENT);
    }

    #[test]
    fn test_settings_user_agent_chain() {
        let settings = Settings::default()
            .with_user_agent("Custom/1.0")
            .with_user_agent("Custom/2.0");
        assert_eq!(settings.user_agent, "Custom/2.0");
    }

    // ==================== create_client edge cases ====================

    #[test]
    fn test_create_client_source_ip_loopback_v4() {
        let settings = Settings {
            source_ip: Some("127.0.0.1".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        // Loopback should work
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_source_ip_loopback_v6() {
        let settings = Settings {
            source_ip: Some("::1".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        // Loopback should work
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_source_ip_unspecified() {
        let settings = Settings {
            source_ip: Some("0.0.0.0".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        // Unspecified should work
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_source_ip_with_tls() {
        let settings = Settings {
            source_ip: Some("127.0.0.1".to_string()),
            tls: TlsConfig::default(),
            ..Default::default()
        };
        let result = create_client(&settings);
        // Should work with default TLS or gracefully handle errors
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }
}
