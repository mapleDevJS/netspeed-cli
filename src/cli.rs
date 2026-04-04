use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "netspeed-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Command line interface for testing internet bandwidth using speedtest.net")]
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

    /// Generate and provide a URL to the speedtest.net share results image
    #[arg(long)]
    pub share: bool,

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

    /// Display a list of speedtest.net servers sorted by distance
    #[arg(long)]
    pub list: bool,

    /// Specify a server ID to test against (can be supplied multiple times)
    #[arg(long)]
    pub server: Vec<String>,

    /// Exclude a server from selection (can be supplied multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// URL of the Speedtest Mini server
    #[arg(long, value_parser = validate_url)]
    pub mini: Option<String>,

    /// Source IP address to bind to
    #[arg(long, value_parser = validate_ip_address)]
    pub source: Option<String>,

    /// HTTP timeout in seconds (default: 10)
    #[arg(long, default_value = "10", value_parser = validate_timeout)]
    pub timeout: u64,

    /// Use HTTPS instead of HTTP
    #[arg(long)]
    pub secure: bool,

    /// Do not pre-allocate upload data
    #[arg(long)]
    pub no_pre_allocate: bool,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    pub generate_completion: Option<ShellType>,
}

fn validate_csv_delimiter(s: &str) -> Result<char, String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() != 1 {
        return Err("CSV delimiter must be a single character".to_string());
    }
    
    let delimiter = chars[0];
    if !",;|\\t".contains(delimiter) {
        return Err(format!("Invalid CSV delimiter '{}'. Must be one of: comma, semicolon, pipe, or tab", delimiter));
    }
    
    Ok(delimiter)
}

fn validate_url(s: &str) -> Result<String, String> {
    if !s.starts_with("http://") && !s.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }
    Ok(s.to_string())
}

fn validate_ip_address(s: &str) -> Result<String, String> {
    // Simple IPv4 validation
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid IP address format: '{}'. Expected format: x.x.x.x", s));
    }
    
    for part in parts {
        match part.parse::<u8>() {
            Ok(_) => (),
            Err(_) => return Err(format!("Invalid IP address octet: '{}'", part)),
        }
    }
    
    Ok(s.to_string())
}

fn validate_timeout(s: &str) -> Result<u64, String> {
    let timeout: u64 = s.parse().map_err(|_| format!("Invalid timeout value: '{}'", s))?;
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
    fn test_validate_url_http() {
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_https() {
        assert!(validate_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_no_protocol() {
        assert!(validate_url("example.com").is_err());
    }

    #[test]
    fn test_validate_url_invalid_ftp() {
        assert!(validate_url("ftp://example.com").is_err());
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
