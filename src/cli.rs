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

Configuration precedence: CLI flags override config file values, which override built-in defaults.
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
pub struct Args {
    /// Do not perform download test
    #[arg(long, action = ArgAction::Set, default_missing_value = "true", num_args = 0..=1)]
    pub no_download: Option<bool>,

    /// Do not perform upload test
    #[arg(long, action = ArgAction::Set, default_missing_value = "true", num_args = 0..=1)]
    pub no_upload: Option<bool>,

    /// Only use a single connection instead of multiple
    ///
    /// A single connection measures sustained throughput.
    /// The default uses multiple connections to measure burst/bandwidth capacity.
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Use a single TCP connection for testing (measures sustained throughput).\nThe default uses multiple connections (measures burst/bandwidth capacity)."
    )]
    pub single: Option<bool>,

    /// Display values in bytes instead of bits
    ///
    /// The default displays values in bits (standard for ISP advertising).
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Display values in bytes instead of bits per second.\nThe default uses bits (standard for ISP advertising)."
    )]
    pub bytes: Option<bool>,

    /// Suppress verbose output, only show basic information
    ///
    /// Basic information = one-line summary: latency, download, upload.
    #[deprecated(since = "0.9.0", note = "Use --format simple instead")]
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Suppress verbose output, only show basic information.\nBasic information = one-line summary: latency, download, upload.\nDeprecated: use --format simple instead."
    )]
    pub simple: Option<bool>,

    /// Output in CSV format
    ///
    /// CSV output is suitable for spreadsheet analysis.
    /// Use --csv-header to include column names.
    #[deprecated(since = "0.9.0", note = "Use --format csv instead")]
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Output in CSV format for spreadsheet analysis.\nDeprecated: use --format csv instead."
    )]
    pub csv: Option<bool>,

    /// Single character delimiter for CSV output (default: ",")
    #[arg(long, default_value = ",", value_parser = validate_csv_delimiter)]
    pub csv_delimiter: char,

    /// Print CSV headers
    #[arg(long, action = ArgAction::Set, default_missing_value = "true", num_args = 0..=1)]
    pub csv_header: Option<bool>,

    /// Output in JSON format
    ///
    /// JSON output is machine-readable and includes all measured values.
    #[deprecated(since = "0.9.0", note = "Use --format json instead")]
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Output in JSON format (machine-readable).\nDeprecated: use --format json instead."
    )]
    pub json: Option<bool>,

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
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Suppress all progress output during the test.\nJSON/CSV output still goes to stdout."
    )]
    pub quiet: Option<bool>,

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
    #[arg(long, action = ArgAction::Set, default_missing_value = "true", num_args = 0..=1)]
    pub minimal: Option<bool>,

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

    /// Enable strict config mode - show warnings for invalid config values
    #[arg(long, action = ArgAction::Set, default_missing_value = "true", num_args = 0..=1)]
    pub strict_config: Option<bool>,

    /// Path to a custom CA certificate file (PEM/DER format)
    ///
    /// When specified, the client will use this certificate for TLS verification
    /// instead of the system default certificates.
    #[arg(long, value_name = "PATH", value_parser = validate_ca_cert_path, long_help = "Path to a custom CA certificate file (PEM/DER format).\nWhen specified, the client uses this certificate for TLS verification\ninstead of the system default certificates.")]
    pub ca_cert: Option<String>,

    /// Minimum TLS version to use (1.2 or 1.3)
    ///
    /// The default allows both TLS 1.2 and 1.3. Use this to restrict
    /// connections to a specific TLS version for testing or compliance.
    #[arg(long, value_name = "VERSION", value_parser = validate_tls_version, long_help = "Minimum TLS version to use (1.2 or 1.3).\nThe default allows both TLS 1.2 and 1.3.\nUse this to restrict connections to a specific TLS version.")]
    pub tls_version: Option<String>,

    /// Enable certificate pinning for speedtest.net servers
    ///
    /// When enabled, the client only accepts connections to speedtest.net
    /// and ookla.com domains. This provides some protection against MITM attacks
    /// but does NOT verify actual certificate hashes (domain-only pinning).
    #[arg(
        long,
        action = ArgAction::Set,
        default_missing_value = "true",
        num_args = 0..=1,
        long_help = "Enable certificate pinning for speedtest.net servers.\nWhen enabled, the client only accepts connections to speedtest.net\nand ookla.com domains.\n\n⚠ SECURITY LIMITATION: This is DOMAIN-ONLY pinning. It only validates\nthat the server hostname ends with .speedtest.net or .ookla.com.\nIt does NOT verify certificate hashes or the certificate chain.\nAn attacker with a valid certificate from any CA for these domains\ncould still perform a man-in-the-middle (MITM) attack.\n\nFor production security, use a custom CA certificate (--ca-cert) instead."
    )]
    pub pin_certs: Option<bool>,
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

fn validate_tls_version(s: &str) -> Result<String, String> {
    let normalized = s.to_lowercase();
    if normalized == "1.2" || normalized == "1.3" {
        Ok(normalized)
    } else {
        Err("TLS version must be '1.2' or '1.3'".to_string())
    }
}

fn validate_ca_cert_path(s: &str) -> Result<String, String> {
    let path = std::path::Path::new(s);
    if !path.exists() {
        return Err(format!(
            "CA certificate file not found: {s}\nUse --pin-certs for domain-only pinning instead."
        ));
    }
    if !path.is_file() {
        return Err(format!(
            "CA certificate path is not a file: {s}\nUse --pin-certs for domain-only pinning instead."
        ));
    }
    Ok(s.to_string())
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
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

    #[test]
    fn test_validate_tls_version_valid_12() {
        assert_eq!(validate_tls_version("1.2"), Ok("1.2".to_string()));
    }

    #[test]
    fn test_validate_tls_version_valid_13() {
        assert_eq!(validate_tls_version("1.3"), Ok("1.3".to_string()));
    }

    #[test]
    fn test_validate_tls_version_case_insensitive() {
        assert_eq!(validate_tls_version("1.2"), Ok("1.2".to_string()));
        assert_eq!(validate_tls_version("1.3"), Ok("1.3".to_string()));
    }

    #[test]
    fn test_validate_tls_version_invalid() {
        assert!(validate_tls_version("1.1").is_err());
        assert!(validate_tls_version("2.0").is_err());
        assert!(validate_tls_version("TLS1.2").is_err());
        assert!(validate_tls_version("").is_err());
    }

    #[test]
    fn test_validate_ca_cert_path_valid() {
        // Create a temp file to test the success path
        let temp_dir = std::env::temp_dir();
        let cert_path = temp_dir.join("test_ca_cert_validate.pem");
        std::fs::write(&cert_path, "dummy cert content").ok();

        let result = validate_ca_cert_path(cert_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cert_path.to_str().unwrap());

        // Clean up
        std::fs::remove_file(&cert_path).ok();
    }

    #[test]
    fn test_validate_ca_cert_path_not_found() {
        let result = validate_ca_cert_path("/nonexistent/path/to/cert.pem");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("not found"));
        assert!(err.contains("/nonexistent/path/to/cert.pem"));
        assert!(err.contains("--pin-certs")); // Suggest alternative
    }

    #[test]
    fn test_validate_ca_cert_path_is_directory() {
        // Use /tmp which should exist as a directory
        let result = validate_ca_cert_path("/tmp");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("not a file"));
    }
}
