//! Server discovery and client location detection.
//!
//! Handles HTTP I/O and XML parsing for:
//! - Fetching the speedtest.net server list
//! - Determining client location from config API

use crate::error::SpeedtestError;
use crate::geo::calculate_distance;
use crate::types::Server;
use quick_xml::de::from_str;
use reqwest::Client;
use serde::Deserialize;

/// Root element for the Speedtest.net servers XML response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "settings")]
struct ServerConfig {
    #[serde(rename = "servers")]
    servers_wrapper: ServersWrapper,
}

/// Wrapper for the list of servers
#[derive(Debug, Clone, Deserialize)]
struct ServersWrapper {
    #[serde(rename = "server", default)]
    servers: Vec<Server>,
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

const SPEEDTEST_SERVERS_URL: &str = "https://www.speedtest.net/speedtest-servers-static.php";
const SPEEDTEST_CONFIG_URL: &str = "https://www.speedtest.net/api/ios-config.php";

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

    servers.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::*;

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
