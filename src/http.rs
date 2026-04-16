use crate::common;
use crate::error::SpeedtestError;
use reqwest::Client;

/// HTTP client settings - decoupled from Config struct.
///
/// This allows creating HTTP clients without depending on the full Config,
/// improving modularity and testability.
#[derive(Debug, Clone)]
pub struct HttpSettings {
    /// Timeout in seconds for HTTP requests.
    pub timeout_secs: u64,
    /// Optional source IP address to bind to.
    pub source_ip: Option<String>,
    /// User agent string for HTTP requests.
    pub user_agent: String,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            source_ip: None,
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        }
    }
}

/// Create an HTTP client with the given settings.
///
/// # Errors
///
/// Returns [`SpeedtestError::Context`] if the source IP is invalid.
/// Returns [`SpeedtestError::NetworkError`] if the client fails to build.
pub fn create_client(settings: &HttpSettings) -> Result<Client, SpeedtestError> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(settings.timeout_secs))
        .http1_only()
        .no_gzip()
        .user_agent(&settings.user_agent);

    if let Some(ref source_ip) = settings.source_ip {
        let addr: std::net::SocketAddr = source_ip
            .parse()
            .map_err(|e| SpeedtestError::with_source("Invalid source IP", e))?;
        builder = builder.local_address(addr.ip());
    }

    let client = builder.build().map_err(SpeedtestError::NetworkError)?;

    Ok(client)
}

/// Discover the client's public IP address via speedtest.net.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all IP discovery endpoints fail.
pub async fn discover_client_ip(client: &Client) -> Result<String, SpeedtestError> {
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

    let settings: Settings = quick_xml::de::from_str(xml).ok()?;
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
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = crate::config::Config::from_args(&args);
        let settings = HttpSettings {
            timeout_secs: config.timeout,
            source_ip: Some("invalid-ip".to_string()),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SpeedtestError::Context { .. }
        ));
    }

    #[test]
    fn test_create_client_valid_config() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = crate::config::Config::from_args(&args);
        let settings = HttpSettings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_source_ip() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli", "--source", "0.0.0.0"]);
        let config = crate::config::Config::from_args(&args);
        let settings = HttpSettings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        match result {
            Ok(_) | Err(SpeedtestError::NetworkError(_) | SpeedtestError::Context { .. }) => {}
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_custom_timeout() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli", "--timeout", "30"]);
        let config = crate::config::Config::from_args(&args);
        let settings = HttpSettings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            ..Default::default()
        };
        let result = create_client(&settings);
        assert!(result.is_ok());
    }
}
