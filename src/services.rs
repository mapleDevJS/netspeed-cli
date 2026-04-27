//! Service abstractions for dependency injection.
//!
//! All traits use `async_trait` for dyn-compatibility,
//! enabling `Arc<dyn Trait>` in `ServiceContainer`.
//!
//! Services own their HTTP client internally — callers never
//! pass `reqwest::Client` to service methods (full DIP).

use crate::error::Error;
use crate::types::{ClientLocation, Server};
use async_trait::async_trait;

// ============================================================================
// Server service traits (no client parameter — services own their client)
// ============================================================================

#[async_trait]
pub trait ServerFetcher: Send + Sync {
    async fn fetch_servers(&self) -> Result<(Vec<Server>, Option<ClientLocation>), Error>;
}

#[async_trait]
pub trait ServerPinger: Send + Sync {
    async fn ping_server(&self, server: &Server) -> Result<(f64, f64, f64, Vec<f64>), Error>;
}

pub trait ServerSelector: Send + Sync {
    fn select_best(&self, servers: &[Server]) -> Result<Server, Error>;
}

pub trait ServerService: ServerFetcher + ServerPinger + ServerSelector + Send + Sync {}

// ============================================================================
// IP discovery trait
// ============================================================================

#[async_trait]
pub trait IpDiscoverer: Send + Sync {
    async fn discover_ip(&self) -> Result<String, Error>;
}

// ============================================================================
// Latency monitor trait
// ============================================================================

#[async_trait]
pub trait LatencyMonitor: Send + Sync {
    async fn measure_latency_under_load(
        &self,
        server_url: String,
        samples: std::sync::Arc<std::sync::Mutex<Vec<f64>>>,
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    );
}

// ============================================================================
// Default implementations (own reqwest::Client internally)
// ============================================================================

#[derive(Clone, Debug)]
pub struct DefaultServerService {
    client: reqwest::Client,
}

impl DefaultServerService {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ServerFetcher for DefaultServerService {
    async fn fetch_servers(&self) -> Result<(Vec<Server>, Option<ClientLocation>), Error> {
        crate::servers::fetch(&self.client).await
    }
}

#[async_trait]
impl ServerPinger for DefaultServerService {
    async fn ping_server(&self, server: &Server) -> Result<(f64, f64, f64, Vec<f64>), Error> {
        crate::servers::ping_test(&self.client, server).await
    }
}

impl ServerSelector for DefaultServerService {
    fn select_best(&self, servers: &[Server]) -> Result<Server, Error> {
        crate::servers::select_best_server(servers)
    }
}

impl ServerService for DefaultServerService {}

#[derive(Clone, Debug)]
pub struct DefaultIpService {
    client: reqwest::Client,
}

impl DefaultIpService {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IpDiscoverer for DefaultIpService {
    async fn discover_ip(&self) -> Result<String, Error> {
        crate::http::discover_client_ip(&self.client).await
    }
}

#[derive(Clone, Debug)]
pub struct DefaultLatencyMonitor {
    client: reqwest::Client,
}

impl DefaultLatencyMonitor {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl LatencyMonitor for DefaultLatencyMonitor {
    async fn measure_latency_under_load(
        &self,
        server_url: String,
        samples: std::sync::Arc<std::sync::Mutex<Vec<f64>>>,
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        crate::servers::measure_latency_under_load(self.client.clone(), server_url, samples, stop)
            .await;
    }
}

// ============================================================================
// Services trait + ServiceContainer
// ============================================================================

/// Trait for accessing all services — enables mocking PhaseContext in tests.
pub trait Services: Send + Sync {
    fn server_service(&self) -> &dyn ServerService;
    fn ip_service(&self) -> &dyn IpDiscoverer;
}

/// Service container holding all injectable services.
/// Uses `Arc<dyn Trait>` for true dependency injection.
#[derive(Clone)]
pub struct ServiceContainer {
    server: std::sync::Arc<dyn ServerService>,
    ip: std::sync::Arc<dyn IpDiscoverer>,
}

impl Services for ServiceContainer {
    fn server_service(&self) -> &dyn ServerService {
        self.server.as_ref()
    }

    fn ip_service(&self) -> &dyn IpDiscoverer {
        self.ip.as_ref()
    }
}

impl std::fmt::Debug for ServiceContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceContainer")
            .field("server", &"dyn ServerService")
            .field("ip", &"dyn IpDiscoverer")
            .finish()
    }
}

impl ServiceContainer {
    /// Create a container with default services using the given HTTP client.
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            server: std::sync::Arc::new(DefaultServerService::new(client.clone())),
            ip: std::sync::Arc::new(DefaultIpService::new(client)),
        }
    }

    pub fn with_server(mut self, server: impl ServerService + 'static) -> Self {
        self.server = std::sync::Arc::new(server);
        self
    }

    pub fn with_ip(mut self, ip: impl IpDiscoverer + 'static) -> Self {
        self.ip = std::sync::Arc::new(ip);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[test]
    fn test_default_service_container_creation() {
        let container = ServiceContainer::new(make_client());
        let _server = container.server_service();
        let _ip = container.ip_service();
    }

    #[test]
    fn test_service_container_with_custom() {
        let client = make_client();
        let container = ServiceContainer::new(client)
            .with_server(DefaultServerService::new(make_client()))
            .with_ip(DefaultIpService::new(make_client()));
        let _server = container.server_service();
        let _ip = container.ip_service();
    }
}
