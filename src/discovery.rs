use crate::config::Config;
use crate::error::SpeedtestError;
use crate::mini::{detect_mini_server, mini_to_server};
use crate::servers::{calculate_distances, fetch_client_config, fetch_servers, select_best_server};
use crate::types::Server;
use reqwest::Client;

/// Discovers the test server to use, either from Mini server config or speedtest.net server list
pub struct ServerDiscovery;

impl ServerDiscovery {
    /// Discover server based on configuration
    #[tracing::instrument(skip(client, config), fields(server_id))]
    pub async fn discover(client: &Client, config: &Config) -> Result<Server, SpeedtestError> {
        if let Some(ref mini_url) = config.mini_url {
            Self::discover_mini(client, mini_url).await
        } else {
            Self::discover_speedtest(client, config).await
        }
    }

    /// Handle --list option and return early if needed
    pub async fn handle_list(client: &Client, config: &Config) -> Result<bool, SpeedtestError> {
        if config.list {
            let servers = fetch_servers(client, config).await?;
            crate::formatter::format_list(&servers)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Discover Mini server
    async fn discover_mini(client: &Client, mini_url: &str) -> Result<Server, SpeedtestError> {
        let mini_server = detect_mini_server(client, mini_url).await?;
        tracing::info!(name = %mini_server.name, "Mini server detected");
        Ok(mini_to_server(&mini_server))
    }

    /// Discover speedtest.net server
    async fn discover_speedtest(
        client: &Client,
        config: &Config,
    ) -> Result<Server, SpeedtestError> {
        let mut servers = fetch_servers(client, config).await?;

        if servers.is_empty() {
            return Err(SpeedtestError::ServerNotFound(
                "No servers available for testing".to_string(),
            ));
        }

        // Get client location and calculate distances
        if let Ok(client_config) = fetch_client_config(client).await {
            if let Some(client_info) = client_config.client_info {
                if let (Ok(lat), Ok(lon)) = (
                    client_info.lat.parse::<f64>(),
                    client_info.lon.parse::<f64>(),
                ) {
                    calculate_distances(&mut servers, lat, lon);
                }
            }
        }

        // Apply filters
        if !config.server_ids.is_empty() {
            servers.retain(|s| config.server_ids.contains(&s.id));
        }
        if !config.exclude_ids.is_empty() {
            servers.retain(|s| !config.exclude_ids.contains(&s.id));
        }

        if servers.is_empty() {
            return Err(SpeedtestError::ServerNotFound(
                "No servers available for testing".to_string(),
            ));
        }

        select_best_server(&servers)
    }
}
