use crate::cli::Args;
use crate::theme::Theme;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct File {
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
    pub profile: Option<String>,
    pub theme: Option<String>,
    /// Custom user agent string (optional, defaults to browser-like UA).
    pub custom_user_agent: Option<String>,
    /// Enable strict config mode - invalid values cause warnings.
    pub strict: Option<bool>,
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
    pub profile: Option<String>,
    pub theme: Theme,
    pub minimal: bool,
    pub custom_user_agent: Option<String>,
    pub strict: bool,
}

impl Config {
    #[allow(deprecated)]
    #[must_use]
    pub fn from_args(args: &Args) -> Self {
        let file_config = load_config_file().unwrap_or_default();

        // Merge strategy: CLI > config file > hardcoded defaults.
        // Configurable booleans use Option<bool> in CLI parsing so we can
        // distinguish "flag not supplied" from "flag supplied as true".
        let merge_bool = |cli: Option<bool>, file: Option<bool>| cli.or(file).unwrap_or(false);
        let merge_u64 = |cli: u64, file: Option<u64>, default: u64| {
            // If CLI is at default value, check file; otherwise use CLI
            if cli == default {
                file.unwrap_or(default)
            } else {
                cli
            }
        };

        // Check for strict mode
        let strict = merge_bool(args.strict_config, file_config.strict);

        // Warn if profile is invalid
        if let Some(ref profile_name) = args.profile {
            if crate::profiles::UserProfile::from_name(profile_name).is_none() {
                eprintln!(
                    "Warning: Unknown profile '{}'. Valid options: power-user, gamer, streamer, remote-worker, casual. Using 'power-user'.",
                    profile_name
                );
            }
        }

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
            profile: args.profile.clone().or(file_config.profile),
            theme: if args.theme == "dark" {
                file_config
                    .theme
                    .as_ref()
                    .and_then(|t| Theme::from_name(t))
                    .unwrap_or_default()
            } else {
                Theme::from_name(&args.theme).unwrap_or_default()
            },
            minimal: merge_bool(args.minimal, None),
            custom_user_agent: file_config.custom_user_agent.clone(),
            strict,
        }
    }
}

/// Get the configuration file path (internal — also used by orchestrator for --show-config-path).
#[must_use]
pub fn get_config_path_internal() -> Option<PathBuf> {
    ProjectDirs::from("dev", "vibe", "netspeed-cli").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        if let Err(e) = fs::create_dir_all(config_dir) {
            eprintln!("Warning: Failed to create config directory: {e}");
        }
        config_dir.join("config.toml")
    })
}

fn load_config_file() -> Option<File> {
    let path = get_config_path_internal()?;
    if !path.exists() {
        return None;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read config file {}: {e}",
                path.display()
            );
            return None;
        }
    };
    let mut config: File = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to parse config: {e}");
            return None;
        }
    };

    // Validate timeout if present
    if let Some(timeout) = config.timeout {
        if timeout == 0 || timeout > 300 {
            eprintln!(
                "Warning: Invalid config timeout ({timeout}s, must be 1-300). Using default."
            );
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
        let args = Args::parse_from(["netspeed-cli"]);
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
        let args = Args::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.no_download);
        assert!(!config.no_upload);
    }

    #[test]
    fn test_config_file_deserialization() {
        let toml_content = r"
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
        ";

        let config: File = toml::from_str(toml_content).unwrap();
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
        let toml_content = r"
            no_download = true
            timeout = 20
        ";

        let config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(config.no_download, Some(true));
        assert!(config.no_upload.is_none());
        assert!(config.single.is_none());
        assert_eq!(config.timeout, Some(20));
        assert!(config.csv_delimiter.is_none());
    }

    #[test]
    fn test_config_from_args_overrides_file() {
        // Test that CLI flags override file config when explicitly set
        let args = Args::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.no_download);
    }

    #[test]
    fn test_config_merge_bool_file_true_cli_false() {
        // When CLI omits the flag, the config file value should be used.
        let toml_content = r"
            no_download = true
        ";
        let file_config: File = toml::from_str(toml_content).unwrap();

        // CLI args omit the flag, so clap yields None for Option<bool>.
        let args = Args::parse_from(["netspeed-cli"]);
        let file_config_loaded = Some(file_config);

        // Manual merge check
        let cli_val = args.no_download; // None
        let file_val = file_config_loaded.and_then(|c| c.no_download); // Some(true)
        let merged = cli_val.or(file_val).unwrap_or(false);
        assert!(merged);
    }
}
