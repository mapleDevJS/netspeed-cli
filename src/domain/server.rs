//! Server discovery and selection.
//!
//! This module provides the core server discovery and selection logic.

use crate::error::Error;
use crate::servers as impl_;
use crate::types::{ClientLocation, Server};
use reqwest::Client;

/// Server discovery result.
///
/// Contains the list of available servers and the client's location.
pub struct ServerDiscovery {
    /// Available servers from speedtest.net
    pub servers: Vec<Server>,
    /// Client location from speedtest.net config API
    pub client_location: Option<ClientLocation>,
}

/// Fetch available servers and client location.
///
/// # Errors
///
/// Returns [`Error::ServerListFetch`] if the server list cannot be fetched.
pub async fn fetch(client: &Client) -> Result<ServerDiscovery, Error> {
    let (servers, client_location) = impl_::fetch(client).await?;
    Ok(ServerDiscovery {
        servers,
        client_location,
    })
}

/// Fetch just the client location from speedtest.net.
///
/// # Errors
///
/// Returns [`Error::Context`] if location cannot be determined.
pub async fn fetch_client_location(client: &Client) -> Result<ClientLocation, Error> {
    impl_::fetch_client_location(client).await
}

/// Select the best server from a list based on distance.
///
/// Returns the server with the lowest distance to the client.
///
/// # Errors
///
/// Returns [`Error::ServerNotFound`] if the server list is empty.
pub fn select_best_server(servers: &[Server]) -> Result<Server, Error> {
    impl_::select_best_server(servers)
}

/// Run a ping test against a server.
///
/// Returns (average latency, jitter, packet_loss%, individual samples).
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if all ping attempts fail.
pub async fn ping_test(
    client: &Client,
    server: &Server,
) -> Result<(f64, f64, f64, Vec<f64>), Error> {
    impl_::ping_test(client, server).await
}

/// Calculate distance between two geographic points using Haversine formula.
///
/// # Examples
///
/// ```
/// use netspeed_cli::domain::server::calculate_distance;
/// let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
/// assert!((dist - 3944.0).abs() < 200.0); // ~3944 km, NYC to LA
/// ```
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    impl_::calculate_distance(lat1, lon1, lat2, lon2)
}

/// Measure latency while running a bandwidth test.
///
/// This runs in the background during download/upload tests to measure
/// latency under load (bufferbloat detection).
pub async fn measure_latency_under_load(
    client: Client,
    url: String,
    samples: std::sync::Arc<std::sync::Mutex<Vec<f64>>>,
    stop_signal: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    impl_::measure_latency_under_load(client, url, samples, stop_signal).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_best_server_empty() {
        let result = select_best_server(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_distance_nyc_to_la() {
        let dist = calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((dist - 3944.0).abs() < 200.0);
    }

    #[test]
    fn test_calculate_distance_same_location() {
        let dist = calculate_distance(40.7128, -74.0060, 40.7128, -74.0060);
        assert!(dist < 1.0); // Should be essentially 0
    }
}
