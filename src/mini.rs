use reqwest::Client;
use crate::error::SpeedtestError;
use crate::types::Server;

/// Represents a Speedtest Mini server configuration
pub struct MiniServer {
    pub url: String,
    pub sponsor: String,
    pub name: String,
    pub id: String,
    pub distance: f64,
    pub latency: f64,
}

/// Detect and configure a Speedtest Mini server
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

/// Create a Server struct from a MiniServer for use with existing tests
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
