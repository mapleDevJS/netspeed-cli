use crate::cli::CliArgs;

pub struct Config {
    pub no_download: bool,
    pub no_upload: bool,
    pub single: bool,
    pub bytes: bool,
    pub share: bool,
    pub simple: bool,
    pub csv: bool,
    pub csv_delimiter: char,
    pub csv_header: bool,
    pub json: bool,
    pub list: bool,
    pub server_ids: Vec<String>,
    pub exclude_ids: Vec<String>,
    #[allow(dead_code)]
    pub mini_url: Option<String>,
    #[allow(dead_code)]
    pub source: Option<String>,
    pub timeout: u64,
    #[allow(dead_code)]
    pub secure: bool,
    #[allow(dead_code)]
    pub no_pre_allocate: bool,
    #[allow(dead_code)]
    pub client_ip: Option<String>,
}

impl Config {
    pub fn from_args(args: &CliArgs) -> Self {
        Self {
            no_download: args.no_download,
            no_upload: args.no_upload,
            single: args.single,
            bytes: args.bytes,
            share: args.share,
            simple: args.simple,
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
            client_ip: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_config_from_args_defaults() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);

        assert!(!config.no_download);
        assert!(!config.no_upload);
        assert!(!config.single);
        assert!(!config.bytes);
        assert!(!config.share);
        assert!(!config.simple);
        assert!(!config.csv);
        assert!(!config.json);
        assert!(!config.list);
        assert_eq!(config.timeout, 10);
        assert!(!config.secure);
        assert_eq!(config.csv_delimiter, ',');
        assert!(!config.csv_header);
        assert!(config.server_ids.is_empty());
        assert!(config.exclude_ids.is_empty());
    }

    #[test]
    fn test_config_from_args_no_download() {
        let args = CliArgs::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.no_download);
        assert!(!config.no_upload);
    }

    #[test]
    fn test_config_from_args_no_upload() {
        let args = CliArgs::parse_from(["netspeed-cli", "--no-upload"]);
        let config = Config::from_args(&args);
        assert!(!config.no_download);
        assert!(config.no_upload);
    }

    #[test]
    fn test_config_from_args_single() {
        let args = CliArgs::parse_from(["netspeed-cli", "--single"]);
        let config = Config::from_args(&args);
        assert!(config.single);
    }

    #[test]
    fn test_config_from_args_bytes() {
        let args = CliArgs::parse_from(["netspeed-cli", "--bytes"]);
        let config = Config::from_args(&args);
        assert!(config.bytes);
    }

    #[test]
    fn test_config_from_args_json() {
        let args = CliArgs::parse_from(["netspeed-cli", "--json"]);
        let config = Config::from_args(&args);
        assert!(config.json);
    }

    #[test]
    fn test_config_from_args_csv_with_delimiter() {
        let args = CliArgs::parse_from(["netspeed-cli", "--csv", "--csv-delimiter", ";"]);
        let config = Config::from_args(&args);
        assert!(config.csv);
        assert_eq!(config.csv_delimiter, ';');
    }

    #[test]
    fn test_config_from_args_server_ids() {
        let args = CliArgs::parse_from([
            "netspeed-cli",
            "--server",
            "1234",
            "--server",
            "5678",
        ]);
        let config = Config::from_args(&args);
        assert_eq!(config.server_ids, vec!["1234", "5678"]);
    }

    #[test]
    fn test_config_from_args_exclude_ids() {
        let args = CliArgs::parse_from([
            "netspeed-cli",
            "--exclude",
            "9999",
        ]);
        let config = Config::from_args(&args);
        assert_eq!(config.exclude_ids, vec!["9999"]);
    }

    #[test]
    fn test_config_from_args_timeout() {
        let args = CliArgs::parse_from(["netspeed-cli", "--timeout", "30"]);
        let config = Config::from_args(&args);
        assert_eq!(config.timeout, 30);
    }

    #[test]
    fn test_config_from_args_secure() {
        let args = CliArgs::parse_from(["netspeed-cli", "--secure"]);
        let config = Config::from_args(&args);
        assert!(config.secure);
    }

    #[test]
    fn test_config_from_args_mini_url() {
        let args = CliArgs::parse_from(["netspeed-cli", "--mini", "http://mini.example.com"]);
        let config = Config::from_args(&args);
        assert_eq!(config.mini_url, Some("http://mini.example.com".to_string()));
    }

    #[test]
    fn test_config_from_args_source() {
        let args = CliArgs::parse_from(["netspeed-cli", "--source", "192.168.1.100"]);
        let config = Config::from_args(&args);
        assert_eq!(config.source, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_config_with_client_ip() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args).with_client_ip("10.0.0.1".to_string());
        assert_eq!(config.client_ip, Some("10.0.0.1".to_string()));
    }
}

