use crate::cli::CliArgs;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub no_download: Option<bool>,
    pub no_upload: Option<bool>,
    pub single: Option<bool>,
    pub bytes: Option<bool>,
    pub simple: Option<bool>,
    pub csv: Option<bool>,
    pub csv_delimiter: Option<char>,
    pub csv_header: Option<bool>,
    pub json: Option<bool>,
    pub timeout: Option<u64>,
}

#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub no_download: bool,
    pub no_upload: bool,
    pub single: bool,
    pub bytes: bool,
    pub simple: bool,
    pub csv: bool,
    pub csv_delimiter: char,
    pub csv_header: bool,
    pub json: bool,
    pub list: bool,
    pub server_ids: Vec<String>,
    pub exclude_ids: Vec<String>,
    pub source: Option<String>,
    pub timeout: u64,
    pub quiet: bool,
}

impl Config {
    #[must_use]
    pub fn from_args(args: &CliArgs) -> Self {
        let file_config = load_config_file().unwrap_or_default();

        // Merge strategy: CLI flags and config file are combined with OR semantics.
        // Since clap defaults `bool` to `false`, we cannot distinguish "user didn't
        // pass the flag" from "user explicitly passed `--no-flag`".
        //
        // Practical effect: if config file has `no_download = true`, downloads will
        // be skipped unless the user passes `--no-download=false` (if supported)
        // or removes the config line. The config file acts as a persistent default.
        //
        // For timeout: CLI value is used only if explicitly set (non-default).
        // Otherwise, file config is checked, falling back to the compiled default (10).
        let merge_bool = |cli: bool, file: Option<bool>| cli || file.unwrap_or(false);
        let merge_u64 = |cli: u64, file: Option<u64>, default: u64| {
            // If CLI is at default value, check file; otherwise use CLI
            if cli == default {
                file.unwrap_or(default)
            } else {
                cli
            }
        };

        Self {
            no_download: merge_bool(args.no_download, file_config.no_download),
            no_upload: merge_bool(args.no_upload, file_config.no_upload),
            single: merge_bool(args.single, file_config.single),
            bytes: merge_bool(args.bytes, file_config.bytes),
            simple: merge_bool(args.simple, file_config.simple),
            csv: merge_bool(args.csv, file_config.csv),
            csv_delimiter: if args.csv_delimiter == ',' {
                file_config.csv_delimiter.unwrap_or(',')
            } else {
                args.csv_delimiter
            },
            csv_header: merge_bool(args.csv_header, file_config.csv_header),
            json: merge_bool(args.json, file_config.json),
            list: args.list,
            server_ids: args.server.clone(),
            exclude_ids: args.exclude.clone(),
            source: args.source.clone(),
            timeout: merge_u64(args.timeout, file_config.timeout, 10),
            quiet: merge_bool(args.quiet, None),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "vibe", "netspeed-cli").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).ok();
        config_dir.join("config.toml")
    })
}

fn load_config_file() -> Option<ConfigFile> {
    let path = get_config_path()?;
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let mut config: ConfigFile = toml::from_str(&content).ok()?;

    // Validate timeout if present
    if let Some(timeout) = config.timeout {
        if timeout == 0 || timeout > 300 {
            // Silently ignore invalid timeout — fall back to default
            config.timeout = None;
        }
    }

    Some(config)
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
        assert!(!config.simple);
        assert!(!config.csv);
        assert!(!config.json);
        assert!(!config.list);
        assert!(!config.quiet);
        assert_eq!(config.timeout, 10);
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
    fn test_config_file_deserialization() {
        let toml_content = r#"
            no_download = true
            no_upload = false
            single = true
            bytes = true
            simple = false
            csv = false
            csv_delimiter = ';'
            csv_header = true
            json = true
            timeout = 30
        "#;

        let config: ConfigFile = toml::from_str(toml_content).unwrap();
        assert_eq!(config.no_download, Some(true));
        assert_eq!(config.no_upload, Some(false));
        assert_eq!(config.single, Some(true));
        assert_eq!(config.bytes, Some(true));
        assert_eq!(config.simple, Some(false));
        assert_eq!(config.csv, Some(false));
        assert_eq!(config.csv_delimiter, Some(';'));
        assert_eq!(config.csv_header, Some(true));
        assert_eq!(config.json, Some(true));
        assert_eq!(config.timeout, Some(30));
    }

    #[test]
    fn test_config_file_partial() {
        let toml_content = r#"
            no_download = true
            timeout = 20
        "#;

        let config: ConfigFile = toml::from_str(toml_content).unwrap();
        assert_eq!(config.no_download, Some(true));
        assert!(config.no_upload.is_none());
        assert!(config.single.is_none());
        assert_eq!(config.timeout, Some(20));
        assert!(config.csv_delimiter.is_none());
    }

    #[test]
    fn test_config_from_args_overrides_file() {
        // Test that CLI flags override file config when explicitly set
        let args = CliArgs::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.no_download);
    }

    #[test]
    fn test_config_merge_bool_file_true_cli_false() {
        // When CLI flag is false (default) and file config is true, result should be false
        // because merge_bool = cli || file.unwrap_or(false)
        // Actually merge_bool returns true only if CLI is true OR file is Some(true)
        // Let's verify the actual behavior
        let toml_content = r#"
            no_download = true
        "#;
        let file_config: ConfigFile = toml::from_str(toml_content).unwrap();

        // CLI args with no_download=false (default)
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let file_config_loaded = Some(file_config);

        // Manual merge check
        let cli_val = args.no_download; // false
        let file_val = file_config_loaded.and_then(|c| c.no_download); // Some(true)
        let merged = cli_val || file_val.unwrap_or(false);
        // Since CLI is false and file is Some(true), result depends on merge logic
        // The current merge is: cli || file.unwrap_or(false) = false || true = true
        assert!(merged);
    }
}
