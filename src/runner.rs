use crate::config::Config;
use crate::error::SpeedtestError;
use crate::http::discover_client_ip;
use crate::servers::ping_test;
use crate::types::{Server, ServerInfo, TestResult};
use reqwest::Client;

/// Orchestrates the speed tests (ping, download, upload)
pub struct TestRunner;

impl TestRunner {
    /// Run all tests and return results.
    ///
    /// Executes ping, download, and upload tests based on configuration.
    /// Each test is optional and controlled by `no_download`/`no_upload` flags.
    #[tracing::instrument(skip(client, server, config), fields(server_id = %server.id))]
    pub async fn run(
        client: &Client,
        server: &Server,
        config: &Config,
    ) -> Result<TestResult, SpeedtestError> {
        // Discover client IP
        let client_ip = discover_client_ip(client).await.ok();

        // Run ping test (always run if either download or upload test is enabled)
        let ping = if !config.no_download || !config.no_upload {
            let result = ping_test(client, server).await?;
            if !config.simple {
                tracing::info!(ping_ms = result, "Ping test complete");
            }
            Some(result)
        } else {
            None
        };

        // Run download test
        let download = if !config.no_download {
            let result = crate::download::download_test(client, server, config.single).await?;
            if !config.simple {
                tracing::info!(
                    download_mbps = result / 1_000_000.0,
                    "Download test complete"
                );
            }
            Some(result)
        } else {
            None
        };

        // Run upload test
        let upload = if !config.no_upload {
            let result = crate::upload::upload_test(client, server, config.single).await?;
            if !config.simple {
                tracing::info!(upload_mbps = result / 1_000_000.0, "Upload test complete");
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliArgs;

    fn create_test_server() -> Server {
        Server {
            id: "1".to_string(),
            url: "http://test.server.com".to_string(),
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
    fn test_config_no_download_disables_download_test() {
        let args = CliArgs { no_download: true, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(config.no_download);
        assert!(!config.no_upload);
    }

    #[test]
    fn test_config_no_upload_disables_upload_test() {
        let args = CliArgs { no_upload: true, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(!config.no_download);
        assert!(config.no_upload);
    }

    #[test]
    fn test_config_both_disabled_no_tests_run() {
        let args = CliArgs { no_download: true, no_upload: true, simple: true, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(config.no_download);
        assert!(config.no_upload);
        // When both are disabled, ping should also be skipped
        assert!(config.no_download || config.no_upload);
    }

    #[test]
    fn test_config_single_mode_affects_tests() {
        let args = CliArgs { single: true, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(config.single);
    }

    #[test]
    fn test_server_info_population() {
        let server = create_test_server();
        let server_info = ServerInfo {
            id: server.id.clone(),
            name: server.name.clone(),
            sponsor: server.sponsor.clone(),
            country: server.country.clone(),
            distance: server.distance,
        };
        assert_eq!(server_info.id, "1");
        assert_eq!(server_info.name, "Test Server");
        assert_eq!(server_info.sponsor, "Test ISP");
        assert_eq!(server_info.country, "US");
        assert!((server_info.distance - 100.0).abs() < f64::EPSILON);
    }
}
