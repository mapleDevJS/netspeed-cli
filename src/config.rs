use crate::cli::CliArgs;

/// Application configuration derived from CLI arguments.
///
/// This struct normalizes and validates CLI args into a consistent config
/// that the rest of the application uses.
pub struct Config {
    /// Skip download speed test
    pub no_download: bool,
    /// Skip upload speed test
    pub no_upload: bool,
    /// Use single connection instead of multiple concurrent streams
    pub single: bool,
    /// Display values in bytes instead of bits
    pub bytes: bool,
    /// Generate shareable results URL
    pub share: bool,
    /// Suppress verbose output
    pub simple: bool,
    /// Enable verbose/debug logging
    pub verbose: bool,
    /// Output results in CSV format
    pub csv: bool,
    /// Delimiter character for CSV output
    pub csv_delimiter: char,
    /// Include header row in CSV output
    pub csv_header: bool,
    /// Output results in JSON format
    pub json: bool,
    /// List available servers instead of running tests
    pub list: bool,
    /// Specific server IDs to test against
    pub server_ids: Vec<String>,
    /// Server IDs to exclude from selection
    pub exclude_ids: Vec<String>,
    /// URL of a Speedtest Mini server (if specified)
    #[allow(dead_code)]
    pub mini_url: Option<String>,
    /// Source IP address to bind to
    pub source: Option<String>,
    /// HTTP request timeout in seconds
    pub timeout: u64,
    /// Use HTTPS instead of HTTP
    pub secure: bool,
    /// Don't pre-allocate upload data (currently unused)
    #[allow(dead_code)]
    pub no_pre_allocate: bool,
}

impl Config {
    /// Create configuration from parsed CLI arguments.
    pub fn from_args(args: &CliArgs) -> Self {
        Self {
            no_download: args.no_download,
            no_upload: args.no_upload,
            single: args.single,
            bytes: args.bytes,
            share: args.share,
            simple: args.simple,
            verbose: args.verbose,
            csv: args.csv,
            csv_delimiter: args.csv_delimiter,
            csv_header: args.csv_header,
            json: args.json,
            list: args.list,
            server_ids: args.server.clone(),
            exclude_ids: args.exclude.clone(),
            mini_url: args.mini.clone(),
            source: args.source.clone(),
            timeout: args.timeout,
            secure: args.secure,
            no_pre_allocate: args.no_pre_allocate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_default_args() {
        let args = CliArgs::default();
        let config = Config::from_args(&args);

        assert!(!config.no_download);
        assert!(!config.no_upload);
        assert!(!config.single);
        assert!(!config.bytes);
        assert!(!config.share);
        assert!(!config.simple);
        assert!(!config.verbose);
        assert!(!config.csv);
        // CliArgs::default() gives '\0' for char, but clap sets it to ','
        // We test the mapping behavior, not the default value
        assert!(!config.csv_header);
        assert!(!config.json);
        assert!(!config.list);
        assert!(config.server_ids.is_empty());
        assert!(config.exclude_ids.is_empty());
        assert!(config.mini_url.is_none());
        assert!(config.source.is_none());
        // CliArgs::default() gives 0 for timeout, but clap sets it to 10
        // We just verify the mapping works, not the clap default
        assert!(!config.secure);
        // CliArgs::default() gives false for no_pre_allocate, but clap sets it to true
    }

    #[test]
    fn test_config_with_custom_flags() {
        let args = CliArgs {
            no_download: true, no_upload: true, single: true, bytes: true, share: true,
            simple: true, verbose: true, csv: true, csv_delimiter: ';', csv_header: true,
            json: false, list: true, timeout: 30, secure: true, no_pre_allocate: false,
            ..Default::default()
        };

        let config = Config::from_args(&args);

        assert!(config.no_download);
        assert!(config.no_upload);
        assert!(config.single);
        assert!(config.bytes);
        assert!(config.share);
        assert!(config.simple);
        assert!(config.verbose);
        assert!(config.csv);
        assert_eq!(config.csv_delimiter, ';');
        assert!(config.csv_header);
        assert!(!config.json);
        assert!(config.list);
        assert_eq!(config.timeout, 30);
        assert!(config.secure);
        assert!(!config.no_pre_allocate);
    }

    #[test]
    fn test_config_server_ids_mapping() {
        let args = CliArgs { server: vec!["123".to_string(), "456".to_string()], ..Default::default() };
        let config = Config::from_args(&args);

        assert_eq!(config.server_ids, vec!["123", "456"]);
    }

    #[test]
    fn test_config_exclude_ids_mapping() {
        let args = CliArgs { exclude: vec!["789".to_string()], ..Default::default() };
        let config = Config::from_args(&args);

        assert_eq!(config.exclude_ids, vec!["789"]);
    }

    #[test]
    fn test_config_csv_delimiter_mapping() {
        let args = CliArgs { csv: true, csv_delimiter: '\t', ..Default::default() };
        let config = Config::from_args(&args);

        assert_eq!(config.csv_delimiter, '\t');
    }

    #[test]
    fn test_config_csv_header_mapping() {
        let args = CliArgs { csv: true, csv_header: true, ..Default::default() };
        let config = Config::from_args(&args);

        assert!(config.csv_header);
    }

    #[test]
    fn test_config_mini_url_mapping() {
        let args = CliArgs { mini: Some("http://mini.example.com".to_string()), ..Default::default() };
        let config = Config::from_args(&args);

        assert_eq!(config.mini_url, Some("http://mini.example.com".to_string()));
    }

    #[test]
    fn test_config_source_mapping() {
        let args = CliArgs { source: Some("192.168.1.1".to_string()), ..Default::default() };
        let config = Config::from_args(&args);

        assert_eq!(config.source, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_config_json_flag_mapping() {
        let args = CliArgs { json: true, ..Default::default() };
        let config = Config::from_args(&args);

        assert!(config.json);
        assert!(!config.csv);
    }

    #[test]
    fn test_config_secure_mapping() {
        let args = CliArgs { secure: true, ..Default::default() };
        let config = Config::from_args(&args);

        assert!(config.secure);
    }
}
