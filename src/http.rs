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

pub async fn discover_client_ip(client: &Client) -> Result<String, SpeedtestError> {
    // Try multiple endpoints for reliability
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

#[allow(dead_code)]
pub fn build_base_url(secure: bool) -> String {
    if secure {
        "https://www.speedtest.net".to_string()
    } else {
        "http://www.speedtest.net".to_string()
    }
}
