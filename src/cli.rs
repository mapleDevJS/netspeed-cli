use clap::{Parser, ValueEnum};

// Shared validation functions (also used by build.rs via include!)
include!("validate.rs");

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(name = "netspeed-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Command line interface for testing internet bandwidth using speedtest.net")]
#[command(after_help = "\
Examples:
  netspeed-cli                          Run a full speed test
  netspeed-cli --simple                 Run with minimal output
  netspeed-cli --json                   Output results as JSON
  netspeed-cli --list                   List available servers
  netspeed-cli --server 1234            Test against a specific server
  netspeed-cli --no-upload              Skip upload test
  netspeed-cli --bytes                  Show results in MB/s instead of Mbit/s
  netspeed-cli --single                 Use a single connection (debugging)
  netspeed-cli --generate-completion zsh > ~/.zsh/functions/_netspeed-cli
                                        Generate Zsh shell completions
")]
pub struct CliArgs {
    /// Do not perform download test
    #[arg(long)]
    pub no_download: bool,

    /// Do not perform upload test
    #[arg(long)]
    pub no_upload: bool,

    /// Only use a single connection instead of multiple
    #[arg(long)]
    pub single: bool,

    /// Display values in bytes instead of bits
    #[arg(long)]
    pub bytes: bool,

    /// Suppress verbose output, only show basic information
    #[arg(long)]
    pub simple: bool,

    /// Output in CSV format
    #[arg(long)]
    pub csv: bool,

    /// Single character delimiter for CSV output (default: ",")
    #[arg(long, default_value = ",", value_parser = validate_csv_delimiter)]
    pub csv_delimiter: char,

    /// Print CSV headers
    #[arg(long)]
    pub csv_header: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Output format (supersedes --json, --csv, --simple)
    #[arg(long, value_enum)]
    pub format: Option<OutputFormatType>,

    /// Display a list of speedtest.net servers sorted by distance
    #[arg(long)]
    pub list: bool,

    /// Specify a server ID to test against (can be supplied multiple times)
    #[arg(long)]
    pub server: Vec<String>,

    /// Exclude a server from selection (can be supplied multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Source IP address to bind to
    #[arg(long, value_parser = validate_ip_address)]
    pub source: Option<String>,

    /// HTTP timeout in seconds (default: 10)
    #[arg(long, default_value = "10", value_parser = validate_timeout)]
    pub timeout: u64,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    pub generate_completion: Option<ShellType>,

    /// Display test history
    #[arg(long)]
    pub history: bool,
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
    PowerShell,
    Elvish,
}

/// Unified output format selection (supersedes --json, --csv, --simple).
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormatType {
    Json,
    Csv,
    Simple,
    Detailed,
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
