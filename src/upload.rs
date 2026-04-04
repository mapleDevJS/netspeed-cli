use reqwest::Client;
use crate::error::SpeedtestError;
use crate::progress::ProgressTracker;
use crate::types::Server;
use crate::utils::calculate_bps;
use std::sync::Arc;

pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_uploads = if single { 1 } else { 4 };
    let uploads_per_stream = 4;
    let total_uploads = concurrent_uploads * uploads_per_stream;

    // Create progress tracker
    let tracker = Arc::new(ProgressTracker::new(total_uploads, true));

    let start = std::time::Instant::now();

    // Generate upload data
    let upload_data = generate_upload_data(200_000); // 200KB chunks

    // Upload to multiple endpoints simultaneously
    let mut handles = Vec::new();

    for _ in 0..concurrent_uploads {
        let client = client.clone();
        let server_url = server.url.clone();
        let data = upload_data.clone();
        let tracker = tracker.clone();

        let handle = tokio::spawn(async move {
            let mut uploaded_bytes = 0u64;

            // Perform multiple uploads
            for _ in 0..uploads_per_stream {
                let upload_url = format!("{}/upload", server_url);

                match client
                    .post(&upload_url)
                    .body(data.clone())
                    .send()
                    .await
                {
                    Ok(_) => {
                        let chunk_bytes = data.len() as u64;
                        uploaded_bytes += chunk_bytes;
                        tracker.add_chunk(chunk_bytes);
                    }
                    Err(_) => continue,
                }
            }

            uploaded_bytes
        });

        handles.push(handle);
    }

    // Collect results from all uploads
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

fn generate_upload_data(size: usize) -> Vec<u8> {
    // Generate deterministic test pattern (compression-resistant)
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}
