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
    let is_tty = atty::is(atty::Stream::Stdout);

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
        wtr.write_record(&[
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
            server.id,
            server.sponsor,
            server.name,
            server.country,
            server.distance
        );
    }

    Ok(())
}

// Helper to detect if stdout is a TTY
mod atty {
    use std::io::IsTerminal;

    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        std::io::stdout().is_terminal()
    }
}
