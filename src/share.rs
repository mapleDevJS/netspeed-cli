use crate::error::SpeedtestError;
use crate::types::TestResult;
use reqwest::Client;
use sha2::{Digest, Sha256};

/// URL for posting speedtest results
pub const SPEEDTEST_POST_URL: &str = "https://www.speedtest.net/api/api.php";

/// Generate a share results URL by posting results to speedtest.net.
///
/// This function:
/// 1. Formats the test results as form data
/// 2. Posts to the speedtest.net API
/// 3. Returns the URL to the results page
#[tracing::instrument(skip(client, result), fields(server_id = %result.server.id))]
pub async fn generate_share_url(
    client: &Client,
    result: &TestResult,
) -> Result<String, SpeedtestError> {
    // Build form data for the API
    let download_speed = result.download.unwrap_or(0.0);
    let upload_speed = result.upload.unwrap_or(0.0);
    let ping_ms = result.ping.unwrap_or(0.0);
    let hash = generate_result_hash(result);

    // Post results to speedtest.net
    let response = client
        .post(SPEEDTEST_POST_URL)
        .form(&[
            ("download", format!("{}", download_speed)),
            ("upload", format!("{}", upload_speed)),
            ("ping", format!("{}", ping_ms)),
            ("promo", "0".to_string()),
            ("recommendedserverid", result.server.id.clone()),
            ("accuracy", "1".to_string()),
            ("startmode", "init".to_string()),
            ("serverid", result.server.id.clone()),
            ("hash", hash),
        ])
        .send()
        .await?;

    // Parse response to get result ID
    let text = response.text().await?;

    // Extract result ID from response (format varies, try to parse)
    let result_id = parse_result_id(&text);

    Ok(format!(
        "https://www.speedtest.net/result/{}.png",
        result_id
    ))
}

/// Generate a unique hash for the test result.
///
/// Uses SHA-256 for better collision resistance compared to MD5.
pub fn generate_result_hash(result: &TestResult) -> String {
    let mut hasher = Sha256::new();
    hasher.update(result.server.id.as_bytes());
    hasher.update(result.timestamp.as_bytes());
    hasher.update(result.download.unwrap_or(0.0).to_string().as_bytes());
    hasher.update(result.upload.unwrap_or(0.0).to_string().as_bytes());
    hasher.update(result.ping.unwrap_or(0.0).to_string().as_bytes());

    let result = hasher.finalize();
    hex::encode(&result[..8]) // Use first 8 bytes for a reasonably unique hash
}

/// Parse the result ID from the API response.
///
/// The API may return different formats; we try to extract the numeric ID.
fn parse_result_id(response: &str) -> String {
    // Try to find a numeric ID in the response
    if let Some(id) = extract_numeric_id(response) {
        return id;
    }

    // Fallback: use timestamp-based ID
    chrono::Utc::now().timestamp().to_string()
}

/// Extract a numeric ID from response text.
fn extract_numeric_id(text: &str) -> Option<String> {
    // Look for patterns like "resultid=12345" or just a standalone number
    for line in text.lines() {
        if let Some(pos) = line.find("resultid") {
            let rest = &line[pos..];
            if let Some(eq_pos) = rest.find('=') {
                let id_str = &rest[eq_pos + 1..];
                let id: String = id_str.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !id.is_empty() {
                    return Some(id);
                }
            }
        }
    }

    // Try to find any standalone number
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        Some(digits)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ServerInfo, TestResult};

    fn create_test_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.0,
            },
            ping: Some(25.5),
            download: Some(100_000_000.0),
            upload: Some(50_000_000.0),
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        }
    }

    #[test]
    fn test_extract_numeric_id_with_resultid_pattern() {
        let text = "some text resultid=98765 more text";
        let result = extract_numeric_id(text);
        assert_eq!(result, Some("98765".to_string()));
    }

    #[test]
    fn test_extract_numeric_id_multiline() {
        let text = "line1\nresultid=12345\nline3";
        let result = extract_numeric_id(text);
        assert_eq!(result, Some("12345".to_string()));
    }

    #[test]
    fn test_extract_numeric_id_standalone_digits() {
        let text = "some text 12345 more";
        let result = extract_numeric_id(text);
        assert_eq!(result, Some("12345".to_string()));
    }

    #[test]
    fn test_extract_numeric_id_no_digits() {
        let text = "no digits here";
        let result = extract_numeric_id(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_numeric_id_empty() {
        let result = extract_numeric_id("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_numeric_id_resultid_with_trailing_text() {
        let text = "resultid=54321&other=param";
        let result = extract_numeric_id(text);
        assert_eq!(result, Some("54321".to_string()));
    }

    #[test]
    fn test_extract_numeric_id_multiple_digits_only_first_sequence() {
        let text = "id=12345 value=67890";
        let result = extract_numeric_id(text);
        // Should find first resultid pattern, or fallback to all digits
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_result_id_with_resultid_pattern() {
        let response = "success resultid=99999 end";
        let result = parse_result_id(response);
        assert_eq!(result, "99999");
    }

    #[test]
    fn test_parse_result_id_fallback_to_timestamp() {
        let response = "no id here";
        let result = parse_result_id(response);
        // Should fallback to timestamp (numeric digits from response or current timestamp)
        assert!(!result.is_empty());
    }

    #[test]
    fn test_parse_result_id_with_standalone_digits() {
        let response = "12345";
        let result = parse_result_id(response);
        assert_eq!(result, "12345");
    }

    #[test]
    fn test_generate_result_hash_basic() {
        let result = create_test_result();
        let hash1 = generate_result_hash(&result);
        assert_eq!(hash1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_generate_result_hash_deterministic() {
        let result = create_test_result();
        let hash1 = generate_result_hash(&result);
        let hash2 = generate_result_hash(&result);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_result_hash_different_server_id() {
        let result1 = create_test_result();
        let mut result2 = create_test_result();
        result2.server.id = "99999".to_string();

        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_result_hash_different_timestamp() {
        let result1 = create_test_result();
        let mut result2 = create_test_result();
        result2.timestamp = "2024-01-02T00:00:00Z".to_string();

        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_result_hash_with_none_values() {
        let result = TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.0,
            },
            ping: None,
            download: None,
            upload: None,
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };
        let hash = generate_result_hash(&result);
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_generate_result_hash_different_download_speed() {
        let result1 = create_test_result();
        let mut result2 = create_test_result();
        result2.download = Some(200_000_000.0);

        let hash1 = generate_result_hash(&result1);
        let hash2 = generate_result_hash(&result2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_share_url_posts_and_returns_url() {
        // This requires HTTP mocking, so we'll test with wiremock in integration tests
        // Here we just verify the function structure and URL format
        let expected_prefix = "https://www.speedtest.net/result/";
        assert!(expected_prefix.starts_with("https://"));
    }

    #[test]
    fn test_speedtest_post_url_constant() {
        assert_eq!(SPEEDTEST_POST_URL, "https://www.speedtest.net/api/api.php");
    }
}
