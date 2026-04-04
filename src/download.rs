use reqwest::Client;
use futures_util::StreamExt;
use crate::error::SpeedtestError;
use crate::types::Server;

/// Calculate download bandwidth from bytes and elapsed time
pub fn calculate_bandwidth(total_bytes: u64, elapsed: f64) -> f64 {
    if elapsed > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed
    } else {
        0.0
    }
}

/// Determine number of concurrent streams based on single flag
pub fn determine_stream_count(single: bool) -> usize {
    if single { 1 } else { 4 }
}

/// Build test file URL
pub fn build_test_url(server_url: &str, size_kb: usize) -> String {
    format!("{}/random{}.random", server_url, size_kb)
}

pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_streams = determine_stream_count(single);
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
                let test_url = build_test_url(&server_url, 350_000 * (j + 1) / 1000);

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
    Ok(calculate_bandwidth(total_bytes, elapsed))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_download_bandwidth_calculation() {
        use super::calculate_bandwidth;
        
        // Test bandwidth calculation logic
        let result = calculate_bandwidth(10_000_000, 2.0);
        assert_eq!(result, 40_000_000.0);
    }

    #[test]
    fn test_download_bandwidth_zero_elapsed() {
        use super::calculate_bandwidth;
        
        // Test division by zero protection
        let result = calculate_bandwidth(10_000_000, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_download_concurrent_streams_single() {
        use super::determine_stream_count;
        
        // Test single stream configuration
        assert_eq!(determine_stream_count(true), 1);
    }

    #[test]
    fn test_download_concurrent_streams_multiple() {
        use super::determine_stream_count;
        
        // Test multiple streams configuration
        assert_eq!(determine_stream_count(false), 4);
    }

    #[test]
    fn test_download_url_generation() {
        use super::build_test_url;
        
        // Test URL generation for test files
        let server_url = "http://server.example.com";
        let test_url = build_test_url(server_url, 350);
        
        assert!(test_url.contains("350"));
        assert!(test_url.ends_with(".random"));
        assert!(test_url.contains("random"));
    }

    #[test]
    fn test_download_url_generation_various_sizes() {
        use super::build_test_url;
        
        let server_url = "http://server.example.com";
        let sizes = vec![350, 700, 1050, 1400];
        
        for size in sizes {
            let test_url = build_test_url(server_url, size);
            assert!(test_url.contains(&size.to_string()));
            assert!(test_url.ends_with(".random"));
        }
    }

    #[test]
    fn test_calculate_bandwidth_small_file() {
        use super::calculate_bandwidth;
        
        // 1 MB in 0.1 seconds = 80 Mbps
        let result = calculate_bandwidth(1_000_000, 0.1);
        assert_eq!(result, 80_000_000.0);
    }

    #[test]
    fn test_calculate_bandwidth_large_file() {
        use super::calculate_bandwidth;
        
        // 100 MB in 10 seconds = 80 Mbps
        let result = calculate_bandwidth(100_000_000, 10.0);
        assert_eq!(result, 80_000_000.0);
    }

    #[test]
    fn test_calculate_bandwidth_zero_bytes() {
        use super::calculate_bandwidth;
        
        // 0 bytes should return 0 bandwidth
        let result = calculate_bandwidth(0, 5.0);
        assert_eq!(result, 0.0);
    }
}
