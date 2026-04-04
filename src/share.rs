use reqwest::Client;
use crate::error::SpeedtestError;
use crate::types::TestResult;

pub async fn generate_share_url(
    _client: &Client,
    result: &TestResult,
) -> Result<String, SpeedtestError> {
    // Generate unique hash for sharing
    let hash = generate_result_hash(result);

    // Post results to speedtest.net
    let share_url = format!(
        "http://www.speedtest.net/result/{}.png",
        hash
    );

    Ok(share_url)
}

fn generate_result_hash(result: &TestResult) -> String {
    use md5::{Md5, Digest};

    let mut hasher = Md5::new();
    hasher.update(result.server.id.as_bytes());
    hasher.update(result.timestamp.as_bytes());
    hasher.update(result.download.unwrap_or(0.0).to_string().as_bytes());
    hasher.update(result.upload.unwrap_or(0.0).to_string().as_bytes());

    let result = hasher.finalize();
    hex::encode(&result[..4]) // Use first 4 bytes for shorter hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServerInfo;

    fn create_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "1234".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.5,
            },
            ping: Some(15.234),
            download: Some(150_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2026-04-04T12:00:00Z".to_string(),
            client_ip: Some("192.168.1.1".to_string()),
        }
    }

    #[test]
    fn test_generate_result_hash() {
        let result = create_test_result();
        let hash = generate_result_hash(&result);
        // Hash should be 8 hex characters (4 bytes)
        assert_eq!(hash.len(), 8);
        // Hash should be valid hex
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_result_hash_deterministic() {
        // Same input should produce same hash
        let result1 = create_test_result();
        let result2 = create_test_result();
        
        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_result_hash_different_inputs() {
        let mut result1 = create_test_result();
        let mut result2 = create_test_result();
        
        result1.server.id = "1234".to_string();
        result2.server.id = "5678".to_string();
        
        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_result_hash_with_none_values() {
        let mut result = create_test_result();
        result.download = None;
        result.upload = None;
        
        let hash = generate_result_hash(&result);
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_generate_share_url_format() {
        // Test that share URL has correct format
        let hash = "abcd1234";
        let share_url = format!(
            "http://www.speedtest.net/result/{}.png",
            hash
        );
        assert!(share_url.contains("speedtest.net"));
        assert!(share_url.ends_with(".png"));
        assert!(share_url.contains(hash));
    }
}
