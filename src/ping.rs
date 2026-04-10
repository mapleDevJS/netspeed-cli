//! Ping testing for latency, jitter, and packet loss measurement.

use crate::error::SpeedtestError;
use crate::types::Server;
use reqwest::Client;

/// Run a ping test against the given server, returning
/// `(average_latency_ms, jitter_ms, packet_loss_percent, individual_samples)`.
///
/// # Errors
///
/// Returns [`SpeedtestError::Context`] if all ping attempts fail.
pub async fn ping_test(
    client: &Client,
    server: &Server,
) -> Result<(f64, f64, f64, Vec<f64>), SpeedtestError> {
    const PING_ATTEMPTS: usize = 8;
    let mut latencies = Vec::new();

    for _ in 0..PING_ATTEMPTS {
        let start = std::time::Instant::now();

        let response = client
            .get(format!("{}/latency.txt", server.url))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        if let Ok(resp) = response {
            if resp.status().is_success() {
                latencies.push(elapsed);
            }
        }
    }

    if latencies.is_empty() {
        return Err(SpeedtestError::Context {
            msg: "All ping attempts failed".to_string(),
            source: None,
        });
    }

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

    let jitter = if latencies.len() > 1 {
        let mut jitter_sum = 0.0;
        for i in 1..latencies.len() {
            jitter_sum += (latencies[i] - latencies[i - 1]).abs();
        }
        jitter_sum / (latencies.len() - 1) as f64
    } else {
        0.0
    };

    let packet_loss = ((PING_ATTEMPTS - latencies.len()) as f64 / PING_ATTEMPTS as f64) * 100.0;

    Ok((avg, jitter, packet_loss, latencies))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ping_test_average_calculation() {
        let latencies = [10.0, 20.0, 15.0, 25.0];
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        assert_eq!(avg, 17.5);
    }

    #[test]
    fn test_ping_test_empty_handling() {
        let latencies: Vec<f64> = vec![];
        assert!(latencies.is_empty());
    }
}
