use reqwest::Client;
use quick_xml::de::from_str;
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::types::{Server, ServerConfig};

const SPEEDTEST_CONFIG_URL: &str = "https://www.speedtest.net/speedtest-config.php";
const SPEEDTEST_SERVERS_URL: &str = "https://www.speedtest.net/speedtest-servers-static.php";

pub async fn fetch_servers(client: &Client, _config: &Config) -> Result<Vec<Server>, SpeedtestError> {
    // Fetch servers list
    let response = client
        .get(SPEEDTEST_SERVERS_URL)
        .send()
        .await?
        .text()
        .await?;

    let server_config: ServerConfig = from_str(&response)?;

    // Calculate distances for each server (simplified - would need client location)
    let mut servers = server_config.servers;
    for server in &mut servers {
        // Simplified distance calculation
        server.distance = 0.0;
        server.latency = 0.0;
    }

    Ok(servers)
}

pub fn select_best_server(servers: &[Server]) -> Result<Server, SpeedtestError> {
    if servers.is_empty() {
        return Err(SpeedtestError::ServerNotFound(
            "No servers available".to_string(),
        ));
    }

    // Select server with lowest distance (closest)
    let best = servers
        .iter()
        .min_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .ok_or_else(|| SpeedtestError::ServerNotFound("No servers available".to_string()))?;

    Ok(best)
}

pub async fn ping_test(client: &Client, server: &Server) -> Result<f64, SpeedtestError> {
    let mut latencies = Vec::new();

    // Perform multiple ping measurements
    for _ in 0..4 {
        let start = std::time::Instant::now();

        let _ = client
            .get(format!("{}/latency.txt", server.url))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        latencies.push(elapsed);
    }

    // Calculate average latency
    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    Ok(avg)
}
