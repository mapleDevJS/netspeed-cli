use reqwest::Client;
use crate::error::SpeedtestError;
use crate::types::Server;

/// Calculate upload bandwidth from bytes and elapsed time
pub fn calculate_bandwidth(total_bytes: u64, elapsed: f64) -> f64 {
    if elapsed > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed
    } else {
        0.0
    }
}

/// Determine number of concurrent uploads based on single flag
pub fn determine_concurrent_upload_count(single: bool) -> usize {
    if single { 1 } else { 4 }
}

/// Build upload URL
pub fn build_upload_url(server_url: &str) -> String {
    format!("{}/upload", server_url)
}

pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
) -> Result<f64, SpeedtestError> {
    let concurrent_uploads = determine_concurrent_upload_count(single);
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
                let upload_url = build_upload_url(&server_url);

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
    Ok(calculate_bandwidth(total_bytes, elapsed))
}

fn generate_upload_data(size: usize) -> Vec<u8> {
    // Generate pattern data for upload testing
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
    fn test_upload_bandwidth_calculation() {
        // Test bandwidth calculation logic
        let result = calculate_bandwidth(1_000_000, 2.0);
        assert_eq!(result, 4_000_000.0);
    }

    #[test]
    fn test_upload_bandwidth_zero_elapsed() {
        // Test division by zero protection
        let result = calculate_bandwidth(1_000_000, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_upload_concurrent_count_single() {
        assert_eq!(determine_concurrent_upload_count(true), 1);
    }

    #[test]
    fn test_upload_concurrent_count_multiple() {
        assert_eq!(determine_concurrent_upload_count(false), 4);
    }

    #[test]
    fn test_upload_url_generation() {
        let url = build_upload_url("http://server.example.com");
        assert!(url.ends_with("/upload"));
        assert!(url.contains("server.example.com"));
    }

    #[test]
    fn test_generate_upload_data_size() {
        let data = generate_upload_data(1000);
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_generate_upload_data_pattern() {
        let data = generate_upload_data(300);
        // Verify the pattern repeats every 256 bytes
        for (i, &byte) in data.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }

    #[test]
    fn test_generate_upload_data_small() {
        let data = generate_upload_data(10);
        assert_eq!(data.len(), 10);
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_generate_upload_data_large() {
        let size = 200_000; // Same as production
        let data = generate_upload_data(size);
        assert_eq!(data.len(), size);
        // Verify first and last bytes
        assert_eq!(data[0], 0);
        assert_eq!(data[255], 255);
        assert_eq!(data[256], 0);
        assert_eq!(data[size - 1], ((size - 1) % 256) as u8);
    }

    #[test]
    fn test_calculate_bandwidth_various_sizes() {
        // Test various bandwidth calculations
        assert_eq!(calculate_bandwidth(1_000_000, 1.0), 8_000_000.0);
        assert_eq!(calculate_bandwidth(500_000, 1.0), 4_000_000.0);
        assert_eq!(calculate_bandwidth(2_000_000, 2.0), 8_000_000.0);
    }

    #[test]
    fn test_calculate_bandwidth_zero_bytes() {
        assert_eq!(calculate_bandwidth(0, 5.0), 0.0);
    }

    #[test]
    fn test_build_upload_url_format() {
        let server_urls = vec![
            "http://server1.com",
            "https://server2.com",
            "http://192.168.1.1:8080",
        ];

        for url in server_urls {
            let upload_url = build_upload_url(url);
            assert!(upload_url.ends_with("/upload"));
            assert!(upload_url.starts_with(url));
        }
    }
}
