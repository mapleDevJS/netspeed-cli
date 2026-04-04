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
