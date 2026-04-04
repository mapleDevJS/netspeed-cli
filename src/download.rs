use reqwest::Client;
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

/// Extract base URL from server URL (strip /upload.php suffix)
/// e.g., "http://host:8080/speedtest/upload.php" -> "http://host:8080/speedtest"
pub fn extract_base_url(url: &str) -> &str {
    url.strip_suffix("/upload.php")
        .or_else(|| url.strip_suffix("/upload.asp"))
        .unwrap_or(url)
}

/// Build test file URL using Speedtest.net standard naming
pub fn build_test_url(server_url: &str, file_index: usize) -> String {
    let base = extract_base_url(server_url);
    // Standard Speedtest.net test file sizes (in bytes dimension naming)
    let sizes = ["2000x2000", "3000x3000", "3500x3500", "4000x4000"];
    let size = sizes[file_index % sizes.len()];
    format!("{}/random{}.jpg", base, size)
}

/// Result from a single download stream
struct StreamResult {
    bytes: u64,
    elapsed_secs: f64,
}

pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_streams = determine_stream_count(single);

    // Download multiple files simultaneously
    let mut handles = Vec::new();

    for _i in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let stream_start = std::time::Instant::now();

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            // Download test files with increasing sizes
            for j in 0..4 {
                let test_url = build_test_url(&server_url, j);

                match client.get(&test_url).send().await {
                    Ok(response) => {
                        if let Ok(body) = response.bytes().await {
                            stream_bytes += body.len() as u64;
                        }
                    }
                    Err(_) => continue,
                }
            }

            StreamResult {
                bytes: stream_bytes,
                elapsed_secs: stream_start.elapsed().as_secs_f64(),
            }
        });

        handles.push(handle);
    }

    // Collect results from all streams
    let mut stream_results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            stream_results.push(result);
        }
    }

    if stream_results.is_empty() {
        return Ok(0.0);
    }

    // Calculate per-stream bandwidth and take the average
    // This matches how official speedtest tools report results
    let total_bandwidth: f64 = stream_results
        .iter()
        .map(|r| calculate_bandwidth(r.bytes, r.elapsed_secs))
        .sum();

    let avg_bandwidth = total_bandwidth / stream_results.len() as f64;

    Ok(avg_bandwidth)
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
        let server_url = "http://server.example.com/speedtest/upload.php";
        let test_url = build_test_url(server_url, 0);

        assert_eq!(test_url, "http://server.example.com/speedtest/random2000x2000.jpg");
    }

    #[test]
    fn test_download_url_generation_various_sizes() {
        use super::build_test_url;

        let server_url = "http://server.example.com/speedtest/upload.php";
        let expected = vec![
            "http://server.example.com/speedtest/random2000x2000.jpg",
            "http://server.example.com/speedtest/random3000x3000.jpg",
            "http://server.example.com/speedtest/random3500x3500.jpg",
            "http://server.example.com/speedtest/random4000x4000.jpg",
        ];

        for (i, exp) in expected.iter().enumerate() {
            let test_url = build_test_url(server_url, i);
            assert_eq!(test_url, *exp);
        }
    }

    #[test]
    fn test_extract_base_url_upload_php() {
        use super::extract_base_url;

        let url = "http://server.example.com:8080/speedtest/upload.php";
        assert_eq!(extract_base_url(url), "http://server.example.com:8080/speedtest");
    }

    #[test]
    fn test_extract_base_url_upload_asp() {
        use super::extract_base_url;

        let url = "http://server.example.com/speedtest/upload.asp";
        assert_eq!(extract_base_url(url), "http://server.example.com/speedtest");
    }

    #[test]
    fn test_extract_base_url_no_suffix() {
        use super::extract_base_url;

        let url = "http://server.example.com/speedtest";
        assert_eq!(extract_base_url(url), "http://server.example.com/speedtest");
    }

    #[test]
    fn test_download_url_generation_cycles() {
        use super::build_test_url;

        // After 4 files, it should cycle back to the first
        let server_url = "http://server.example.com/speedtest/upload.php";
        let url_0 = build_test_url(server_url, 0);
        let url_4 = build_test_url(server_url, 4);

        assert_eq!(url_0, url_4);
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
