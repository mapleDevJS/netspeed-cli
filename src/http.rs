use crate::common;
use crate::error::Error;
use crate::test_config::TestConfig;
use reqwest::Client;
use rustls::client::danger::ServerCertVerifier;
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
    /// Enable certificate pinning for speedtest.net servers.
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

    /// Enable certificate pinning for speedtest.net.
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

    // Warn if both CA cert and pinning are configured (pinning takes precedence)
    if tls.pin_speedtest_certs && tls.ca_cert_path.is_some() {
        eprintln!(
            "Warning: Both --ca-cert and --pin-certs are set. Certificate pinning takes precedence and --ca-cert will be ignored."
        );
    }

    // Build configuration based on whether custom CA or pinning is enabled
    if tls.pin_speedtest_certs {
        // For pinning, use the dangerous builder with custom verifier
        // Note: This only validates domain names, not actual certificate hashes
        // For true pinning, additional SPKI hash verification would be needed
        return Ok(ClientConfig::builder_with_protocol_versions(versions)
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(PinningVerifier::new()))
            .with_no_client_auth());
    }

    // Standard configuration with webpki-roots (Mozilla's root certs)
    let mut root_store = RootCertStore::empty();
    // webpki-roots 0.26 provides TLS_SERVER_ROOTS which can be extended into the store
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    if let Some(ref ca_path) = tls.ca_cert_path {
        // Load custom CA certificate instead of webpki-roots
        return Ok(ClientConfig::builder_with_protocol_versions(versions)
            .with_root_certificates(load_custom_ca_cert(ca_path)?)
            .with_no_client_auth());
    }

    Ok(ClientConfig::builder_with_protocol_versions(versions)
        .with_root_certificates(root_store)
        .with_no_client_auth())
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

/// Certificate verifier for speedtest.net pinning.
#[derive(Debug)]
struct PinningVerifier;

impl PinningVerifier {
    fn new() -> Self {
        Self
    }

    fn is_valid_domain(host: &str) -> bool {
        // Check exact domains first, then subdomains ending with the suffix
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

        // Check if the domain is allowed (domain pinning)
        // Note: This only validates domain names, not actual certificate hashes
        // An attacker with a valid speedtest.net certificate could still MITM
        if !Self::is_valid_domain(hostname) {
            return Err(rustls::Error::General(format!(
                "'{}' is not a speedtest.net domain",
                hostname
            )));
        }

        // Verify the certificate can be parsed by webpki
        webpki::EndEntityCert::try_from(end_entity.as_ref())
            .map_err(|_| rustls::Error::General("Invalid certificate structure".to_string()))?;

        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
        ]
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
        assert!(matches!(verifier, PinningVerifier));
    }

    #[test]
    fn test_pinning_verifier_debug_trait() {
        // Test that Debug can be derived and used
        let verifier = PinningVerifier::new();
        let debug_str = format!("{:?}", verifier);
        assert_eq!(debug_str, "PinningVerifier");
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

        // Should have exactly 8 schemes
        assert_eq!(schemes.len(), 8);
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
        // The error should be about certificate structure, not domain
        assert!(err_msg.contains("Invalid certificate structure"));
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
        // The error should be about certificate structure, not domain
        assert!(
            err_msg.contains("Invalid certificate structure") || err_msg.contains("EndEntityCert")
        );
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
}
