use reqwest::Client;
use futures_util::StreamExt;
use crate::error::SpeedtestError;
use crate::progress::ProgressTracker;
use crate::types::Server;
use crate::utils::calculate_bps;
use std::sync::Arc;

pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_streams = if single { 1 } else { 4 };
    let chunks_per_stream = 4;
    let total_chunks = concurrent_streams * chunks_per_stream;

    // Create progress tracker
    let tracker = Arc::new(ProgressTracker::new(total_chunks, true));

    let start = std::time::Instant::now();

    // Download multiple files simultaneously
    let mut handles = Vec::new();

    for _ in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let tracker = tracker.clone();

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            // Download test files
            for j in 0..chunks_per_stream {
                let test_url = format!("{}/random{}.random", server_url, 350_000 * (j + 1) / 1000);

                match client.get(&test_url).send().await {
                    Ok(response) => {
                        let mut stream = response.bytes_stream();
                        let mut chunk_bytes = 0u64;
                        while let Some(chunk) = stream.next().await {
                            if let Ok(bytes) = chunk {
                                chunk_bytes += bytes.len() as u64;
                            }
                        }
                        stream_bytes += chunk_bytes;
                        tracker.add_chunk(chunk_bytes);
                    }
                    Err(_) => continue,
                }
            }

            stream_bytes
        });

        handles.push(handle);
    }

    // Collect results from all streams
    for handle in handles {
        if let Ok(bytes) = handle.await {
            let _ = bytes; // Already tracked via progress
        }
    }

    tracker.finish();

    let elapsed = start.elapsed().as_secs_f64();
    let total_bytes = tracker.total_bytes();

    Ok(calculate_bps(total_bytes, elapsed))
}
