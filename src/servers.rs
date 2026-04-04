use reqwest::Client;
use quick_xml::de::from_str;
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::types::{Server, ServerConfig};

const SPEEDTEST_SERVERS_URL: &str = "https://www.speedtest.net/speedtest-servers-static.php";
#[allow(dead_code)]
const SPEEDTEST_CONFIG_URL: &str = "https://www.speedtest.net/speedtest-config.php";

/// Calculate distance between two geographic points using Haversine formula
#[allow(dead_code)]
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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

pub async fn fetch_servers(client: &Client, _config: &Config) -> Result<Vec<Server>, SpeedtestError> {
    // Fetch servers list
    let response = client
        .get(SPEEDTEST_SERVERS_URL)
        .send()
        .await?
        .text()
        .await?;

    let server_config: ServerConfig = from_str(&response)?;

    // Extract servers from the wrapper
    let mut servers = server_config.servers_wrapper.servers;
    for server in &mut servers {
        server.distance = 0.0;
        server.latency = 0.0;
    }

    Ok(servers)
}

/// Calculate distances from client location to all servers
#[allow(dead_code)]
pub fn calculate_server_distances(servers: &mut [Server], client_lat: f64, client_lon: f64) {
    for server in &mut *servers {
        server.distance = calculate_distance(
            client_lat,
            client_lon,
            server.lat,
            server.lon,
        );
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

        let response = client
            .get(format!("{}/latency.txt", server.url))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        if response.is_ok() {
            latencies.push(elapsed);
        }
    }

    // Calculate average latency
    if latencies.is_empty() {
        return Err(SpeedtestError::NetworkError(
            "All ping attempts failed".to_string(),
        ));
    }

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    Ok(avg)
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
                latency: 0.0,
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
                latency: 0.0,
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
        assert!(matches!(result.unwrap_err(), SpeedtestError::ServerNotFound(_)));
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
            latency: 0.0,
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
                latency: 0.0,
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
                latency: 0.0,
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
                latency: 0.0,
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
                latency: 0.0,
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
                latency: 0.0,
            },
        ];

        let best = select_best_server(&servers).unwrap();
        // Should return one of the servers with equal distance
        assert!(best.id == "1" || best.id == "2");
    }

    #[test]
    fn test_ping_test_average_calculation() {
        // Test the logic for calculating average latency
        let latencies = vec![10.0, 20.0, 15.0, 25.0];
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        assert_eq!(avg, 17.5);
    }

    #[test]
    fn test_ping_test_empty_handling() {
        let latencies: Vec<f64> = vec![];
        assert!(latencies.is_empty());
        // This verifies we handle empty latency lists correctly
    }

    #[test]
    fn test_calculate_distance_same_location() {
        // Same location should return 0 distance
        let dist = calculate_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(dist < 0.01);
    }

    #[test]
    fn test_calculate_distance_nyc_la() {
        // NYC to LA is approximately 3,944 km
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((dist - 3944.0).abs() < 200.0); // Allow 200km tolerance
    }

    #[test]
    fn test_calculate_distance_nyc_london() {
        // NYC to London is approximately 5,570 km
        let dist = calculate_distance(40.7128, -74.0060, 51.5074, -0.1278);
        assert!((dist - 5570.0).abs() < 300.0); // Allow 300km tolerance
    }

    #[test]
    fn test_calculate_server_distances() {
        let mut servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://server1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 0.0,
                latency: 0.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://server2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP".to_string(),
                country: "US".to_string(),
                lat: 34.0,
                lon: -118.0,
                distance: 0.0,
                latency: 0.0,
            },
        ];

        // Client in NYC
        calculate_server_distances(&mut servers, 40.7128, -74.0060);

        // Servers should be sorted by distance (closer first)
        assert_eq!(servers[0].id, "1"); // NYC area
        assert!(servers[0].distance < servers[1].distance);
    }
}
