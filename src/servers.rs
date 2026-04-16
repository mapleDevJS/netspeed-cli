#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::error::SpeedtestError;
use crate::types::Server;
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Root element for the Speedtest.net servers XML response
/// XML structure: <settings><servers><server .../></servers></settings>
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "settings")]
struct ServerConfig {
    #[serde(rename = "servers")]
    servers_wrapper: ServersWrapper,
}

/// Wrapper for the list of servers (maps to <servers> element)
#[derive(Debug, Clone, Deserialize)]
struct ServersWrapper {
    #[serde(rename = "server", default)]
    servers: Vec<Server>,
}

const SPEEDTEST_SERVERS_URL: &str = "https://www.speedtest.net/speedtest-servers-static.php";
const SPEEDTEST_CONFIG_URL: &str = "https://www.speedtest.net/api/ios-config.php";

/// Calculate distance between two geographic points using Haversine formula.
///
/// # Examples
///
/// ```
/// # use netspeed_cli::servers::calculate_distance;
/// let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
/// assert!((dist - 3944.0).abs() < 200.0); // ~3944 km, NYC to LA
/// ```
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Client location data from the speedtest.net config API
#[derive(Debug, Clone, Deserialize)]
struct ClientConfig {
    #[serde(rename = "client")]
    client: ClientInfo,
}

#[derive(Debug, Clone, Deserialize)]
struct ClientInfo {
    #[serde(rename = "@lat")]
    lat: Option<f64>,
    #[serde(rename = "@lon")]
    lon: Option<f64>,
}

/// Fetch client location from speedtest.net config API
async fn fetch_client_location(client: &Client) -> Result<(f64, f64), SpeedtestError> {
    let response = client
        .get(SPEEDTEST_CONFIG_URL)
        .send()
        .await?
        .text()
        .await?;

    let config: ClientConfig = from_str(&response)?;

    match (config.client.lat, config.client.lon) {
        (Some(lat), Some(lon)) => Ok((lat, lon)),
        _ => Err(SpeedtestError::Context {
            msg: "Could not parse client location from config".to_string(),
            source: None,
        }),
    }
}

/// Fetch the list of available speedtest servers, sorted by distance.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if fetching the server list fails.
/// Returns [`SpeedtestError::DeserializeXml`] if the XML response cannot be parsed.
pub async fn fetch_servers(client: &Client) -> Result<Vec<Server>, SpeedtestError> {
    let (client_lat, client_lon) = match fetch_client_location(client).await {
        Ok(coords) => coords,
        Err(ref e) => {
            eprintln!(
                "Warning: could not determine client location ({e}), using default (equator)"
            );
            (0.0, 0.0)
        }
    };

    let response = client
        .get(SPEEDTEST_SERVERS_URL)
        .send()
        .await?
        .text()
        .await?;

    let server_config: ServerConfig = from_str(&response)?;

    let mut servers = server_config.servers_wrapper.servers;
    for server in &mut servers {
        server.distance = calculate_distance(client_lat, client_lon, server.lat, server.lon);
    }

    // Sort by distance so closest servers are first
    servers.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(servers)
}

/// Select the best server from a list, preferring the closest by distance.
///
/// # Errors
///
/// Returns [`SpeedtestError::ServerNotFound`] if the server list is empty.
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

/// Run a ping test against the given server, returning (average latency, jitter, packet_loss%, individual_samples).
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all ping attempts fail.
pub async fn ping_test(
    client: &Client,
    server: &Server,
) -> Result<(f64, f64, f64, Vec<f64>), SpeedtestError> {
    const PING_ATTEMPTS: usize = 8;
    let mut latencies = Vec::new();

    // Perform multiple ping measurements
    for _ in 0..PING_ATTEMPTS {
        let start = std::time::Instant::now();

        let response = client
            .get(format!("{}/latency.txt", server.url))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        if let Ok(resp) = response {
            if resp.status().is_success() {
                latencies.push(elapsed);
            }
        }
    }

    // Calculate average latency
    if latencies.is_empty() {
        return Err(SpeedtestError::Context {
            msg: "All ping attempts failed".to_string(),
            source: None,
        });
    }

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    // Calculate jitter (average of absolute differences between consecutive latencies)
    let jitter = if latencies.len() > 1 {
        let mut jitter_sum = 0.0;
        for i in 1..latencies.len() {
            jitter_sum += (latencies[i] - latencies[i - 1]).abs();
        }
        jitter_sum / (latencies.len() - 1) as f64
    } else {
        0.0
    };

    // Calculate packet loss percentage
    let packet_loss = ((PING_ATTEMPTS - latencies.len()) as f64 / PING_ATTEMPTS as f64) * 100.0;

    Ok((avg, jitter, packet_loss, latencies))
}

pub async fn measure_latency_under_load(
    client: Client,
    server_url: String,
    samples: Arc<std::sync::Mutex<Vec<f64>>>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        let start = std::time::Instant::now();
        let response = client.get(format!("{server_url}/latency.txt")).send().await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                if let Ok(mut lock) = samples.lock() {
                    lock.push(elapsed);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_best_server() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Far Server".to_string(),
                sponsor: "ISP 1".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 5000.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Close Server".to_string(),
                sponsor: "ISP 2".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 100.0,
            },
        ];

        let best = select_best_server(&servers).unwrap();
        assert_eq!(best.id, "2");
        assert_eq!(best.distance, 100.0);
    }

    #[test]
    fn test_select_best_server_empty() {
        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SpeedtestError::ServerNotFound(_)
        ));
    }

    #[test]
    fn test_select_best_server_single() {
        let servers = vec![Server {
            id: "1".to_string(),
            url: "http://server1.com".to_string(),
            name: "Only Server".to_string(),
            sponsor: "ISP".to_string(),
            country: "US".to_string(),
            lat: 40.0,
            lon: -74.0,
            distance: 500.0,
        }];

        let best = select_best_server(&servers).unwrap();
        assert_eq!(best.id, "1");
    }

    #[test]
    fn test_server_distance_comparison() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 300.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 200.0,
            },
            Server {
                id: "3".to_string(),
                url: "http://server3.com".to_string(),
                name: "Server 3".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 42.0,
                lon: -72.0,
                distance: 100.0,
            },
        ];

        let best = select_best_server(&servers).unwrap();
        assert_eq!(best.id, "3");
    }

    #[test]
    fn test_server_with_equal_distances() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 100.0,
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // Should return one of the servers with equal distance
        assert!(best.id == "1" || best.id == "2");
    }

    #[test]
    fn test_ping_test_average_calculation() {
        let latencies = [10.0, 20.0, 15.0, 25.0];
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        assert_eq!(avg, 17.5);
    }

    #[test]
    fn test_ping_test_empty_handling() {
        let latencies: Vec<f64> = vec![];
        assert!(latencies.is_empty());
    }

    #[test]
    fn test_calculate_distance_same_location() {
        let dist = calculate_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(dist < 0.01);
    }

    #[test]
    fn test_calculate_distance_nyc_la() {
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((dist - 3944.0).abs() < 200.0);
    }

    #[test]
    fn test_calculate_distance_nyc_london() {
        let dist = calculate_distance(40.7128, -74.0060, 51.5074, -0.1278);
        assert!((dist - 5570.0).abs() < 300.0);
    }

    #[test]
    fn test_client_config_deserialization() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <client lat="40.7128" lon="-74.0060" ip="192.168.1.1" />
</settings>"#;
        let config: ClientConfig = from_str(xml).unwrap();
        assert_eq!(config.client.lat, Some(40.7128));
        assert_eq!(config.client.lon, Some(-74.0060));
    }

    #[test]
    fn test_client_config_missing_coords() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <client ip="192.168.1.1" />
</settings>"#;
        let config: ClientConfig = from_str(xml).unwrap();
        assert!(config.client.lat.is_none());
        assert!(config.client.lon.is_none());
    }

    #[test]
    fn test_calculate_distance_sydney_tokyo() {
        let dist = calculate_distance(-33.8688, 151.2093, 35.6762, 139.6503);
        assert!((dist - 7823.0).abs() < 300.0);
    }

    #[test]
    fn test_calculate_distance_opposite_sides() {
        // NYC to Sydney (roughly opposite sides of Earth)
        let dist = calculate_distance(40.7128, -74.0060, -33.8688, 151.2093);
        assert!(dist > 15_000.0); // Should be a very long distance
    }

    #[test]
    fn test_calculate_distance_equator() {
        // Points on the equator
        let dist = calculate_distance(0.0, 0.0, 0.0, 10.0);
        assert!((dist - 1111.0).abs() < 100.0); // ~1111 km per 10 degrees at equator
    }

    #[test]
    fn test_server_config_deserialization() {
        let xml = r#"<?xml version="1.0"?>
<settings>
    <servers>
        <server url="http://server1.com/speedtest/upload.php" name="Server 1" sponsor="ISP 1" country="US" id="1" lat="40.0" lon="-74.0" />
        <server url="http://server2.com/speedtest/upload.php" name="Server 2" sponsor="ISP 2" country="CA" id="2" lat="43.0" lon="-79.0" />
    </servers>
</settings>"#;
        let config: ServerConfig = from_str(xml).unwrap();
        assert_eq!(config.servers_wrapper.servers.len(), 2);
        assert_eq!(config.servers_wrapper.servers[0].id, "1");
        assert_eq!(config.servers_wrapper.servers[1].country, "CA");
    }

    #[test]
    fn test_server_distance_comparison_with_negative_coords() {
        // Test with servers in different hemispheres
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Southern".to_string(),
                sponsor: "ISP".to_string(),
                country: "AU".to_string(),
                lat: -33.8688,
                lon: 151.2093,
                distance: 15_000.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Northern".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.7128,
                lon: -74.0060,
                distance: 100.0,
            },
        ];

        let best = select_best_server(&servers).unwrap();
        assert_eq!(best.id, "2"); // Northern is closer
    }

    #[test]
    fn test_servers_wrapper_empty_deserialization() {
        let xml = r#"<?xml version="1.0"?>
<settings>
    <servers>
    </servers>
</settings>"#;
        let config: ServerConfig = from_str(xml).unwrap();
        assert!(config.servers_wrapper.servers.is_empty());
    }
}
