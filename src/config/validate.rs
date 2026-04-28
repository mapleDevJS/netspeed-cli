//! Config validation types and functions.
//!
//! [`ValidationResult`] uses a builder pattern — start with [`ValidationResult::ok()`]
//! then chain [`with_error`](ValidationResult::with_error) /
//! [`with_warning`](ValidationResult::with_warning) calls.

use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use super::File;

/// Validation result with error details.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::ValidationResult;
///
/// let result = ValidationResult::ok()
///     .with_warning("deprecated option")
///     .with_error("invalid profile");
///
/// assert!(!result.valid);
/// assert_eq!(result.errors.len(), 1);
/// assert_eq!(result.warnings.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed (no errors). Warnings do not affect this.
    pub valid: bool,
    /// Error messages (any error sets [`valid`](ValidationResult::valid) to `false`).
    pub errors: Vec<String>,
    /// Warning messages (do not affect [`valid`](ValidationResult::valid)).
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::ok();
    /// assert!(result.valid);
    /// assert!(result.errors.is_empty());
    /// assert!(result.warnings.is_empty());
    /// ```
    #[must_use]
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result.
    ///
    /// Warnings do **not** change [`valid`](ValidationResult::valid).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::ok().with_warning("'simple' is deprecated");
    /// assert!(result.valid);
    /// assert_eq!(result.warnings.len(), 1);
    /// assert!(result.warnings[0].contains("deprecated"));
    /// ```
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Create a validation failure.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::error("invalid profile 'foo'");
    /// assert!(!result.valid);
    /// assert_eq!(result.errors.len(), 1);
    /// assert!(result.errors[0].contains("foo"));
    /// assert!(result.warnings.is_empty());
    /// ```
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result, flipping [`valid`](ValidationResult::valid) to `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::ok().with_error("bad theme");
    /// assert!(!result.valid);
    /// assert_eq!(result.errors.len(), 1);
    /// ```
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.valid = false;
        self
    }

    /// Merge another validation result into this one.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let a = ValidationResult::ok().with_warning("warn-a");
    /// let b = ValidationResult::error("bad profile");
    /// let merged = a.merge(b);
    /// assert!(!merged.valid);
    /// assert_eq!(merged.errors.len(), 1);
    /// assert_eq!(merged.warnings.len(), 1);
    /// ```
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

/// Validate the entire file config structure.
pub fn validate_config(file_config: &File) -> ValidationResult {
    let mut result = ValidationResult::ok();

    if let Some(ref profile) = file_config.profile {
        if let Err(e) = crate::profiles::UserProfile::validate(profile) {
            result = result.with_error(e);
        }
    }

    if let Some(ref theme) = file_config.theme {
        if let Err(e) = crate::theme::Theme::validate(theme) {
            result = result.with_error(e);
        }
    }

    if let Some(delimiter) = file_config.csv_delimiter {
        if let Err(e) = validate_csv_delimiter_config(delimiter) {
            result = result.with_error(e);
        }
    }

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

/// Get the configuration file path (also used by orchestrator for --show-config-path).
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

/// Load the configuration file from the standard config path.
///
/// Returns `None` if no config file exists or if loading fails.
pub fn load_config_file() -> Option<File> {
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
