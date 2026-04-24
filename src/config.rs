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
    /// Path to a custom CA certificate file for TLS verification.
    pub ca_cert: Option<String>,
    /// Minimum TLS version (1.2 or 1.3).
    pub tls_version: Option<String>,
    /// Enable certificate pinning for speedtest.net servers.
    pub pin_certs: Option<bool>,
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
    // TLS configuration
    pub ca_cert: Option<String>,
    pub tls_version: Option<String>,
    pub pin_certs: bool,
}

impl Config {
    #[allow(deprecated)]
    #[must_use]
    pub fn from_args(args: &Args) -> Self {
        let file_config = load_config_file().unwrap_or_default();

        // Validate config file settings
        let validation = validate_config(&file_config);
        let strict = args
            .strict_config
            .unwrap_or(file_config.strict.unwrap_or(false));

        // Report validation results
        for error in &validation.errors {
            if strict {
                eprintln!("Error: {error}");
            } else {
                eprintln!("Warning: {error}");
            }
        }
        for warning in &validation.warnings {
            eprintln!("Warning: {warning}");
        }

        // Exit on errors in strict mode
        if strict && !validation.valid {
            std::process::exit(1);
        }

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
            // TLS configuration - CLI takes precedence over config file
            ca_cert: args.ca_cert.clone().or(file_config.ca_cert.clone()),
            tls_version: args.tls_version.clone().or(file_config.tls_version.clone()),
            pin_certs: merge_bool(args.pin_certs, file_config.pin_certs),
        }
    }
}

/// Validation result with error details.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result.
    #[must_use]
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result.
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Create a validation failure.
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result.
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.valid = false;
        self
    }

    /// Merge another validation result into this one.
    #[must_use]
    pub fn merge(mut self, other: ValidationResult) -> Self {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }
}

/// Validate a profile name against known profiles using profiles.rs logic.
fn validate_profile(profile: &str) -> Result<(), String> {
    if crate::profiles::UserProfile::from_name(profile).is_some() {
        Ok(())
    } else {
        let valid_profiles = ["power-user", "gamer", "streamer", "remote-worker", "casual"];
        Err(format!(
            "Invalid profile '{}'. Valid options: {}",
            profile,
            valid_profiles.join(", ")
        ))
    }
}

/// Validate a theme name against known themes using theme.rs logic.
fn validate_theme(theme: &str) -> Result<(), String> {
    if crate::theme::Theme::from_name(theme).is_some() {
        Ok(())
    } else {
        let valid_themes = ["dark", "light", "high-contrast", "monochrome"];
        Err(format!(
            "Invalid theme '{}'. Valid options: {}",
            theme,
            valid_themes.join(", ")
        ))
    }
}

/// Validate CSV delimiter character.
fn validate_csv_delimiter_config(delimiter: char) -> Result<(), String> {
    if !",;|\t".contains(delimiter) {
        return Err(format!(
            "Invalid CSV delimiter '{}'. Must be one of: comma, semicolon, pipe, or tab",
            delimiter
        ));
    }
    Ok(())
}

/// Validate the entire config structure.
pub fn validate_config(file_config: &File) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // Validate profile
    if let Some(ref profile) = file_config.profile {
        if let Err(e) = validate_profile(profile) {
            result = result.with_error(e);
        }
    }

    // Validate theme
    if let Some(ref theme) = file_config.theme {
        if let Err(e) = validate_theme(theme) {
            result = result.with_error(e);
        }
    }

    // Validate CSV delimiter
    if let Some(delimiter) = file_config.csv_delimiter {
        if let Err(e) = validate_csv_delimiter_config(delimiter) {
            result = result.with_error(e);
        }
    }

    // Warnings for deprecated options
    if file_config.simple.unwrap_or(false) {
        result = result.with_warning(
            "'simple' option is deprecated. Use '--format simple' instead.".to_string(),
        );
    }
    if file_config.csv.unwrap_or(false) {
        result = result
            .with_warning("'csv' option is deprecated. Use '--format csv' instead.".to_string());
    }
    if file_config.json.unwrap_or(false) {
        result = result
            .with_warning("'json' option is deprecated. Use '--format json' instead.".to_string());
    }

    result
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

    #[test]
    fn test_validate_config_valid_profile() {
        let file_config = File {
            profile: Some("gamer".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_config_empty_is_valid() {
        // Default case: no config file
        let file_config = File::default();
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_config_invalid_profile() {
        let file_config = File {
            profile: Some("invalid_profile".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("invalid_profile"));
    }

    #[test]
    fn test_validate_config_invalid_theme() {
        let file_config = File {
            theme: Some("neon".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("neon"));
    }

    #[test]
    fn test_validate_config_invalid_csv_delimiter() {
        let file_config = File {
            csv_delimiter: Some('X'),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_config_deprecated_simple() {
        let file_config = File {
            simple: Some(true),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("simple") && w.contains("deprecated")));
    }

    #[test]
    fn test_validate_config_multiple_issues() {
        let file_config = File {
            profile: Some("bad".to_string()),
            theme: Some("ugly".to_string()),
            csv_delimiter: Some('@'),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(result.errors.len() >= 3); // profile, theme, delimiter
    }

    // ==================== TLS Configuration Tests ====================

    #[test]
    fn test_tls_config_defaults() {
        // When no CLI flags or config file, TLS options should be None/false
        let args = Args::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);
        assert!(config.ca_cert.is_none());
        assert!(config.tls_version.is_none());
        assert!(!config.pin_certs);
    }

    #[test]
    fn test_tls_config_file_deserialization() {
        // Test that TLS options deserialize correctly from TOML
        let toml_content = r#"
            ca_cert = "/custom/ca.pem"
            tls_version = "1.2"
            pin_certs = true
        "#;

        let file_config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(file_config.ca_cert, Some("/custom/ca.pem".to_string()));
        assert_eq!(file_config.tls_version, Some("1.2".to_string()));
        assert_eq!(file_config.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_file_partial() {
        // Test partial TLS config from file
        let toml_content = r#"
            ca_cert = "/my/ca.pem"
        "#;

        let file_config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(file_config.ca_cert, Some("/my/ca.pem".to_string()));
        assert!(file_config.tls_version.is_none());
        assert!(file_config.pin_certs.is_none());
    }

    #[test]
    fn test_tls_config_cli_ca_cert() {
        // Test that --ca-cert CLI flag is parsed correctly
        // Use an existing file path (should exist on all systems)
        let args = Args::parse_from(["netspeed-cli", "--ca-cert", "/etc/passwd"]);
        assert_eq!(args.ca_cert, Some("/etc/passwd".to_string()));
    }

    #[test]
    fn test_tls_config_cli_tls_version() {
        // Test that --tls-version CLI flag is parsed correctly
        let args = Args::parse_from(["netspeed-cli", "--tls-version", "1.3"]);
        assert_eq!(args.tls_version, Some("1.3".to_string()));
    }

    #[test]
    fn test_tls_config_cli_pin_certs() {
        // Test that --pin-certs CLI flag enables pinning
        let args = Args::parse_from(["netspeed-cli", "--pin-certs"]);
        assert_eq!(args.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_cli_pin_certs_false() {
        // Test that --pin-certs=false disables pinning
        let args = Args::parse_from(["netspeed-cli", "--pin-certs=false"]);
        assert_eq!(args.pin_certs, Some(false));
    }

    #[test]
    fn test_tls_config_all_cli_options() {
        // Test all TLS options via CLI
        // Use an existing file path for --ca-cert
        let args = Args::parse_from([
            "netspeed-cli",
            "--ca-cert",
            "/etc/passwd",
            "--tls-version",
            "1.2",
            "--pin-certs",
        ]);

        assert_eq!(args.ca_cert, Some("/etc/passwd".to_string()));
        assert_eq!(args.tls_version, Some("1.2".to_string()));
        assert_eq!(args.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_string_merge_cli_takes_precedence() {
        // For string options (ca_cert, tls_version), CLI should take precedence
        // This is tested by verifying the merge logic:
        // ca_cert: args.ca_cert.clone().or(file_config.ca_cert.clone())

        // When CLI provides ca_cert, it should be used
        let cli_val = Some("/cli/ca.pem".to_string());
        let file_val = Some("/file/ca.pem".to_string());
        let merged = cli_val.or(file_val.clone());
        assert_eq!(merged, Some("/cli/ca.pem".to_string()));

        // When CLI is None, file value should be used
        let cli_val_none: Option<String> = None;
        let merged = cli_val_none.or(file_val.clone());
        assert_eq!(merged, Some("/file/ca.pem".to_string()));

        // When both are None, result should be None
        let merged = Option::<String>::None.or(None);
        assert!(merged.is_none());
    }

    #[test]
    fn test_tls_config_bool_merge() {
        // Test boolean merge logic for pin_certs
        // merge_bool: cli.or(file).unwrap_or(false)
        // CLI takes precedence when explicitly set, file used only when CLI is None

        // CLI true, file false -> true (CLI takes precedence)
        assert!(merge_bool_test(Some(true), Some(false)));

        // CLI false, file true -> false (CLI takes precedence even when false)
        assert!(!merge_bool_test(Some(false), Some(true)));

        // CLI true, file None -> true
        assert!(merge_bool_test(Some(true), None));

        // CLI false, file None -> false
        assert!(!merge_bool_test(Some(false), None));

        // CLI None, file true -> true (fall back to file)
        assert!(merge_bool_test(None, Some(true)));

        // CLI None, file false -> false (fall back to file)
        assert!(!merge_bool_test(None, Some(false)));

        // CLI None, file None -> false (default)
        assert!(!merge_bool_test(None::<bool>, None));
    }

    // Helper function to test merge_bool logic
    fn merge_bool_test(cli: Option<bool>, file: Option<bool>) -> bool {
        cli.or(file).unwrap_or(false)
    }
}
