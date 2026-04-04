// Integration tests for netspeed-cli configuration parsing

#[cfg(test)]
mod tests {
    use netspeed_cli::error::SpeedtestError;

    #[test]
    fn test_error_display_network() {
        let err = SpeedtestError::NetworkError("connection refused".to_string());
        assert_eq!(format!("{}", err), "Network error: connection refused");
    }

    #[test]
    fn test_error_display_server_not_found() {
        let err = SpeedtestError::ServerNotFound("no servers".to_string());
        assert_eq!(format!("{}", err), "Server not found: no servers");
    }

    #[test]
    fn test_error_display_parse() {
        let err = SpeedtestError::ParseError("invalid XML".to_string());
        assert_eq!(format!("{}", err), "Parse error: invalid XML");
    }

    #[test]
    fn test_config_from_args_defaults() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;
        use netspeed_cli::config::Config;

        // Parse with no args (all defaults)
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);

        assert!(!config.no_download);
        assert!(!config.no_upload);
        assert!(!config.single);
        assert!(!config.bytes);
        assert!(!config.share);
        assert!(!config.simple);
        assert!(!config.json);
        assert!(!config.csv);
        assert_eq!(config.timeout, 10);
        assert!(!config.secure);
    }

    #[test]
    fn test_config_from_args_custom() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;
        use netspeed_cli::config::Config;

        let args = CliArgs::parse_from([
            "netspeed-cli",
            "--no-download",
            "--no-upload",
            "--single",
            "--bytes",
            "--json",
            "--timeout",
            "30",
            "--secure",
        ]);
        let config = Config::from_args(&args);

        assert!(config.no_download);
        assert!(config.no_upload);
        assert!(config.single);
        assert!(config.bytes);
        assert!(config.json);
        assert_eq!(config.timeout, 30);
        assert!(config.secure);
    }

    #[test]
    fn test_config_server_ids() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;
        use netspeed_cli::config::Config;

        let args = CliArgs::parse_from([
            "netspeed-cli",
            "--server",
            "1234",
            "--server",
            "5678",
            "--exclude",
            "9999",
        ]);
        let config = Config::from_args(&args);

        assert_eq!(config.server_ids, vec!["1234", "5678"]);
        assert_eq!(config.exclude_ids, vec!["9999"]);
    }

    #[test]
    fn test_config_json_and_csv_conflict() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;

        let result = CliArgs::try_parse_from(["netspeed-cli", "--json", "--csv"]);

        assert!(result.is_err(), "--json and --csv should conflict");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("cannot be used with"),
            "Error should mention conflict: {}",
            err
        );
    }

    #[test]
    fn test_config_csv_header_requires_csv() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;

        let result = CliArgs::try_parse_from(["netspeed-cli", "--csv-header"]);

        assert!(result.is_err(), "--csv-header requires --csv");
    }

    #[test]
    fn test_config_csv_delimiter_requires_csv() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;

        let result = CliArgs::try_parse_from(["netspeed-cli", "--csv-delimiter", ";"]);

        assert!(result.is_err(), "--csv-delimiter requires --csv");
    }

    #[test]
    fn test_config_csv_with_options() {
        use clap::Parser;
        use netspeed_cli::cli::CliArgs;
        use netspeed_cli::config::Config;

        let args = CliArgs::parse_from([
            "netspeed-cli",
            "--csv",
            "--csv-delimiter",
            ";",
            "--csv-header",
        ]);
        let config = Config::from_args(&args);

        assert!(config.csv);
        assert_eq!(config.csv_delimiter, ';');
        assert!(config.csv_header);
    }
}
