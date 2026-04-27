//! Speedtest server endpoint derivation.
//!
//! The speedtest.net XML feed commonly publishes server URLs ending in
//! `upload.php`. Runtime endpoints such as `latency.txt` and `random*.jpg`
//! live in the same directory. This module centralizes that URL knowledge.

/// Canonical endpoint set derived from a speedtest server URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEndpoints {
    raw: String,
    base: String,
    upload: String,
}

impl ServerEndpoints {
    /// Derive endpoints from a speedtest server URL.
    #[must_use]
    pub fn from_server_url(url: &str) -> Self {
        let trimmed = url.trim_end_matches('/');
        let base = trimmed
            .strip_suffix("/upload.php")
            .or_else(|| trimmed.strip_suffix("/upload"))
            .unwrap_or(trimmed)
            .to_string();
        let upload = if trimmed.ends_with("/upload.php") || trimmed.ends_with("/upload") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/upload.php")
        };

        Self {
            raw: trimmed.to_string(),
            base,
            upload,
        }
    }

    /// Original normalized server URL.
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Base directory containing the speedtest assets.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Upload endpoint.
    #[must_use]
    pub fn upload(&self) -> &str {
        &self.upload
    }

    /// Latency probe endpoint.
    #[must_use]
    pub fn latency(&self) -> String {
        format!("{}/latency.txt", self.base)
    }

    /// Download asset endpoint for a specific test asset name.
    #[must_use]
    pub fn download_asset(&self, asset_name: &str) -> String {
        format!("{}/{}", self.base, asset_name.trim_start_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::ServerEndpoints;

    #[test]
    fn test_from_upload_php_url() {
        let endpoints = ServerEndpoints::from_server_url("http://example.com/speedtest/upload.php");
        assert_eq!(endpoints.base(), "http://example.com/speedtest");
        assert_eq!(
            endpoints.upload(),
            "http://example.com/speedtest/upload.php"
        );
        assert_eq!(
            endpoints.latency(),
            "http://example.com/speedtest/latency.txt"
        );
    }

    #[test]
    fn test_from_base_url() {
        let endpoints = ServerEndpoints::from_server_url("http://example.com/speedtest");
        assert_eq!(endpoints.base(), "http://example.com/speedtest");
        assert_eq!(
            endpoints.upload(),
            "http://example.com/speedtest/upload.php"
        );
    }

    #[test]
    fn test_download_asset() {
        let endpoints = ServerEndpoints::from_server_url("https://cdn.example.net/upload.php");
        assert_eq!(
            endpoints.download_asset("random3500x3500.jpg"),
            "https://cdn.example.net/random3500x3500.jpg"
        );
    }
}
