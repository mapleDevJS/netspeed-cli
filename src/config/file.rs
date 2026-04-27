//! Configuration file types and validation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration file format (YAML).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct File {
    #[serde(default)]
    pub output: Option<FileOutput>,
    #[serde(default)]
    pub test: Option<FileTest>,
    #[serde(default)]
    pub network: Option<FileNetwork>,
    #[serde(default)]
    pub server: Option<FileServer>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub bytes: Option<bool>,
    #[serde(default)]
    pub simple: Option<bool>,
    #[serde(default)]
    pub csv: Option<bool>,
    #[serde(default)]
    pub csv_delimiter: Option<char>,
    #[serde(default)]
    pub csv_header: Option<bool>,
    #[serde(default)]
    pub json: Option<bool>,
    #[serde(default)]
    pub no_download: Option<bool>,
    #[serde(default)]
    pub no_upload: Option<bool>,
    #[serde(default)]
    pub single: Option<bool>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub bind: Option<String>,
    #[serde(default)]
    pub ca_cert: Option<String>,
    #[serde(default)]
    pub tls_version: Option<String>,
    #[serde(default)]
    pub pin_certs: Option<bool>,
    #[serde(default)]
    pub insecure: Option<bool>,
    #[serde(default)]
    pub server_ids: Option<Vec<u64>>,
    #[serde(default)]
    pub exclude_ids: Option<Vec<u64>>,
}

/// File-level output config (subset of OutputSource for file parsing).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileOutput {
    pub bytes: Option<bool>,
    pub simple: Option<bool>,
    pub csv: Option<bool>,
    pub csv_delimiter: Option<char>,
    pub csv_header: Option<bool>,
    pub json: Option<bool>,
    pub minimal: Option<bool>,
    pub profile: Option<String>,
    pub theme: Option<String>,
    pub format: Option<super::Format>,
}

/// File-level test config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileTest {
    pub no_download: Option<bool>,
    pub no_upload: Option<bool>,
    pub single: Option<bool>,
    pub timeout: Option<u64>,
}

/// File-level network config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileNetwork {
    pub bind: Option<String>,
    pub timeout: Option<u64>,
    pub ca_cert: Option<String>,
    pub tls_version: Option<String>,
    pub pin_certs: Option<bool>,
    pub insecure: Option<bool>,
}

/// File-level server config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileServer {
    pub server_ids: Option<Vec<u64>>,
    pub exclude_ids: Option<Vec<u64>>,
}

/// Validation result for config files.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.valid = false;
        self.errors.push(error.into());
        self
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

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
        if let Err(e) = crate::profiles::UserProfile::validate(profile) {
            result = result.with_error(e);
        }
    }

    // Validate theme
    if let Some(ref theme) = file_config.theme {
        if let Err(e) = crate::theme::Theme::validate(theme) {
            result = result.with_error(e);
        }
    }

    // Validate CSV delimiter
    if let Some(delimiter) = file_config.csv_delimiter {
        if let Err(e) = validate_csv_delimiter_config(delimiter) {
            result = result.with_error(e);
        }
    }

    // Validate timeout
    if let Some(timeout) = file_config.timeout {
        if timeout == 0 {
            result = result.with_error("Timeout cannot be zero");
        }
    }

    // Validate insecure + pin_certs
    let pin_certs = file_config.pin_certs.unwrap_or(false);
    let insecure = file_config.insecure.unwrap_or(false);
    if insecure && pin_certs {
        result = result.with_error("Cannot use both 'insecure' and 'pin_certs'");
    }

    result
}

/// Get config file path.
pub fn get_config_path_internal() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "netspeed-cli", "netspeed-cli")
        .map(|dirs| dirs.config_dir().join("config.yaml"))
}

/// Load config file if it exists.
pub fn load_config_file() -> Option<File> {
    get_config_path_internal().and_then(|path| {
        if path.exists() {
            serde_yaml::from_reader(std::fs::File::ok()?).ok()
        } else {
            None
        }
    })
}