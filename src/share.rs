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
