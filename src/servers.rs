#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::endpoints::ServerEndpoints;
use crate::error::Error;
use crate::test_config::TestConfig;
use crate::types::Server;
use futures::{StreamExt, stream};
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
#[must_use]
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let a = a.clamp(0.0, 1.0);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Client location data from the speedtest.net config API.
///
/// This struct mirrors the client element from the speedtest.net iOS config API.
/// It contains geographic coordinates and optional location metadata.
#[derive(Debug, Clone, Deserialize, Default)]
struct ClientConfig {
    #[serde(rename = "client", default)]
    client: ClientInfo,
}

/// Client information from the speedtest.net config API.
#[derive(Debug, Clone, Deserialize, Default)]
struct ClientInfo {
    #[serde(rename = "@lat", default)]
    lat: Option<f64>,
    #[serde(rename = "@lon", default)]
    lon: Option<f64>,
    #[serde(rename = "@city", default)]
    city: Option<String>,
    #[serde(rename = "@country", default)]
    country: Option<String>,
}

/// Fetch client location from speedtest.net config API.
///
/// Returns a [`crate::types::ClientLocation`] with coordinates and optional
/// city/country information from the speedtest.net iOS config endpoint.
///
/// # Errors
///
/// Returns [`Error::Context`] if the response cannot be parsed or coordinates
/// are missing.
pub async fn fetch_client_location(client: &Client) -> Result<crate::types::ClientLocation, Error> {
    fetch_client_location_from_url(client, SPEEDTEST_CONFIG_URL).await
}

fn valid_coordinates(lat: f64, lon: f64) -> bool {
    lat.is_finite()
        && lon.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lon)
}

async fn fetch_client_location_from_url(
    client: &Client,
    url: &str,
) -> Result<crate::types::ClientLocation, Error> {
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let config: ClientConfig = from_str(&response)?;

    match (config.client.lat, config.client.lon) {
        (Some(lat), Some(lon)) if valid_coordinates(lat, lon) => Ok(crate::types::ClientLocation {
            lat,
            lon,
            city: config.client.city,
            country: config.client.country,
        }),
        _ => Err(Error::Context {
            msg: "Could not parse client location from config".to_string(),
            source: None,
        }),
    }
}

/// Fetch the list of available speedtest servers, sorted by distance.
///
/// Also returns the client location if available, so callers don't need
/// to make a separate API call.
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if fetching the server list fails.
/// Returns [`Error::DeserializeXml`] if the XML response cannot be parsed.
pub async fn fetch(
    client: &Client,
) -> Result<(Vec<Server>, Option<crate::types::ClientLocation>), Error> {
    fetch_from_urls(client, SPEEDTEST_CONFIG_URL, SPEEDTEST_SERVERS_URL).await
}

async fn fetch_from_urls(
    client: &Client,
    config_url: &str,
    servers_url: &str,
) -> Result<(Vec<Server>, Option<crate::types::ClientLocation>), Error> {
    let client_location = match fetch_client_location_from_url(client, config_url).await {
        Ok(loc) => Some(loc),
        Err(ref e) => {
            eprintln!(
                "Warning: could not determine client location ({e}), using latency-based selection"
            );
            None
        }
    };

    let response = client
        .get(servers_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let server_config: ServerConfig = from_str(&response)?;

    let mut servers = server_config.servers_wrapper.servers;
    for server in &mut servers {
        server.distance = client_location
            .as_ref()
            .filter(|_| valid_coordinates(server.lat, server.lon))
            .map_or(f64::INFINITY, |loc| {
                calculate_distance(loc.lat, loc.lon, server.lat, server.lon)
            });
    }

    // Sort by distance so closest servers are first
    servers.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok((servers, client_location))
}

/// Select the best server from a list, preferring the closest by distance.
///
/// # Errors
///
/// Returns [`Error::ServerNotFound`] if the server list is empty.
pub fn select_best_server(servers: &[Server]) -> Result<Server, Error> {
    if servers.is_empty() {
        return Err(Error::ServerNotFound("No servers available".to_string()));
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
        .ok_or_else(|| Error::ServerNotFound("No servers available".to_string()))?;

    Ok(best)
}

// Limit both work and elapsed time. Dropping this future cancels all probes.
const SELECTION_CANDIDATES: usize = 20;
const SELECTION_CONCURRENCY: usize = 5;

fn selection_candidates(servers: &[Server], location_known: bool) -> Vec<Server> {
    let mut unique: Vec<Server> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for server in servers {
        if seen.insert(server.url.clone()) {
            unique.push(server.clone());
        }
    }
    if location_known {
        unique.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        unique.truncate(SELECTION_CANDIDATES);
        unique
    } else if unique.len() > SELECTION_CANDIDATES {
        // Spread probes across the feed when there is no geographic hint.
        (0..SELECTION_CANDIDATES)
            .map(|i| unique[i * (unique.len() - 1) / (SELECTION_CANDIDATES - 1)].clone())
            .collect()
    } else {
        unique
    }
}

async fn select_with_probe<F, Fut>(
    servers: &[Server],
    location_known: bool,
    budget: Duration,
    probe: F,
) -> Result<Server, Error>
where
    F: Fn(Server) -> Fut,
    Fut: std::future::Future<Output = Result<f64, Error>>,
{
    if servers.is_empty() {
        return Err(Error::ServerNotFound("No servers available".into()));
    }
    let candidates = selection_candidates(servers, location_known);
    let count = candidates.len();
    let mut pending = stream::iter(candidates)
        .map(|server| {
            let future = probe(server.clone());
            async move { (server, future.await) }
        })
        .buffer_unordered(SELECTION_CONCURRENCY);
    let deadline = tokio::time::Instant::now() + budget;
    let mut best: Option<(Server, f64)> = None;
    while let Ok(Some((server, result))) = tokio::time::timeout_at(deadline, pending.next()).await {
        if let Ok(latency) = result {
            if latency.is_finite()
                && latency >= 0.0
                && best
                    .as_ref()
                    .is_none_or(|(_, previous)| latency < *previous)
            {
                best = Some((server, latency));
            }
        }
    }
    best.map(|(server, _)| server).ok_or_else(|| {
        Error::NoReachableServers(format!(
            "None of {count} candidates responded within the selection deadline"
        ))
    })
}

/// Choose the lowest-latency healthy server from a bounded shortlist.
/// Unknown location uses candidates spread across the server feed.
///
/// # Errors
/// Returns [`Error::ServerNotFound`] for an empty list and
/// [`Error::NoReachableServers`] when all probes fail or time out.
pub async fn select_reachable_server(
    client: &Client,
    servers: &[Server],
    location_known: bool,
) -> Result<Server, Error> {
    select_with_probe(
        servers,
        location_known,
        Duration::from_secs(4),
        |server| async move {
            tokio::time::timeout(Duration::from_secs(2), async {
                let mut total = 0.0;
                for _ in 0..2 {
                    let started = std::time::Instant::now();
                    let mut response = client
                        .get(ServerEndpoints::from_server_url(&server.url).latency())
                        .send()
                        .await?
                        .error_for_status()?;
                    let mut size = 0;
                    while let Some(chunk) = response.chunk().await? {
                        size += chunk.len();
                        if size > 1024 {
                            return Err(Error::NoReachableServers(
                                "Oversized latency response".into(),
                            ));
                        }
                    }
                    if size == 0 {
                        return Err(Error::NoReachableServers("Empty latency response".into()));
                    }
                    total += started.elapsed().as_secs_f64() * 1000.0;
                }
                Ok(total / 2.0)
            })
            .await
            .map_err(|_| Error::NoReachableServers("Latency probe timed out".into()))?
        },
    )
    .await
}

/// Run a ping test against the given server, returning (average latency, jitter, `packet_loss`%, `individual_samples`).
///
/// # Errors
///
/// Returns [`Error::NetworkError`] if all ping attempts fail.
pub async fn ping_test(
    client: &Client,
    server: &Server,
) -> Result<(f64, f64, f64, Vec<f64>), Error> {
    let config = TestConfig::default();
    let ping_attempts = config.ping_attempts;
    let mut latencies = Vec::new();

    // Perform multiple ping measurements
    for _ in 0..ping_attempts {
        let start = std::time::Instant::now();

        let response = client
            .get(ServerEndpoints::from_server_url(&server.url).latency())
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
        return Err(Error::NoReachableServers(
            "All ping attempts failed".to_string(),
        ));
    }

    // Safe: len() is at most PING_ATTEMPTS (8), well under 2^53.
    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    // Calculate jitter (average of absolute differences between consecutive latencies)
    let jitter = if latencies.len() > 1 {
        let mut jitter_sum = 0.0;
        for i in 1..latencies.len() {
            jitter_sum += (latencies[i] - latencies[i - 1]).abs();
        }
        // Safe: len() is at most PING_ATTEMPTS (8).
        jitter_sum / (latencies.len() - 1) as f64
    } else {
        0.0
    };

    // Calculate packet loss percentage
    // Safe: both operands are at most PING_ATTEMPTS (8), tiny values.
    let packet_loss = ((ping_attempts - latencies.len()) as f64 / ping_attempts as f64) * 100.0;

    Ok((avg, jitter, packet_loss, latencies))
}

pub async fn measure_latency_under_load(
    client: Client,
    server_url: String,
    samples: Arc<std::sync::Mutex<Vec<f64>>>,
    stop: Arc<AtomicBool>,
) {
    let config = TestConfig::default();
    let poll_interval = config.latency_poll_interval_ms;

    while !stop.load(Ordering::Acquire) {
        let start = std::time::Instant::now();
        let response = client
            .get(ServerEndpoints::from_server_url(&server_url).latency())
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                if let Ok(mut lock) = samples.lock() {
                    lock.push(elapsed);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(poll_interval)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: usize, distance: f64) -> Server {
        Server {
            id: id.to_string(),
            url: format!("http://server{id}/upload.php"),
            name: "test".into(),
            sponsor: "test".into(),
            country: "US".into(),
            lat: 0.0,
            lon: 0.0,
            distance,
        }
    }

    #[tokio::test]
    async fn selection_skips_failed_nearest_and_prefers_measured_latency() {
        let servers = vec![candidate(0, 1.0), candidate(1, 2.0), candidate(2, 3.0)];
        let chosen = select_with_probe(
            &servers,
            true,
            Duration::from_secs(1),
            |server| async move {
                match server.id.as_str() {
                    "0" => Err(Error::NoReachableServers("offline".into())),
                    "1" => Ok(50.0),
                    _ => Ok(10.0),
                }
            },
        )
        .await
        .unwrap();
        assert_eq!(chosen.id, "2");
    }

    #[tokio::test]
    async fn selection_all_failed_is_network_error_and_empty_is_config_error() {
        let result = select_with_probe(
            &[candidate(0, 1.0)],
            true,
            Duration::from_secs(1),
            |_| async { Ok(f64::NAN) },
        )
        .await
        .unwrap_err();
        assert!(matches!(result, Error::NoReachableServers(_)));
        assert_eq!(result.category(), crate::error::ErrorCategory::Network);
        let result = select_with_probe(&[], true, Duration::from_secs(1), |_| async { Ok(1.0) })
            .await
            .unwrap_err();
        assert!(matches!(result, Error::ServerNotFound(_)));
    }

    #[test]
    fn shortlist_is_bounded_deduplicated_and_spread_without_location() {
        let mut servers: Vec<_> = (0..100).map(|i| candidate(i, (100 - i) as f64)).collect();
        servers.push(servers[0].clone());
        let nearby = selection_candidates(&servers, true);
        assert_eq!(nearby.len(), 20);
        assert_eq!(nearby[0].id, "99");
        let spread = selection_candidates(&servers, false);
        assert_eq!(spread.len(), 20);
        assert_eq!(spread[0].id, "0");
        assert_eq!(spread[19].id, "99");
    }

    #[tokio::test]
    async fn selection_deadline_keeps_completed_result_and_cancels_pending_probes() {
        use std::sync::atomic::AtomicUsize;
        struct Guard(Arc<AtomicUsize>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::SeqCst);
            }
        }
        let active = Arc::new(AtomicUsize::new(0));
        let max = Arc::new(AtomicUsize::new(0));
        let servers: Vec<_> = (0..20).map(|i| candidate(i, i as f64)).collect();
        let chosen = select_with_probe(&servers, true, Duration::from_millis(50), |server| {
            let active = Arc::clone(&active);
            let max = Arc::clone(&max);
            async move {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                max.fetch_max(current, Ordering::SeqCst);
                let _guard = Guard(active);
                if server.id == "0" {
                    Ok(1.0)
                } else {
                    std::future::pending().await
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(chosen.id, "0");
        assert!(max.load(Ordering::SeqCst) <= SELECTION_CONCURRENCY);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        let failed = select_with_probe(&servers, false, Duration::from_millis(10), |_| {
            std::future::pending::<Result<f64, Error>>()
        })
        .await;
        assert!(matches!(failed, Err(Error::NoReachableServers(_))));
    }

    #[test]
    fn invalid_coordinates_are_rejected() {
        assert!(valid_coordinates(0.0, 0.0));
        for (lat, lon) in [
            (91.0, 0.0),
            (0.0, 181.0),
            (f64::NAN, 0.0),
            (0.0, f64::INFINITY),
        ] {
            assert!(!valid_coordinates(lat, lon));
        }
    }

    #[tokio::test]
    #[ignore = "requires local socket binding"]
    async fn socket_regression_discovery_and_reachable_selection() {
        use crate::services::{DefaultServerService, ServerSelector};
        use wiremock::matchers::path;
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let mock = MockServer::start().await;
        Mock::given(path("/config"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock)
            .await;
        let xml = format!(
            r#"<settings><servers><server id="1" url="{0}/bad/upload.php" name="bad" sponsor="test" country="US" lat="0" lon="0"/><server id="2" url="{0}/good/upload.php" name="good" sponsor="test" country="US" lat="50" lon="50"/></servers></settings>"#,
            mock.uri()
        );
        Mock::given(path("/servers"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml))
            .mount(&mock)
            .await;
        Mock::given(path("/bad/latency.txt"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock)
            .await;
        Mock::given(path("/good/latency.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_string("test=test"))
            .expect(2)
            .mount(&mock)
            .await;
        let client = Client::new();
        let (servers, location) = fetch_from_urls(
            &client,
            &format!("{}/config", mock.uri()),
            &format!("{}/servers", mock.uri()),
        )
        .await
        .unwrap();
        assert!(location.is_none());
        assert!(
            servers
                .iter()
                .all(|server| server.distance == f64::INFINITY)
        );
        let service = DefaultServerService::new(client.clone());
        let selected = service.select_reachable(&servers, false).await.unwrap();
        assert_eq!(selected.id, "2");
        let result = service.select_reachable(&servers[..1], false).await;
        assert!(matches!(result, Err(Error::NoReachableServers(_))));
        Mock::given(path("/invalid-config"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"<settings><client lat="100" lon="0"/></settings>"#),
            )
            .mount(&mock)
            .await;
        assert!(
            fetch_client_location_from_url(&client, &format!("{}/invalid-config", mock.uri()))
                .await
                .is_err()
        );
        Mock::given(path("/failed-servers"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock)
            .await;
        assert!(matches!(
            fetch_from_urls(
                &client,
                &format!("{}/config", mock.uri()),
                &format!("{}/failed-servers", mock.uri())
            )
            .await,
            Err(Error::NetworkError(_))
        ));
    }

    #[tokio::test]
    #[ignore = "requires local socket binding"]
    async fn socket_regression_selection_rejects_empty_oversized_and_stalled_responses() {
        use wiremock::matchers::path;
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let mock = MockServer::start().await;
        for (name, body) in [("empty", String::new()), ("oversized", "x".repeat(1025))] {
            Mock::given(path(format!("/{name}/latency.txt")))
                .respond_with(ResponseTemplate::new(200).set_body_string(body))
                .mount(&mock)
                .await;
        }
        Mock::given(path("/stalled/latency.txt"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("test=test")
                    .set_delay(Duration::from_secs(3)),
            )
            .mount(&mock)
            .await;
        let mut servers: Vec<_> = (0..3).map(|i| candidate(i, i as f64)).collect();
        for (server, name) in servers.iter_mut().zip(["empty", "oversized", "stalled"]) {
            server.url = format!("{}/{name}/upload.php", mock.uri());
        }
        let error = select_reachable_server(&Client::new(), &servers, true)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::NoReachableServers(_)));
        Mock::given(path("/valid-config"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"<settings><client lat="0" lon="0"/></settings>"#),
            )
            .mount(&mock)
            .await;
        Mock::given(path("/valid-servers")).respond_with(ResponseTemplate::new(200).set_body_string(format!(r#"<settings><servers><server id="1" url="{}/upload.php" name="test" sponsor="test" country="US" lat="0" lon="1"/></servers></settings>"#, mock.uri()))).mount(&mock).await;
        let (servers, location) = fetch_from_urls(
            &Client::new(),
            &format!("{}/valid-config", mock.uri()),
            &format!("{}/valid-servers", mock.uri()),
        )
        .await
        .unwrap();
        assert!(location.is_some());
        assert!((servers[0].distance - 111.195).abs() < 0.01);
    }

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
        assert!((best.distance - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_select_best_server_empty() {
        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ServerNotFound(_)));
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
        assert!((avg - 17.5).abs() < f64::EPSILON);
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
