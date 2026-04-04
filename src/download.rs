use crate::error::SpeedtestError;
use crate::progress::ProgressTracker;
use crate::types::Server;
use crate::utils::calculate_bps;
use futures_util::StreamExt;
use reqwest::Client;
use std::sync::Arc;

/// Number of concurrent download streams in multi-connection mode
pub const CONCURRENT_STREAMS: usize = 4;

/// Number of chunks per download stream
pub const CHUNKS_PER_STREAM: usize = 4;

/// Base size for test files in bytes (350KB)
pub const TEST_FILE_BASE_SIZE: u64 = 350_000;

/// Run download speed test against the specified server.
///
/// Downloads multiple files concurrently to measure throughput.
/// Returns speed in bits per second.
#[tracing::instrument(skip(client, server), fields(server_id = %server.id))]
pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_streams = if single { 1 } else { CONCURRENT_STREAMS };
    let total_chunks = concurrent_streams * CHUNKS_PER_STREAM;

    // Create progress tracker
    let tracker = Arc::new(ProgressTracker::new(total_chunks, true));

    let start = std::time::Instant::now();

    // Download multiple files simultaneously
    let mut handles = Vec::new();

    for i in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let tracker = tracker.clone();
        let stream_index = i;

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;
            let mut errors = Vec::new();

            // Download test files
            for j in 0..CHUNKS_PER_STREAM {
                let file_size = TEST_FILE_BASE_SIZE * (j as u64 + 1) / 1000;
                let test_url = format!("{}/random{}.random", server_url, file_size);

                match client.get(&test_url).send().await {
                    Ok(response) => {
                        if !response.status().is_success() {
                            errors.push(format!("HTTP {}", response.status()));
                            continue;
                        }

                        let mut stream = response.bytes_stream();
                        let mut chunk_bytes = 0u64;
                        while let Some(chunk) = stream.next().await {
                            match chunk {
                                Ok(bytes) => {
                                    chunk_bytes += bytes.len() as u64;
                                }
                                Err(e) => {
                                    errors.push(format!("Stream error: {}", e));
                                    break;
                                }
                            }
                        }
                        stream_bytes += chunk_bytes;
                        tracker.add_chunk(chunk_bytes);
                    }
                    Err(e) => {
                        errors.push(format!("Request error: {}", e));
                        continue;
                    }
                }
            }

            if !errors.is_empty() && stream_bytes == 0 {
                tracing::warn!(stream = stream_index, ?errors, "Download stream had errors");
            }

            stream_bytes
        });

        handles.push(handle);
    }

    // Collect results from all streams
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
                tracing::warn!(error = %e, "Download task failed");
            }
        }
    }

    if successful_streams == 0 {
        return Err(SpeedtestError::NetworkError(
            "All download streams failed".to_string(),
        ));
    }

    tracker.finish();

    let elapsed = start.elapsed().as_secs_f64();

    Ok(calculate_bps(total_bytes, elapsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_streams_constant() {
        assert_eq!(CONCURRENT_STREAMS, 4);
    }

    #[test]
    fn test_chunks_per_stream_constant() {
        assert_eq!(CHUNKS_PER_STREAM, 4);
    }

    #[test]
    fn test_test_file_base_size_constant() {
        assert_eq!(TEST_FILE_BASE_SIZE, 350_000);
    }

    #[test]
    fn test_total_chunks_multi_stream() {
        let total = CONCURRENT_STREAMS * CHUNKS_PER_STREAM;
        assert_eq!(total, 16);
    }

    #[test]
    fn test_total_chunks_single_stream() {
        let single = 1;
        let total = single * CHUNKS_PER_STREAM;
        assert_eq!(total, 4);
    }

    #[test]
    fn test_file_size_calculation_first_chunk() {
        let j = 0;
        let file_size = TEST_FILE_BASE_SIZE * (j as u64 + 1) / 1000;
        assert_eq!(file_size, 350); // 350000 * 1 / 1000
    }

    #[test]
    fn test_file_size_calculation_last_chunk() {
        let j = 3; // CHUNKS_PER_STREAM - 1
        let file_size = TEST_FILE_BASE_SIZE * (j as u64 + 1) / 1000;
        assert_eq!(file_size, 1400); // 350000 * 4 / 1000
    }

    #[test]
    fn test_download_url_format() {
        let server_url = "http://server.example.com";
        let file_size = 700;
        let test_url = format!("{}/random{}.random", server_url, file_size);
        assert_eq!(test_url, "http://server.example.com/random700.random");
    }
}
