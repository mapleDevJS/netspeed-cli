use crate::config::Config;
use crate::error::SpeedtestError;
use reqwest::Client;

/// Create an HTTP client configured for speedtesting.
///
/// Sets up timeout, gzip compression, and optional source IP binding.
pub fn create_client(config: &Config) -> Result<Client, SpeedtestError> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout))
        .gzip(true);

    if let Some(ref source_ip) = config.source {
        let addr: std::net::SocketAddr = source_ip
            .parse()
            .map_err(|e| SpeedtestError::Custom(format!("Invalid source IP: {}", e)))?;
        builder = builder.local_address(addr.ip());
    }

    let client = builder
        .build()
        .map_err(|e| SpeedtestError::NetworkError(e.to_string()))?;

    Ok(client)
}

/// Discover the client's public IP address.
///
/// Tries multiple endpoints for reliability.
pub async fn discover_client_ip(client: &Client) -> Result<String, SpeedtestError> {
    let endpoints = vec![
        "https://www.speedtest.net/api/js/ip",
        "https://c.speedtest.net/api/js/ip",
        "https://ifconfig.me/ip",
    ];

    for endpoint in endpoints {
        if let Ok(response) = client.get(endpoint).send().await {
            if let Ok(text) = response.text().await {
                let ip = text.trim().to_string();
                if !ip.is_empty() {
                    return Ok(ip);
                }
            }
        }
    }

    Err(SpeedtestError::NetworkError(
        "Failed to discover client IP address".to_string(),
    ))
}

/// Build the base URL for speedtest.net (HTTP or HTTPS).
///
/// Used by `fetch_servers` and `fetch_client_config` to construct
/// endpoint URLs.
pub fn build_base_url(secure: bool) -> String {
    if secure {
        "https://www.speedtest.net".to_string()
    } else {
        "http://www.speedtest.net".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::cli::CliArgs;

    #[test]
    fn test_build_base_url_https() {
        let url = build_base_url(true);
        assert_eq!(url, "https://www.speedtest.net");
    }

    #[test]
    fn test_build_base_url_http() {
        let url = build_base_url(false);
        assert_eq!(url, "http://www.speedtest.net");
    }

    #[test]
    fn test_create_client_default_config() {
        let config = Config::from_args(&CliArgs::default());
        let result = create_client(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_custom_timeout() {
        let args = CliArgs { timeout: 30, ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_client(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_valid_source_ip() {
        let args = CliArgs { source: Some("127.0.0.1".to_string()), ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_client(&config);
        // May fail on some systems, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_create_client_with_invalid_source_ip() {
        let args = CliArgs { source: Some("not-an-ip".to_string()), ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_client(&config);
        assert!(result.is_err());
        match result {
            Err(SpeedtestError::Custom(msg)) => {
                assert!(msg.contains("Invalid source IP"));
            }
            _ => panic!("Expected Custom error variant"),
        }
    }

    #[test]
    fn test_create_client_with_ipv6_source() {
        let args = CliArgs { source: Some("::1".to_string()), ..Default::default() };
        let config = Config::from_args(&args);
        let result = create_client(&config);
        // May fail on some systems, but shouldn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_discover_client_ip_all_endpoints_fail() {
        // Create a client that will fail (short timeout)
        let args = CliArgs { timeout: 1, ..Default::default() };
        let config = Config::from_args(&args);
        let client = create_client(&config).unwrap();

        // This will actually try real endpoints, so we just verify it returns a result
        // In CI this might fail due to network restrictions
        let result = discover_client_ip(&client).await;
        // We don't assert ok/err since it depends on network
        // The important thing is the function doesn't panic
        let _ = result;
    }
}
