use reqwest::Client;
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::http::discover_client_ip;
use crate::servers::ping_test;
use crate::types::{Server, ServerInfo, TestResult};

/// Orchestrates the speed tests (ping, download, upload)
pub struct TestRunner;

impl TestRunner {
    /// Run all tests and return results
    pub async fn run(
        client: &Client,
        server: &Server,
        config: &Config,
    ) -> Result<TestResult, SpeedtestError> {
        // Discover client IP
        let client_ip = discover_client_ip(client).await.ok();

        // Run ping test
        let ping = if !config.no_download || !config.no_upload {
            let result = ping_test(client, server).await?;
            eprintln!("Ping: {:.3} ms", result);
            Some(result)
        } else {
            None
        };

        // Run download test
        let download = if !config.no_download {
            let result = crate::download::download_test(client, server, config.single).await?;
            eprintln!("Download: {:.2} Mbit/s", result / 1_000_000.0);
            Some(result)
        } else {
            None
        };

        // Run upload test
        let upload = if !config.no_upload {
            let result = crate::upload::upload_test(client, server, config.single).await?;
            eprintln!("Upload: {:.2} Mbit/s", result / 1_000_000.0);
            Some(result)
        } else {
            None
        };

        Ok(TestResult {
            server: ServerInfo {
                id: server.id.clone(),
                name: server.name.clone(),
                sponsor: server.sponsor.clone(),
                country: server.country.clone(),
                distance: server.distance,
            },
            ping,
            download,
            upload,
            share_url: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            client_ip,
        })
    }
}
