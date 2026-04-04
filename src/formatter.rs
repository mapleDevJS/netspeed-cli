use crate::error::SpeedtestError;
use crate::types::{CsvOutput, Server, TestResult};

pub fn format_simple(result: &TestResult, bytes: bool) -> Result<(), SpeedtestError> {
    let divider = if bytes { 8.0 } else { 1.0 };
    let unit = if bytes { "Byte/s" } else { "bit/s" };

    if let Some(ping) = result.ping {
        println!("Ping: {:.3} ms", ping);
    }

    if let Some(download) = result.download {
        let value = download / divider;
        println!("Download: {:.2} M{}/s", value / 1_000_000.0, unit);
    }

    if let Some(upload) = result.upload {
        let value = upload / divider;
        println!("Upload: {:.2} M{}/s", value / 1_000_000.0, unit);
    }

    Ok(())
}

pub fn format_json(result: &TestResult) -> Result<(), SpeedtestError> {
    use std::io::IsTerminal;

    let is_tty = std::io::stdout().is_terminal();

    let output = if is_tty {
        serde_json::to_string_pretty(result)?
    } else {
        serde_json::to_string(result)?
    };

    println!("{}", output);

    Ok(())
}

pub fn format_csv(
    result: &TestResult,
    delimiter: char,
    print_header: bool,
) -> Result<(), SpeedtestError> {
    let stdout = std::io::stdout();
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter as u8)
        .from_writer(stdout);

    if print_header {
        wtr.write_record([
            "Server ID",
            "Sponsor",
            "Server Name",
            "Timestamp",
            "Distance",
            "Ping",
            "Download",
            "Upload",
            "Share",
            "IP Address",
        ])?;
    }

    let csv_output = CsvOutput {
        server_id: result.server.id.clone(),
        sponsor: result.server.sponsor.clone(),
        server_name: result.server.name.clone(),
        timestamp: result.timestamp.clone(),
        distance: result.server.distance,
        ping: result.ping.unwrap_or(0.0),
        download: result.download.unwrap_or(0.0),
        upload: result.upload.unwrap_or(0.0),
        share: result.share_url.clone().unwrap_or_default(),
        ip_address: result.client_ip.clone().unwrap_or_default(),
    };

    wtr.serialize(csv_output)?;
    wtr.flush()?;

    Ok(())
}

pub fn format_list(servers: &[Server]) -> Result<(), SpeedtestError> {
    for server in servers {
        println!(
            "{:>6}: {} ({}, {}) [{:.2} km]",
            server.id, server.sponsor, server.name, server.country, server.distance
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Server, ServerInfo, TestResult};

    fn create_sample_result() -> TestResult {
        TestResult {
            server: ServerInfo {
                id: "12345".to_string(),
                name: "Test Server".to_string(),
                sponsor: "Test ISP".to_string(),
                country: "US".to_string(),
                distance: 100.0,
            },
            ping: Some(25.5),
            download: Some(100_000_000.0),
            upload: Some(50_000_000.0),
            share_url: Some("https://example.com/result".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: Some("1.2.3.4".to_string()),
        }
    }

    #[test]
    fn test_format_simple_bits() {
        let result = create_sample_result();
        // Should not panic
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_bytes() {
        let result = create_sample_result();
        assert!(format_simple(&result, true).is_ok());
    }

    #[test]
    fn test_format_simple_no_download() {
        let mut result = create_sample_result();
        result.download = None;
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_upload() {
        let mut result = create_sample_result();
        result.upload = None;
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_no_ping() {
        let mut result = create_sample_result();
        result.ping = None;
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_simple_all_none() {
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: None,
            download: None,
            upload: None,
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };
        assert!(format_simple(&result, false).is_ok());
    }

    #[test]
    fn test_format_json_pretty() {
        let result = create_sample_result();
        // When stdout is not TTY (in tests), should produce compact JSON
        assert!(format_json(&result).is_ok());
    }

    #[test]
    fn test_format_json_contains_fields() {
        let result = create_sample_result();
        // We can't easily capture stdout, but we can verify the function doesn't panic
        // and returns Ok
        assert!(format_json(&result).is_ok());
    }

    #[test]
    fn test_format_csv_with_header() {
        let result = create_sample_result();
        assert!(format_csv(&result, ',', true).is_ok());
    }

    #[test]
    fn test_format_csv_without_header() {
        let result = create_sample_result();
        assert!(format_csv(&result, ',', false).is_ok());
    }

    #[test]
    fn test_format_csv_custom_delimiter() {
        let result = create_sample_result();
        assert!(format_csv(&result, ';', true).is_ok());
    }

    #[test]
    fn test_format_csv_tab_delimiter() {
        let result = create_sample_result();
        assert!(format_csv(&result, '\t', false).is_ok());
    }

    #[test]
    fn test_format_csv_with_none_values() {
        let result = TestResult {
            server: ServerInfo {
                id: "1".to_string(),
                name: "Test".to_string(),
                sponsor: "Test".to_string(),
                country: "US".to_string(),
                distance: 0.0,
            },
            ping: None,
            download: None,
            upload: None,
            share_url: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            client_ip: None,
        };
        assert!(format_csv(&result, ',', true).is_ok());
    }

    #[test]
    fn test_format_list_empty() {
        let servers: Vec<Server> = vec![];
        assert!(format_list(&servers).is_ok());
    }

    #[test]
    fn test_format_list_single_server() {
        let servers = vec![Server {
            id: "12345".to_string(),
            url: "http://example.com".to_string(),
            name: "Test Server".to_string(),
            sponsor: "Test ISP".to_string(),
            country: "US".to_string(),
            lat: 40.0,
            lon: -74.0,
            distance: 100.0,
            latency: 25.0,
        }];
        assert!(format_list(&servers).is_ok());
    }

    #[test]
    fn test_format_list_multiple_servers() {
        let servers = vec![
            Server {
                id: "1".to_string(),
                url: "http://srv1.com".to_string(),
                name: "Server 1".to_string(),
                sponsor: "ISP 1".to_string(),
                country: "US".to_string(),
                lat: 40.0,
                lon: -74.0,
                distance: 50.0,
                latency: 20.0,
            },
            Server {
                id: "2".to_string(),
                url: "http://srv2.com".to_string(),
                name: "Server 2".to_string(),
                sponsor: "ISP 2".to_string(),
                country: "DE".to_string(),
                lat: 52.0,
                lon: 13.0,
                distance: 150.0,
                latency: 35.0,
            },
        ];
        assert!(format_list(&servers).is_ok());
    }
}
