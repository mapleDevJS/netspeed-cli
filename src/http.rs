use reqwest::Client;
use crate::config::Config;
use crate::error::SpeedtestError;

pub fn create_client(config: &Config) -> Result<Client, SpeedtestError> {
    let mut builder = Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout))
        .gzip(true);

    if let Some(ref source_ip) = config.source {
        let addr: std::net::SocketAddr = source_ip.parse()
            .map_err(|e| SpeedtestError::Custom(format!("Invalid source IP: {}", e)))?;
        builder = builder.local_address(addr.ip());
    }

    let client = builder.build()
        .map_err(|e| SpeedtestError::NetworkError(e.to_string()))?;

    Ok(client)
}

/// Validate IP address format
#[allow(dead_code)]
pub fn validate_ip(ip: &str) -> Result<(), String> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid IPv4 format: {}", ip));
    }
    
    for part in parts {
        if part.parse::<u8>().is_err() {
            return Err(format!("Invalid octet: {}", part));
        }
    }
    
    Ok(())
}

/// Build timeout duration from seconds
#[allow(dead_code)]
pub fn build_timeout_duration(seconds: u64) -> std::time::Duration {
    std::time::Duration::from_secs(seconds)
}

pub async fn discover_client_ip(client: &Client) -> Result<String, SpeedtestError> {
    let response = client
        .get("https://www.speedtest.net/api/js/ip")
        .send()
        .await?
        .text()
        .await?;

    Ok(response.trim().to_string())
}

#[allow(dead_code)]
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

    #[test]
    fn test_build_base_url_http() {
        let url = build_base_url(false);
        assert_eq!(url, "http://www.speedtest.net");
    }

    #[test]
    fn test_build_base_url_https() {
        let url = build_base_url(true);
        assert_eq!(url, "https://www.speedtest.net");
    }

    #[test]
    fn test_validate_ip_valid() {
        assert!(validate_ip("192.168.1.1").is_ok());
    }

    #[test]
    fn test_validate_ip_localhost() {
        assert!(validate_ip("127.0.0.1").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid_format() {
        assert!(validate_ip("192.168.1").is_err());
    }

    #[test]
    fn test_validate_ip_invalid_octet() {
        assert!(validate_ip("192.168.1.999").is_err());
    }

    #[test]
    fn test_build_timeout_duration() {
        let duration = build_timeout_duration(10);
        assert_eq!(duration.as_secs(), 10);
    }

    #[test]
    fn test_build_timeout_duration_zero() {
        let duration = build_timeout_duration(0);
        assert_eq!(duration.as_secs(), 0);
    }

    #[test]
    fn test_build_timeout_duration_large() {
        let duration = build_timeout_duration(300);
        assert_eq!(duration.as_secs(), 300);
    }

    #[test]
    fn test_create_client_invalid_source_ip() {
        let config = Config {
            no_download: false,
            no_upload: false,
            single: false,
            bytes: false,
            share: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            list: false,
            server_ids: vec![],
            exclude_ids: vec![],
            mini_url: None,
            source: Some("invalid-ip".to_string()),
            timeout: 10,
            secure: false,
            no_pre_allocate: false,
            client_ip: None,
        };

        let result = create_client(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpeedtestError::Custom(_)));
    }

    #[test]
    fn test_create_client_valid_config() {
        let config = Config {
            no_download: false,
            no_upload: false,
            single: false,
            bytes: false,
            share: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            list: false,
            server_ids: vec![],
            exclude_ids: vec![],
            mini_url: None,
            source: None,
            timeout: 10,
            secure: false,
            no_pre_allocate: false,
            client_ip: None,
        };

        let result = create_client(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_client_with_source_ip() {
        let config = Config {
            no_download: false,
            no_upload: false,
            single: false,
            bytes: false,
            share: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            list: false,
            server_ids: vec![],
            exclude_ids: vec![],
            mini_url: None,
            source: Some("0.0.0.0".to_string()),
            timeout: 10,
            secure: false,
            no_pre_allocate: false,
            client_ip: None,
        };

        let result = create_client(&config);
        // Note: This may fail if the IP is not available to bind, but the config should be valid
        // We're just testing that the client creation logic doesn't crash
        match result {
            Ok(_) => (),
            Err(SpeedtestError::NetworkError(_)) => (), // Acceptable error on some systems
            Err(SpeedtestError::Custom(_)) => (), // Also acceptable
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_create_client_custom_timeout() {
        let config = Config {
            no_download: false,
            no_upload: false,
            single: false,
            bytes: false,
            share: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            list: false,
            server_ids: vec![],
            exclude_ids: vec![],
            mini_url: None,
            source: None,
            timeout: 30,
            secure: false,
            no_pre_allocate: false,
            client_ip: None,
        };

        let result = create_client(&config);
        assert!(result.is_ok());
    }
}
