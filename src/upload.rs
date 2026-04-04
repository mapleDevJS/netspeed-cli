use reqwest::Client;
use crate::error::SpeedtestError;
use crate::types::Server;

pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_uploads = if single { 1 } else { 4 };
    let mut total_bytes = 0u64;
    let start = std::time::Instant::now();

    // Generate upload data
    let upload_data = generate_upload_data(200_000); // 200KB chunks

    // Upload to multiple endpoints simultaneously
    let mut handles = Vec::new();

    for _ in 0..concurrent_uploads {
        let client = client.clone();
        let server_url = server.url.clone();
        let data = upload_data.clone();

        let handle = tokio::spawn(async move {
            let mut uploaded_bytes = 0u64;

            // Perform multiple uploads
            for _ in 0..4 {
                let upload_url = format!("{}/upload", server_url);

                match client
                    .post(&upload_url)
                    .body(data.clone())
                    .send()
                    .await
                {
                    Ok(_) => {
                        uploaded_bytes += data.len() as u64;
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
            total_bytes += bytes;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    // Calculate bits per second
    let bits_per_sec = if elapsed > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed
    } else {
        0.0
    };

    Ok(bits_per_sec)
}

fn generate_upload_data(size: usize) -> Vec<u8> {
    // Generate random data for upload testing
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}
