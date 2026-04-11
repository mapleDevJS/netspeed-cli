//! Server discovery, selection, and ping testing.
//!
//! This module re-exports from focused sub-modules for backward compatibility:
//! - [`calculate_distance`] from [`crate::geo`] — Haversine distance formula
//! - [`fetch_servers`] from [`crate::server_fetch`] — HTTP/XML server list fetching
//! - [`ping_test`] from [`crate::ping`] — Latency/jitter/packet-loss measurement
//!
//! New code should import directly from the specific sub-module.

// Re-export focused modules for backward compatibility
pub use crate::geo::calculate_distance;
pub use crate::ping::ping_test;
pub use crate::server_fetch::fetch_servers;

/// Select the best server from a list, preferring the closest by distance.
///
/// # Errors
///
/// Returns [`crate::error::SpeedtestError::ServerNotFound`] if the server list is empty.
pub fn select_best_server(
    servers: &[crate::types::Server],
) -> Result<crate::types::Server, crate::error::SpeedtestError> {
    if servers.is_empty() {
        return Err(crate::error::SpeedtestError::ServerNotFound(
            "No servers available".to_string(),
        ));
    }

    let best = servers
        .iter()
        .min_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .ok_or_else(|| {
            crate::error::SpeedtestError::ServerNotFound("No servers available".to_string())
        })?;

    Ok(best)
}

#[cfg(test)]
mod tests {
    use crate::types::Server;

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
            crate::error::SpeedtestError::ServerNotFound(_)
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
        assert!(best.id == "1" || best.id == "2");
    }

    #[test]
    fn test_server_distance_comparison_with_negative_coords() {
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
        assert_eq!(best.id, "2");
    }
}
