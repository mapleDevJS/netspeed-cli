use crate::common;
use crate::error::Error;
use crate::test_config::TestConfig;
use reqwest::Client;

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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            source_ip: None,
            // Default browser-like user agent for speedtest.net compatibility
            // Can be overridden via config file with custom_user_agent option
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            retry_enabled: true,
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
        .user_agent(&settings.user_agent);

    if let Some(ref source_ip) = settings.source_ip {
        let addr: std::net::SocketAddr = source_ip
            .parse()
            .map_err(|e| Error::with_source("Invalid source IP", e))?;
        builder = builder.local_address(addr.ip());
    }

    let client = builder.build().map_err(Error::NetworkError)?;

    Ok(client)
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
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["netspeed-cli"]);
        let config = crate::config::Config::from_args(&args);
        let settings = Settings {
            timeout_secs: config.timeout,
            source_ip: Some("invalid-ip".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Context { .. }));
    }

    #[test]
    fn test_create_client_valid_config() {
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["netspeed-cli"]);
        let config = crate::config::Config::from_args(&args);
        let settings = Settings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_source_ip() {
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["netspeed-cli", "--source", "0.0.0.0"]);
        let config = crate::config::Config::from_args(&args);
        let settings = Settings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        match result {
            Ok(_) | Err(Error::NetworkError(_) | Error::Context { .. }) => {}
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_custom_timeout() {
        use crate::cli::Args;
        use clap::Parser;
        let args = Args::parse_from(["netspeed-cli", "--timeout", "30"]);
        let config = crate::config::Config::from_args(&args);
        let settings = Settings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }
}
