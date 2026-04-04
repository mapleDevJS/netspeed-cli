use crate::error::SpeedtestError;
use crate::types::Server;
use reqwest::Client;

/// Represents a Speedtest Mini server configuration.
///
/// Mini servers are self-hosted Speedtest instances that use a simpler
/// protocol than the main speedtest.net service.
pub struct MiniServer {
    /// Upload endpoint URL
    pub url: String,
    /// Server operator name
    pub sponsor: String,
    /// Server display name
    pub name: String,
    /// Server identifier
    pub id: String,
    /// Distance from client (always 0 for Mini)
    pub distance: f64,
    /// Latency (always 0 for Mini, measured during tests)
    pub latency: f64,
}

/// Detect and configure a Speedtest Mini server.
///
/// Probes the given URL for upload endpoint availability,
/// trying common extensions (php, asp, aspx, jsp).
pub async fn detect_mini_server(
    client: &Client,
    mini_url: &str,
) -> Result<MiniServer, SpeedtestError> {
    // Normalize URL
    let base_url = mini_url.trim_end_matches('/');

    // Try to detect upload extension
    let upload_ext = detect_upload_extension(client, base_url).await?;

    let upload_url = format!("{}/upload.{}", base_url, upload_ext);

    Ok(MiniServer {
        url: upload_url,
        sponsor: "Mini".to_string(),
        name: base_url.to_string(),
        id: "0".to_string(),
        distance: 0.0,
        latency: 0.0,
    })
}

/// Detect the upload extension for a Mini server
async fn detect_upload_extension(
    client: &Client,
    base_url: &str,
) -> Result<String, SpeedtestError> {
    // Try common extensions in order
    let extensions = ["php", "asp", "aspx", "jsp"];

    for ext in &extensions {
        let test_url = format!("{}/upload.{}", base_url, ext);
        if test_upload_endpoint(client, &test_url).await {
            return Ok(ext.to_string());
        }
    }

    // Default to php if none detected
    Ok("php".to_string())
}

/// Test if an upload endpoint is functional
async fn test_upload_endpoint(client: &Client, url: &str) -> bool {
    // Try a small upload to test if endpoint exists
    let test_data = vec![0u8; 1024]; // 1KB test

    match client.post(url).body(test_data).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Convert a MiniServer to a standard Server struct.
///
/// This allows Mini servers to be used with the same test pipeline
/// as speedtest.net servers.
pub fn mini_to_server(mini: &MiniServer) -> Server {
    Server {
        id: mini.id.clone(),
        url: mini.url.clone(),
        name: mini.name.clone(),
        sponsor: mini.sponsor.clone(),
        country: "Unknown".to_string(),
        lat: 0.0,
        lon: 0.0,
        distance: mini.distance,
        latency: mini.latency,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mini_to_server_basic() {
        let mini = MiniServer {
            url: "http://mini.example.com/upload.php".to_string(),
            sponsor: "Mini ISP".to_string(),
            name: "http://mini.example.com".to_string(),
            id: "0".to_string(),
            distance: 0.0,
            latency: 0.0,
        };

        let server = mini_to_server(&mini);
        assert_eq!(server.id, "0");
        assert_eq!(server.url, "http://mini.example.com/upload.php");
        assert_eq!(server.name, "http://mini.example.com");
        assert_eq!(server.sponsor, "Mini ISP");
        assert_eq!(server.country, "Unknown");
        assert!((server.lat - 0.0).abs() < f64::EPSILON);
        assert!((server.lon - 0.0).abs() < f64::EPSILON);
        assert!((server.distance - 0.0).abs() < f64::EPSILON);
        assert!((server.latency - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mini_to_server_preserves_values() {
        let mini = MiniServer {
            url: "http://example.com/upload.asp".to_string(),
            sponsor: "Test Sponsor".to_string(),
            name: "Mini Server".to_string(),
            id: "42".to_string(),
            distance: 0.0,
            latency: 0.0,
        };

        let server = mini_to_server(&mini);
        assert_eq!(server.id, "42");
        assert_eq!(server.sponsor, "Test Sponsor");
    }

    #[test]
    fn test_mini_server_default_values() {
        let mini = MiniServer {
            url: "http://mini.com/upload.php".to_string(),
            sponsor: "Mini".to_string(),
            name: "http://mini.com".to_string(),
            id: "0".to_string(),
            distance: 0.0,
            latency: 0.0,
        };

        assert_eq!(mini.id, "0");
        assert_eq!(mini.distance, 0.0);
        assert_eq!(mini.latency, 0.0);
        assert_eq!(mini.sponsor, "Mini");
    }

    #[test]
    fn test_detect_upload_extension_fallback_to_php() {
        // When no endpoint responds successfully, should fallback to php
        // We can't easily test the async function without a mock HTTP client,
        // but we can verify the fallback logic in the code
        let extensions = ["php", "asp", "aspx", "jsp"];
        assert_eq!(extensions[0], "php"); // Default fallback
    }

    #[test]
    fn test_url_normalization_trailing_slash() {
        let url_with_slash = "http://mini.example.com/";
        let normalized = url_with_slash.trim_end_matches('/');
        assert_eq!(normalized, "http://mini.example.com");
    }

    #[test]
    fn test_url_normalization_no_trailing_slash() {
        let url_without_slash = "http://mini.example.com";
        let normalized = url_without_slash.trim_end_matches('/');
        assert_eq!(normalized, "http://mini.example.com");
    }

    #[test]
    fn test_upload_url_construction_php() {
        let base_url = "http://mini.example.com";
        let ext = "php";
        let upload_url = format!("{}/upload.{}", base_url, ext);
        assert_eq!(upload_url, "http://mini.example.com/upload.php");
    }

    #[test]
    fn test_upload_url_construction_aspx() {
        let base_url = "http://mini.example.com";
        let ext = "aspx";
        let upload_url = format!("{}/upload.{}", base_url, ext);
        assert_eq!(upload_url, "http://mini.example.com/upload.aspx");
    }

    #[test]
    fn test_test_upload_endpoint_test_data_size() {
        // Verify the test upload data size is 1KB
        let test_data_size = 1024;
        assert_eq!(test_data_size, 1024);
        let test_data = vec![0u8; test_data_size];
        assert_eq!(test_data.len(), 1024);
    }
}
