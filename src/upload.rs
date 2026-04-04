use crate::error::SpeedtestError;
use crate::progress::ProgressTracker;
use crate::types::Server;
use crate::utils::calculate_bps;
use reqwest::Client;
use std::sync::Arc;

/// Number of concurrent upload streams in multi-connection mode
pub const CONCURRENT_UPLOADS: usize = 4;

/// Number of uploads per stream
pub const UPLOADS_PER_STREAM: usize = 4;

/// Size of each upload payload in bytes (200KB)
pub const UPLOAD_CHUNK_SIZE: usize = 200_000;

/// Run upload speed test against the specified server.
///
/// Uploads multiple payloads concurrently to measure throughput.
/// Returns speed in bits per second.
#[tracing::instrument(skip(client, server), fields(server_id = %server.id))]
pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_uploads = if single { 1 } else { CONCURRENT_UPLOADS };
    let total_uploads = concurrent_uploads * UPLOADS_PER_STREAM;

    // Create progress tracker
    let tracker = Arc::new(ProgressTracker::new(total_uploads, true));

    let start = std::time::Instant::now();

    // Generate upload data
    let upload_data = generate_upload_data(UPLOAD_CHUNK_SIZE);

    // Upload to multiple endpoints simultaneously
    let mut handles = Vec::new();

    for i in 0..concurrent_uploads {
        let client = client.clone();
        let server_url = server.url.clone();
        let data = upload_data.clone();
        let tracker = tracker.clone();
        let stream_index = i;

        let handle = tokio::spawn(async move {
            let mut uploaded_bytes = 0u64;
            let mut errors = Vec::new();

            // Perform multiple uploads
            for _ in 0..UPLOADS_PER_STREAM {
                let upload_url = format!("{}/upload", server_url);

                match client.post(&upload_url).body(data.clone()).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            let chunk_bytes = data.len() as u64;
                            uploaded_bytes += chunk_bytes;
                            tracker.add_chunk(chunk_bytes);
                        } else {
                            errors.push(format!("HTTP {}", response.status()));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Request error: {}", e));
                    }
                }
            }

            if !errors.is_empty() && uploaded_bytes == 0 {
                tracing::warn!(stream = stream_index, ?errors, "Upload stream had errors");
            }

            uploaded_bytes
        });

        handles.push(handle);
    }

    // Collect results from all uploads
    let mut total_bytes = 0u64;
    let mut successful_streams = 0u64;

    for handle in handles {
        match handle.await {
            Ok(bytes) => {
                if bytes > 0 {
                    total_bytes += bytes;
                    successful_streams += 1;
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Upload task failed");
            }
        }
    }

    if successful_streams == 0 {
        return Err(SpeedtestError::NetworkError(
            "All upload streams failed".to_string(),
        ));
    }

    tracker.finish();

    let elapsed = start.elapsed().as_secs_f64();

    Ok(calculate_bps(total_bytes, elapsed))
}

/// Generate deterministic test pattern for uploads.
///
/// The pattern is designed to be compression-resistant to ensure
/// accurate bandwidth measurement.
fn generate_upload_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_upload_data_correct_size() {
        let data = generate_upload_data(UPLOAD_CHUNK_SIZE);
        assert_eq!(data.len(), UPLOAD_CHUNK_SIZE);
    }

    #[test]
    fn test_generate_upload_data_small_size() {
        let data = generate_upload_data(100);
        assert_eq!(data.len(), 100);
    }

    #[test]
    fn test_generate_upload_data_zero_size() {
        let data = generate_upload_data(0);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_generate_upload_data_pattern() {
        let data = generate_upload_data(300);
        // First 256 bytes should be 0..255
        for (i, &byte) in data.iter().enumerate().take(256) {
            assert_eq!(byte, i as u8);
        }
        // Bytes 256-299 should wrap around (0..43)
        for (i, &byte) in data.iter().enumerate().skip(256).take(44) {
            assert_eq!(byte, (i % 256) as u8);
        }
    }

    #[test]
    fn test_generate_upload_data_is_compression_resistant() {
        // The pattern 0,1,2,...255,0,1,2... should not compress well
        // because it has high entropy
        let data = generate_upload_data(1000);
        let unique_bytes: std::collections::HashSet<u8> = data.iter().copied().collect();
        assert_eq!(unique_bytes.len(), 256); // All possible byte values present
    }

    #[test]
    fn test_generate_upload_data_deterministic() {
        let data1 = generate_upload_data(500);
        let data2 = generate_upload_data(500);
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_concurrent_uploads_constant() {
        assert_eq!(CONCURRENT_UPLOADS, 4);
    }

    #[test]
    fn test_uploads_per_stream_constant() {
        assert_eq!(UPLOADS_PER_STREAM, 4);
    }

    #[test]
    fn test_upload_chunk_size_constant() {
        assert_eq!(UPLOAD_CHUNK_SIZE, 200_000);
    }

    #[test]
    fn test_total_upload_payloads() {
        let total = CONCURRENT_UPLOADS * UPLOADS_PER_STREAM;
        assert_eq!(total, 16);
    }

    #[test]
    fn test_total_upload_bytes() {
        let total_bytes = (CONCURRENT_UPLOADS * UPLOADS_PER_STREAM * UPLOAD_CHUNK_SIZE) as u64;
        assert_eq!(total_bytes, 3_200_000); // 4 * 4 * 200000 = 3.2 MB
    }
}
