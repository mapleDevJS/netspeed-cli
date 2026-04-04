use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub id: String,
    pub url: String,
    pub name: String,
    pub sponsor: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub distance: f64,
    pub latency: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub server: ServerInfo,
    pub ping: Option<f64>,
    pub download: Option<f64>,
    pub upload: Option<f64>,
    pub share_url: Option<String>,
    pub timestamp: String,
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub sponsor: String,
    pub country: String,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvOutput {
    pub server_id: String,
    pub sponsor: String,
    pub server_name: String,
    pub timestamp: String,
    pub distance: f64,
    pub ping: f64,
    pub download: f64,
    pub upload: f64,
    pub share: String,
    pub ip_address: String,
}
