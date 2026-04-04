use reqwest::Client;
use serde::Deserialize;
use quick_xml::de::from_str;
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::http::build_base_url;
use crate::types::Server;

const SPEEDTEST_SERVERS_PATH: &str = "/speedtest-servers-static.php";
const SPEEDTEST_CONFIG_PATH: &str = "/speedtest-config.php";

#[derive(Debug, Deserialize)]
struct SpeedtestServers {
    #[serde(rename = "servers", default)]
    servers_list: ServersList,
}

#[derive(Debug, Deserialize, Default)]
struct ServersList {
    #[serde(rename = "server", default)]
    servers: Vec<RawServer>,
}

#[derive(Debug, Deserialize)]
struct RawServer {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "url")]
    url: String,
    #[serde(rename = "lat")]
    lat: String,
    #[serde(rename = "lon")]
    lon: String,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "country")]
    country: String,
    #[serde(rename = "sponsor")]
    sponsor: String,
}

#[derive(Debug, Deserialize)]
pub struct SpeedtestConfig {
    #[serde(rename = "client")]
    pub client_info: Option<ClientConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "ip")]
    pub ip: String,
    #[serde(rename = "lat")]
    pub lat: String,
    #[serde(rename = "lon")]
    pub lon: String,
}

pub async fn fetch_servers(client: &Client, config: &Config) -> Result<Vec<Server>, SpeedtestError> {
    let base_url = build_base_url(config.secure);
    let servers_url = format!("{}{}", base_url, SPEEDTEST_SERVERS_PATH);

    // Fetch servers list
    let response = client
        .get(&servers_url)
        .send()
        .await?
        .text()
        .await?;

    let parsed: SpeedtestServers = from_str(&response)?;

    // Convert raw servers to Server structs
    let servers: Vec<Server> = parsed
        .servers_list
        .servers
        .into_iter()
        .filter_map(|raw| {
            let lat = raw.lat.parse::<f64>().ok()?;
            let lon = raw.lon.parse::<f64>().ok()?;

            Some(Server {
                id: raw.id,
                url: raw.url,
                name: raw.name,
                sponsor: raw.sponsor,
                country: raw.country,
                lat,
                lon,
                distance: 0.0, // Will be calculated later
                latency: 0.0,  // Will be measured during ping test
            })
        })
        .collect();

    Ok(servers)
}

pub async fn fetch_client_config(client: &Client) -> Result<SpeedtestConfig, SpeedtestError> {
    let base_url = build_base_url(true); // Config always HTTPS
    let config_url = format!("{}{}", base_url, SPEEDTEST_CONFIG_PATH);

    let response = client
        .get(&config_url)
        .send()
        .await?
        .text()
        .await?;

    let config: SpeedtestConfig = from_str(&response)?;

    Ok(config)
}

pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    // Haversine formula to calculate distance between two points
    let earth_radius_km = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_km * c
}

pub fn calculate_distances(servers: &mut [Server], client_lat: f64, client_lon: f64) {
    for server in servers.iter_mut() {
        server.distance = calculate_distance(client_lat, client_lon, server.lat, server.lon);
    }

    // Sort by distance
    servers.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

pub fn select_best_server(servers: &[Server]) -> Result<Server, SpeedtestError> {
    if servers.is_empty() {
        return Err(SpeedtestError::ServerNotFound(
            "No servers available".to_string(),
        ));
    }

    // Servers are already sorted by distance in calculate_distances
    servers
        .first()
        .cloned()
        .ok_or_else(|| SpeedtestError::ServerNotFound("No servers available".to_string()))
}

pub async fn ping_test(client: &Client, server: &Server) -> Result<f64, SpeedtestError> {
    let mut latencies = Vec::new();

    // Perform multiple ping measurements
    for _ in 0..4 {
        let start = std::time::Instant::now();

        let _ = client
            .get(format!("{}/latency.txt", server.url.trim_end_matches('/')))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        latencies.push(elapsed);
    }

    // Calculate average latency, excluding the first (often includes connection setup)
    if latencies.len() > 1 {
        latencies.remove(0);
    }

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    Ok(avg)
}
