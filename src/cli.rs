pub use clap::ArgAction;
use clap::{Parser, ValueEnum};

// Shared validation functions (also used by build.rs via include!)
include!("validate.rs");

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[allow(deprecated)]
#[command(name = "netspeed-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "Test internet bandwidth via speedtest.net servers",
    long_about = "Test internet bandwidth via speedtest.net servers.

The default workflow runs a full bandwidth test:
  1. Discover nearest speedtest.net servers
  2. Measure latency (8 ping samples → latency, jitter, packet loss)
  3. Measure download speed (multi-stream, concurrent downloads)
  4. Measure upload speed (multi-stream, concurrent uploads)
  5. Grade results (A+ to F) and show real-world usage estimates

Results are saved to a local history file for trend tracking."
)]
#[command(
    after_help = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Examples:
  netspeed-cli                          Run a full speed test
  netspeed-cli --format compact         Key metrics with ratings
  netspeed-cli --format dashboard       Rich dashboard with history
  netspeed-cli --format json            Machine-readable output
  netspeed-cli --list                   List available servers
  netspeed-cli --history                Show test history
  netspeed-cli --profile gamer          Optimize output for gaming
  netspeed-cli --theme light            Light terminal background
  netspeed-cli --no-emoji               Disable emoji output
  netspeed-cli --quiet                  Suppress progress output
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
)]
pub struct CliArgs {
    /// Do not perform download test
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_download: bool,

    /// Do not perform upload test
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_upload: bool,

    /// Only use a single connection instead of multiple
    ///
    /// A single connection measures sustained throughput.
    /// The default uses multiple connections to measure burst/bandwidth capacity.
    #[arg(
        long,
        action = ArgAction::SetTrue,
        long_help = "Use a single TCP connection for testing (measures sustained throughput).\nThe default uses multiple connections (measures burst/bandwidth capacity)."
    )]
    pub single: bool,

    /// Display values in bytes instead of bits
    ///
    /// The default displays values in bits (standard for ISP advertising).
    #[arg(
        long,
        action = ArgAction::SetTrue,
        long_help = "Display values in bytes instead of bits per second.\nThe default uses bits (standard for ISP advertising)."
    )]
    pub bytes: bool,

    /// Suppress verbose output, only show basic information
    ///
    /// Basic information = one-line summary: latency, download, upload.
    #[deprecated(since = "0.9.0", note = "Use --format simple instead")]
    #[arg(
        long,
        long_help = "Suppress verbose output, only show basic information.\nBasic information = one-line summary: latency, download, upload.\nDeprecated: use --format simple instead."
    )]
    pub simple: bool,

    /// Output in CSV format
    ///
    /// CSV output is suitable for spreadsheet analysis.
    /// Use --csv-header to include column names.
    #[deprecated(since = "0.9.0", note = "Use --format csv instead")]
    #[arg(
        long,
        long_help = "Output in CSV format for spreadsheet analysis.\nDeprecated: use --format csv instead."
    )]
    pub csv: bool,

    /// Single character delimiter for CSV output (default: ",")
    #[arg(long, default_value = ",", value_parser = validate_csv_delimiter)]
    pub csv_delimiter: char,

    /// Print CSV headers
    #[arg(long)]
    pub csv_header: bool,

    /// Output in JSON format
    ///
    /// JSON output is machine-readable and includes all measured values.
    #[deprecated(since = "0.9.0", note = "Use --format json instead")]
    #[arg(
        long,
        long_help = "Output in JSON format (machine-readable).\nDeprecated: use --format json instead."
    )]
    pub json: bool,

    /// Output format (supersedes --json, --csv, --simple)
    #[arg(long, value_enum)]
    pub format: Option<OutputFormatType>,

    /// Display a list of speedtest.net servers sorted by distance
    #[arg(
        long,
        long_help = "Display a list of nearby speedtest.net servers sorted by distance.\nDoes not run a bandwidth test."
    )]
    pub list: bool,

    /// Specify a server ID to test against (can be supplied multiple times)
    #[arg(long)]
    pub server: Vec<String>,

    /// Exclude a server from selection (can be supplied multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Source IP address to bind to (IPv4 or IPv6)
    #[arg(long, value_parser = validate_ip_address, long_help = "Source IP address to bind to (IPv4 or IPv6).\nUseful on multi-homed systems to select a specific interface.")]
    pub source: Option<String>,

    /// HTTP timeout in seconds (default: 10)
    #[arg(long, default_value = "10", value_parser = validate_timeout)]
    pub timeout: u64,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    pub generate_completion: Option<ShellType>,

    /// Display test history
    #[arg(
        long,
        long_help = "Display test history from the local JSON file.\nDoes not run a bandwidth test."
    )]
    pub history: bool,

    /// Suppress all progress output (JSON/CSV still go to stdout)
    #[arg(
        long,
        long_help = "Suppress all progress output during the test.\nJSON/CSV output still goes to stdout."
    )]
    pub quiet: bool,

    /// Validate configuration and exit without running tests
    #[arg(
        long,
        long_help = "Validate configuration and exit without running tests.\nPrints the server that would be selected and confirms connectivity."
    )]
    pub dry_run: bool,

    /// Disable emoji output (for environments where emojis don't render well)
    #[arg(long)]
    pub no_emoji: bool,

    /// Minimal ASCII-only output (no Unicode box-drawing characters)
    #[arg(long)]
    pub minimal: bool,

    /// User profile for customized output (gamer, streamer, remote-worker, power-user, casual)
    ///
    /// Profiles control which sections are shown and grading thresholds.
    /// gamer = latency-focused, streamer = download-focused, etc.
    #[arg(
        long,
        value_name = "PROFILE",
        long_help = "User profile for customized output.\nProfiles control displayed sections and grading thresholds:\n  gamer:          Latency-focused (ping/jitter weighted higher)\n  streamer:       Download-focused (download weighted higher)\n  remote-worker:  Upload-focused (upload weighted higher)\n  power-user:     All metrics with full detail\n  casual:         Simple pass/fail view"
    )]
    pub profile: Option<String>,

    /// Output color theme (dark, light, high-contrast, monochrome)
    #[arg(long, value_name = "THEME", default_value = "dark")]
    pub theme: String,

    /// Show the configuration file path and exit
    #[arg(long)]
    pub show_config_path: bool,
}

fn validate_csv_delimiter(s: &str) -> Result<char, String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() != 1 {
        return Err("CSV delimiter must be a single character".to_string());
    }

    let delimiter = chars[0];
    if !",;|\\t".contains(delimiter) {
        return Err(format!(
            "Invalid CSV delimiter '{delimiter}'. Must be one of: comma, semicolon, pipe, or tab"
        ));
    }

    Ok(delimiter)
}

fn validate_timeout(s: &str) -> Result<u64, String> {
    let timeout: u64 = s
        .parse()
        .map_err(|_| format!("Invalid timeout value: '{s}'"))?;
    if timeout == 0 {
        return Err("Timeout must be greater than 0".to_string());
    }
    if timeout > 300 {
        return Err("Timeout must be 300 seconds or less".to_string());
    }
    Ok(timeout)
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Elvish,
}

/// Unified output format selection (supersedes --json, --csv, --simple).
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormatType {
    /// Machine-readable JSON output
    Json,
    /// JSON Lines for logging (one JSON object per line)
    Jsonl,
    /// CSV format for spreadsheet analysis
    Csv,
    /// Ultra-minimal: just grade + speeds (e.g., "B+ 150.5↓ 25.3↑ 12ms")
    Minimal,
    /// Minimal one-line summary
    Simple,
    /// Key metrics with quality ratings
    Compact,
    /// Full analysis with per-metric grades (default)
    Detailed,
    /// Rich terminal dashboard with capability matrix
    Dashboard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_csv_delimiter_comma() {
        assert!(validate_csv_delimiter(",").is_ok());
    }

    #[test]
    fn test_validate_csv_delimiter_semicolon() {
        assert!(validate_csv_delimiter(";").is_ok());
    }

    #[test]
    fn test_validate_csv_delimiter_pipe() {
        assert!(validate_csv_delimiter("|").is_ok());
    }

    #[test]
    fn test_validate_csv_delimiter_invalid() {
        assert!(validate_csv_delimiter("a").is_err());
    }

    #[test]
    fn test_validate_csv_delimiter_multiple_chars() {
        assert!(validate_csv_delimiter(",,,").is_err());
    }

    #[test]
    fn test_validate_ip_address_valid() {
        assert!(validate_ip_address("192.168.1.1").is_ok());
    }

    #[test]
    fn test_validate_ip_address_localhost() {
        assert!(validate_ip_address("127.0.0.1").is_ok());
    }

    #[test]
    fn test_validate_ip_address_invalid_format() {
        assert!(validate_ip_address("192.168.1").is_err());
    }

    #[test]
    fn test_validate_ip_address_invalid_octet() {
        assert!(validate_ip_address("192.168.1.999").is_err());
    }

    #[test]
    fn test_validate_timeout_valid() {
        assert!(validate_timeout("10").is_ok());
    }

    #[test]
    fn test_validate_timeout_min() {
        assert!(validate_timeout("1").is_ok());
    }

    #[test]
    fn test_validate_timeout_max() {
        assert!(validate_timeout("300").is_ok());
    }

    #[test]
    fn test_validate_timeout_zero() {
        let result = validate_timeout("0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than 0"));
    }

    #[test]
    fn test_validate_timeout_too_large() {
        let result = validate_timeout("301");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("300 seconds or less"));
    }

    #[test]
    fn test_validate_timeout_invalid() {
        assert!(validate_timeout("abc").is_err());
    }
}
