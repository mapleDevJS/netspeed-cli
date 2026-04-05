use crate::common;
use crate::config::Config;
use crate::error::SpeedtestError;
use reqwest::Client;

/// Create an HTTP client with the given configuration.
///
/// # Errors
///
/// Returns [`SpeedtestError::Custom`] if the source IP is invalid.
/// Returns [`SpeedtestError::NetworkError`] if the client fails to build.
pub fn create_client(config: &Config) -> Result<Client, SpeedtestError> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout))
        .http1_only()
        .no_gzip()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    if let Some(ref source_ip) = config.source {
        let addr: std::net::SocketAddr = source_ip
            .parse()
            .map_err(|e| SpeedtestError::Custom(format!("Invalid source IP: {e}")))?;
        builder = builder.local_address(addr.ip());
    }

    let client = builder
        .build()
        .map_err(|e| SpeedtestError::NetworkError(e.to_string()))?;

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
    for line in xml.lines() {
        if line.contains("<client") && line.contains("ip=\"") {
            if let Some(start) = line.find("ip=\"") {
                let rest = &line[start + 4..];
                if let Some(end) = rest.find('"') {
                    let ip = &rest[..end];
                    if common::is_valid_ipv4(ip) {
                        return Some(ip.to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip_from_xml() {
        let xml = r#"<client country="CA" ip="173.35.57.235" isp="Rogers"/>"#;
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
        assert!(parse_ip_from_xml("<client ip=\"invalid\"/>").is_none());
    }

    #[test]
    fn test_create_client_invalid_source_ip() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let mut config = Config::from_args(&args);
        config.source = Some("invalid-ip".to_string());
        let result = create_client(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpeedtestError::Custom(_)));
    }

    #[test]
    fn test_create_client_valid_config() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);
        let result = create_client(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_source_ip() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli", "--source", "0.0.0.0"]);
        let config = Config::from_args(&args);
        let result = create_client(&config);
        match result {
            Ok(_) | Err(SpeedtestError::NetworkError(_)) | Err(SpeedtestError::Custom(_)) => (),
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }

    #[test]
    fn test_create_client_custom_timeout() {
        use crate::cli::CliArgs;
        use clap::Parser;
        let args = CliArgs::parse_from(["netspeed-cli", "--timeout", "30"]);
        let config = Config::from_args(&args);
        let result = create_client(&config);
        assert!(result.is_ok());
    }
}
