use reqwest::Client;
use futures_util::StreamExt;
use crate::error::SpeedtestError;
use crate::types::Server;

pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_streams = if single { 1 } else { 4 };
    let mut total_bytes = 0u64;
    let start = std::time::Instant::now();

    // Download multiple files simultaneously
    let mut handles = Vec::new();

    for _i in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            // Download test files
            for j in 0..4 {
                let test_url = format!("{}/random{}.random", server_url, 350_000 * (j + 1) / 1000);

                match client.get(&test_url).send().await {
                    Ok(response) => {
                        let mut stream = response.bytes_stream();
                        while let Some(chunk) = stream.next().await {
                            if let Ok(bytes) = chunk {
                                stream_bytes += bytes.len() as u64;
                            }
                        }
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
