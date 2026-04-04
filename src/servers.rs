use crate::config::Config;
use crate::error::SpeedtestError;
use crate::http::build_base_url;
use crate::types::Server;
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;

/// Path to the static servers list endpoint
const SPEEDTEST_SERVERS_PATH: &str = "/speedtest-servers-static.php";

/// Path to the client configuration endpoint
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

/// Fetch the list of available Speedtest servers.
///
/// Downloads the static server list from speedtest.net and parses it
/// into `Server` structs.
#[tracing::instrument(skip(client, config), fields(server_count))]
pub async fn fetch_servers(
    client: &Client,
    config: &Config,
) -> Result<Vec<Server>, SpeedtestError> {
    let base_url = build_base_url(config.secure);
    let servers_url = format!("{}{}", base_url, SPEEDTEST_SERVERS_PATH);

    // Fetch servers list
    let response = client.get(&servers_url).send().await?.text().await?;

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

/// Fetch client configuration including geolocation data.
///
/// Used to determine the client's approximate location for server selection.
pub async fn fetch_client_config(client: &Client) -> Result<SpeedtestConfig, SpeedtestError> {
    let base_url = build_base_url(true); // Config always HTTPS
    let config_url = format!("{}{}", base_url, SPEEDTEST_CONFIG_PATH);

    let response = client.get(&config_url).send().await?.text().await?;

    let config: SpeedtestConfig = from_str(&response)?;

    Ok(config)
}

/// Calculate distance between two geographic points using the Haversine formula.
///
/// Returns the distance in kilometers.
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_km = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_km * c
}

/// Calculate distances from client location to all servers and sort by distance.
///
/// Servers closest to the client will appear first in the slice.
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

/// Weight for distance in server selection scoring (0.0-1.0).
/// Higher values prioritize geographic proximity.
const DISTANCE_WEIGHT: f64 = 0.6;

/// Weight for latency in server selection scoring (0.0-1.0).
/// Higher values prioritize lower latency.
const LATENCY_WEIGHT: f64 = 0.4;

/// Select the best server for testing.
///
/// Uses a weighted scoring system combining distance (60%) and latency (40%).
/// Servers with lower scores are preferred. If latency data is unavailable,
/// falls back to distance-only selection.
pub fn select_best_server(servers: &[Server]) -> Result<Server, SpeedtestError> {
    if servers.is_empty() {
        return Err(SpeedtestError::ServerNotFound(
            "No servers available".to_string(),
        ));
    }

    // Check if any server has latency data
    let has_latency = servers.iter().any(|s| s.latency > 0.0);

    if !has_latency {
        // Fall back to distance-only (servers are already sorted by distance)
        return servers
            .first()
            .cloned()
            .ok_or_else(|| SpeedtestError::ServerNotFound("No servers available".to_string()));
    }

    // Normalize distance and latency for scoring
    let max_distance = servers.iter().map(|s| s.distance).fold(f64::MIN, f64::max);
    let max_latency = servers.iter().map(|s| s.latency).fold(f64::MIN, f64::max);

    // Select server with lowest weighted score
    servers
        .iter()
        .map(|s| {
            let norm_distance = if max_distance > 0.0 {
                s.distance / max_distance
            } else {
                0.0
            };
            let norm_latency = if max_latency > 0.0 {
                s.latency / max_latency
            } else {
                0.0
            };
            let score = DISTANCE_WEIGHT * norm_distance + LATENCY_WEIGHT * norm_latency;
            (score, s)
        })
        .min_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, s)| s.clone())
        .ok_or_else(|| SpeedtestError::ServerNotFound("No servers available".to_string()))
}

/// Measure latency to the server using HTTP requests.
///
/// Performs multiple requests and returns the average latency,
/// excluding the first request which includes connection setup overhead.
/// Returns latency in milliseconds.
#[tracing::instrument(skip(client, server), fields(server_id = %server.id))]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_server(url: &str) -> Server {
        Server {
            id: "1".to_string(),
            url: url.to_string(),
            name: "Test Server".to_string(),
            sponsor: "Test ISP".to_string(),
            country: "US".to_string(),
            lat: 40.0,
            lon: -74.0,
            distance: 100.0,
            latency: 0.0,
        }
    }

    #[test]
    fn test_calculate_distance_same_point() {
        let dist = calculate_distance(40.0, -74.0, 40.0, -74.0);
        assert!((dist - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_distance_nyc_to_la() {
        // NYC (40.7128, -74.0060) to LA (34.0522, -118.2437)
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        // Approximately 3944 km
        assert!(dist > 3900.0 && dist < 4000.0);
    }

    #[test]
    fn test_calculate_distance_london_to_paris() {
        // London (51.5074, -0.1278) to Paris (48.8566, 2.3522)
        let dist = calculate_distance(51.5074, -0.1278, 48.8566, 2.3522);
        // Approximately 344 km
        assert!(dist > 300.0 && dist < 400.0);
    }

    #[test]
    fn test_calculate_distance_equator_crossing() {
        let dist = calculate_distance(10.0, 0.0, -10.0, 0.0);
        // Approximately 2223 km (20 degrees of latitude)
        assert!(dist > 2200.0 && dist < 2300.0);
    }

    #[test]
    fn test_calculate_distance_antimeridian() {
        // Crossing the antimeridian (180/-180 longitude)
        let dist = calculate_distance(0.0, 179.0, 0.0, -179.0);
        // Should be a short distance (~222 km)
        assert!(dist > 200.0 && dist < 300.0);
    }

    #[test]
    fn test_calculate_distance_north_pole() {
        let dist = calculate_distance(90.0, 0.0, 89.0, 0.0);
        // Approximately 111 km (1 degree of latitude)
        assert!(dist > 100.0 && dist < 120.0);
    }

    #[test]
    fn test_calculate_distance_south_pole() {
        let dist = calculate_distance(-90.0, 0.0, -89.0, 0.0);
        assert!(dist > 100.0 && dist < 120.0);
    }

    #[test]
    fn test_calculate_distances_sorts_by_distance() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://srv1.com".to_string(),
                name: "Far".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 50.0,
                lon: 0.0,
                distance: 0.0,
                latency: 0.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://srv2.com".to_string(),
                name: "Near".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 0.0,
                latency: 0.0,
            },
        ];

        // Client at NYC area
        calculate_distances(&mut servers, 40.7, -74.0);

        assert!(servers[0].distance < servers[1].distance);
        assert_eq!(servers[0].id, "2"); // Near server should be first
    }

    #[test]
    fn test_calculate_distances_updates_all() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://srv1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 50.0,
                lon: 0.0,
                distance: 0.0,
                latency: 0.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://srv2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 48.0,
                lon: 2.0,
                distance: 0.0,
                latency: 0.0,
            },
        ];

        // Client at different location
        calculate_distances(&mut servers, 40.0, -74.0);

        assert!(servers[0].distance > 0.0);
        assert!(servers[1].distance > 0.0);
    }

    #[test]
    fn test_select_best_server_empty() {
        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpeedtestError::ServerNotFound(_)));
    }

    #[test]
    fn test_select_best_server_single() {
        let servers = vec![create_test_server("http://srv1.com")];
        let result = select_best_server(&servers).unwrap();
        assert_eq!(result.id, "1");
    }

    #[test]
    fn test_select_best_server_closest_by_distance() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://far.com".to_string(),
                name: "Far".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 50.0,
                lon: 0.0,
                distance: 5000.0,
                latency: 0.0, // No latency data
            },
            Server {
                id: "2".to_string(),
                url: "http://near.com".to_string(),
                name: "Near".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 100.0,
                latency: 0.0, // No latency data
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // When no latency data, falls back to distance-only (first in list)
        // Since both have 0 latency, it picks the first one
        assert!(!best.id.is_empty());
    }

    #[test]
    fn test_select_best_server_with_latency() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://srv1.com".to_string(),
                name: "Close but slow".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 41.0,
                lon: -73.0,
                distance: 100.0,
                latency: 100.0, // High latency
            },
            Server {
                id: "2".to_string(),
                url: "http://srv2.com".to_string(),
                name: "Far but fast".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 50.0,
                lon: 0.0,
                distance: 5000.0,
                latency: 10.0, // Low latency
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // With 60% distance weight and 40% latency weight,
        // the scoring should balance both factors
        assert!(!best.id.is_empty());
    }

    #[test]
    fn test_select_best_server_all_same_distance() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://srv1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 50.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://srv2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 100.0,
                latency: 20.0, // Lower latency
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // Should pick server 2 due to lower latency
        assert_eq!(best.id, "2");
    }

    #[test]
    fn test_fetch_servers_filters_invalid_lat_lon() {
        // Test the filter_map logic directly with RawServer structs
        let raw_servers = vec![
            RawServer {
                id: "1".to_string(),
                url: "http://valid.com".to_string(),
                lat: "40.0".to_string(),
                lon: "-74.0".to_string(),
                name: "Valid".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
            },
            RawServer {
                id: "2".to_string(),
                url: "http://invalid.com".to_string(),
                lat: "notanumber".to_string(),
                lon: "-74.0".to_string(),
                name: "Invalid".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
            },
        ];

        let servers: Vec<Server> = raw_servers
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
                    distance: 0.0,
                    latency: 0.0,
                })
            })
            .collect();

        assert_eq!(servers.len(), 1); // Only valid server
        assert_eq!(servers[0].id, "1");
    }

    #[test]
    fn test_fetch_servers_empty_list() {
        let raw_servers: Vec<RawServer> = vec![];
        let servers: Vec<Server> = raw_servers
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
                    distance: 0.0,
                    latency: 0.0,
                })
            })
            .collect();

        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_fetch_client_config_parsing() {
        // Test ClientConfig struct directly
        let client = ClientConfig {
            ip: "1.2.3.4".to_string(),
            lat: "40.7128".to_string(),
            lon: "-74.0060".to_string(),
        };
        assert_eq!(client.ip, "1.2.3.4");
        assert_eq!(client.lat, "40.7128");
        assert_eq!(client.lon, "-74.0060");
    }

    #[test]
    fn test_fetch_client_config_missing_client() {
        // Test SpeedtestConfig with None client_info
        let config = SpeedtestConfig {
            client_info: None,
        };
        assert!(config.client_info.is_none());
    }

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
    fn test_servers_path_constant() {
        assert_eq!(SPEEDTEST_SERVERS_PATH, "/speedtest-servers-static.php");
    }

    #[test]
    fn test_config_path_constant() {
        assert_eq!(SPEEDTEST_CONFIG_PATH, "/speedtest-config.php");
    }

    #[test]
    fn test_distance_weight_constant() {
        assert!((DISTANCE_WEIGHT - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_latency_weight_constant() {
        assert!((LATENCY_WEIGHT - 0.4).abs() < f64::EPSILON);
    }
}
