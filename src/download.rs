#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use crate::types::Server;
use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Calculate download bandwidth from bytes and elapsed time
#[must_use]
pub fn calculate_bandwidth(total_bytes: u64, elapsed_secs: f64) -> f64 {
    if elapsed_secs > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed_secs
    } else {
        0.0
    }
}

/// Determine number of concurrent streams based on single flag
#[must_use]
pub fn determine_stream_count(single: bool) -> usize {
    if single { 1 } else { 4 }
}

/// Extract base URL from server URL (strip /upload.php suffix)
#[must_use]
pub fn extract_base_url(url: &str) -> &str {
    url.strip_suffix("/upload.php")
        .or_else(|| url.strip_suffix("/upload.asp"))
        .unwrap_or(url)
}

/// Build test file URL using Speedtest.net standard naming
#[must_use]
pub fn build_test_url(server_url: &str, file_index: usize) -> String {
    let base = extract_base_url(server_url);
    let sizes = ["2000x2000", "3000x3000", "3500x3500", "4000x4000"];
    let size = sizes[file_index % sizes.len()];
    format!("{base}/random{size}.jpg")
}

/// Result from a single download stream
struct StreamResult {
    bytes: u64,
    elapsed_secs: f64,
}

use futures_util::StreamExt;

/// Run download bandwidth test against the given server.
///
/// Returns `(avg_speed_bps, peak_speed_bps, total_bytes_downloaded)`.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all download streams fail.
/// Returns [`SpeedtestError::Custom`] if the server URL is invalid.
pub async fn download_test(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<SpeedProgress>,
) -> Result<(f64, f64, u64), SpeedtestError> {
    let concurrent_streams = determine_stream_count(single);
    let total_bytes = Arc::new(AtomicU64::new(0));
    let peak_bps = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    // Estimated total: ~8-12 MB for typical speedtest, we'll update dynamically
    // Use a large estimate so the bar fills gradually
    let estimated_total: u64 = 15_000_000; // 15 MB estimate

    // Spawn streams that report progress
    let mut handles = Vec::new();
    for _ in 0..concurrent_streams {
        let client = client.clone();
        let server_url = server.url.clone();
        let total_ref = Arc::clone(&total_bytes);
        let peak_ref = Arc::clone(&peak_bps);
        let start_ref = start;
        let prog = Arc::clone(&progress);

        let handle = tokio::spawn(async move {
            let mut stream_bytes = 0u64;

            for j in 0..4 {
                let test_url = build_test_url(&server_url, j);

                if let Ok(response) = client.get(&test_url).send().await {
                    let mut stream = response.bytes_stream();
                    while let Some(item) = stream.next().await {
                        if let Ok(chunk) = item {
                            let len = chunk.len() as u64;
                            stream_bytes += len;
                            total_ref.fetch_add(len, Ordering::Relaxed);

                            let total_so_far = total_ref.load(Ordering::Relaxed);
                            let elapsed = start_ref.elapsed().as_secs_f64();
                            let speed = calculate_bandwidth(total_so_far, elapsed);

                            let current_peak = peak_ref.load(Ordering::Relaxed);
                            if speed > current_peak as f64 {
                                peak_ref.store(speed as u64, Ordering::Relaxed);
                            }

                            let pct = (total_so_far as f64 / estimated_total as f64).min(1.0);
                            prog.update(speed / 1_000_000.0, pct, total_so_far);
                        }
                    }
                }
            }

            StreamResult {
                bytes: stream_bytes,
                elapsed_secs: start_ref.elapsed().as_secs_f64(),
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    if results.is_empty() {
        return Ok((0.0, 0.0, 0));
    }

    let total_bandwidth: f64 = results
        .iter()
        .map(|r| calculate_bandwidth(r.bytes, r.elapsed_secs))
        .sum();

    let final_total_bytes = total_bytes.load(Ordering::Relaxed);
    let final_peak_speed = peak_bps.load(Ordering::Relaxed) as f64;
    let avg_bandwidth = total_bandwidth / results.len() as f64;
    Ok((avg_bandwidth, final_peak_speed, final_total_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_bandwidth_calculation() {
        let result = calculate_bandwidth(10_000_000, 2.0);
        assert_eq!(result, 40_000_000.0);
    }

    #[test]
    fn test_download_bandwidth_zero_elapsed() {
        let result = calculate_bandwidth(10_000_000, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_download_concurrent_streams_single() {
        assert_eq!(determine_stream_count(true), 1);
    }

    #[test]
    fn test_download_concurrent_streams_multiple() {
        assert_eq!(determine_stream_count(false), 4);
    }

    #[test]
    fn test_download_url_generation() {
        let server_url = "http://server.example.com/speedtest/upload.php";
        let test_url = build_test_url(server_url, 0);
        assert_eq!(
            test_url,
            "http://server.example.com/speedtest/random2000x2000.jpg"
        );
    }

    #[test]
    fn test_download_url_generation_cycles() {
        let server_url = "http://server.example.com/speedtest/upload.php";
        let url_0 = build_test_url(server_url, 0);
        let url_4 = build_test_url(server_url, 4);
        assert_eq!(url_0, url_4);
    }

    #[test]
    fn test_extract_base_url() {
        let url = "http://server.example.com:8080/speedtest/upload.php";
        assert_eq!(
            extract_base_url(url),
            "http://server.example.com:8080/speedtest"
        );
    }
}
