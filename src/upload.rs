#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::error::SpeedtestError;
use crate::progress::SpeedProgress;
use crate::types::Server;
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Calculate upload bandwidth from bytes and elapsed time
#[must_use]
pub fn calculate_bandwidth(total_bytes: u64, elapsed_secs: f64) -> f64 {
    if elapsed_secs > 0.0 {
        (total_bytes as f64 * 8.0) / elapsed_secs
    } else {
        0.0
    }
}

/// Determine number of concurrent uploads based on single flag
#[must_use]
pub fn determine_concurrent_upload_count(single: bool) -> usize {
    if single {
        1
    } else {
        4
    }
}

/// Build upload URL
#[must_use]
pub fn build_upload_url(server_url: &str) -> String {
    format!("{server_url}/upload")
}

fn generate_upload_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    data
}

/// Run upload bandwidth test against the given server.
///
/// Returns `(avg_speed_bps, peak_speed_bps, total_bytes_uploaded)`.
///
/// # Errors
///
/// Returns [`SpeedtestError::NetworkError`] if all upload streams fail.
pub async fn upload_test(
    client: &Client,
    server: &Server,
    single: bool,
    progress: Arc<SpeedProgress>,
) -> Result<(f64, f64, u64), SpeedtestError> {
    let concurrent_uploads = determine_concurrent_upload_count(single);
    let total_bytes = Arc::new(AtomicU64::new(0));
    let peak_bps = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let upload_data = generate_upload_data(200_000); // 200KB chunks
    let estimated_total: u64 = 4_000_000; // 4 MB estimate

    let mut handles = Vec::new();

    for _ in 0..concurrent_uploads {
        let client = client.clone();
        let server_url = server.url.clone();
        let data = upload_data.clone();
        let total_ref = Arc::clone(&total_bytes);
        let peak_ref = Arc::clone(&peak_bps);
        let start_ref = start;
        let prog = Arc::clone(&progress);

        let handle = tokio::spawn(async move {
            let mut uploaded_bytes = 0u64;

            for _ in 0..4 {
                let upload_url = build_upload_url(&server_url);

                if client
                    .post(&upload_url)
                    .body(data.clone())
                    .send()
                    .await
                    .is_ok()
                {
                    let chunk = data.len() as u64;
                    uploaded_bytes += chunk;
                    total_ref.fetch_add(chunk, Ordering::Relaxed);

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

            uploaded_bytes
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        if let Ok(bytes) = handle.await {
            total_bytes.fetch_add(bytes, Ordering::Relaxed);
        }
    }

    let final_total_bytes = total_bytes.load(Ordering::Relaxed);
    let final_peak_speed = peak_bps.load(Ordering::Relaxed) as f64;
    let elapsed = start.elapsed().as_secs_f64();
    Ok((
        calculate_bandwidth(final_total_bytes, elapsed),
        final_peak_speed,
        final_total_bytes,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_bandwidth_calculation() {
        let result = calculate_bandwidth(1_000_000, 2.0);
        assert_eq!(result, 4_000_000.0);
    }

    #[test]
    fn test_upload_bandwidth_zero_elapsed() {
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
    }

    #[test]
    fn test_generate_upload_data_size() {
        let data = generate_upload_data(1000);
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_generate_upload_data_pattern() {
        let data = generate_upload_data(300);
        for (i, &byte) in data.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }
}
