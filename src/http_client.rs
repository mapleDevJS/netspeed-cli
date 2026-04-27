//! Abstraction over HTTP client used by phases.

use crate::error::Error;
use async_trait::async_trait;
use reqwest::Response;

/// Minimal HTTP client interface required by the test phases.
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<Response, Error>;
    async fn post(
        &self,
        url: &str,
        body: impl Into<reqwest::Body> + Send,
    ) -> Result<Response, Error>;
}

/// Production implementation that forwards to `reqwest::Client`.
pub struct ReqwestClient(pub reqwest::Client);

impl ReqwestClient {
    // Convenience wrapper used by legacy tests
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, crate::error::Error> {
        self.0
            .get(url)
            .send()
            .await
            .map_err(crate::error::Error::from)
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<Response, Error> {
        self.0.get(url).send().await.map_err(Error::NetworkError)
    }
    async fn post(
        &self,
        url: &str,
        body: impl Into<reqwest::Body> + Send,
    ) -> Result<Response, Error> {
        self.0
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(Error::NetworkError)
    }
}
