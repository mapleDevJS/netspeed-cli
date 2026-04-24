//! Structured logging infrastructure for netspeed-cli.
//!
//! This module provides logging utilities that can be used throughout the application.
//! It supports log levels, structured output, and runtime log level configuration.

use std::env;

/// Log level enumeration matching common severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    /// Debug-level messages for development and troubleshooting
    Debug,
    /// Informational messages about normal operation
    #[default]
    Info,
    /// Warning messages indicating potential issues
    Warn,
    /// Error messages indicating failures
    Error,
}

impl Level {
    /// Parse log level from environment variable value.
    #[must_use]
    pub fn from_env(var: &str) -> Self {
        match var.to_lowercase().as_str() {
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" | "warning" => Level::Warn,
            "error" | "err" => Level::Error,
            _ => Level::Info,
        }
    }

    /// Get the environment variable name for log level.
    #[must_use]
    pub const fn env_var() -> &'static str {
        "NETSPEED_LOG"
    }

    /// Check if this level should be logged given the current threshold.
    #[must_use]
    pub fn should_log(self, threshold: Level) -> bool {
        self as u8 >= threshold as u8
    }
}

/// Get the current log level from the NETSPEED_LOG environment variable.
#[must_use]
pub fn current_level() -> Level {
    env::var(Level::env_var())
        .ok()
        .map(|v| Level::from_env(&v))
        .unwrap_or_default()
}

/// Check if verbose logging is enabled.
#[must_use]
pub fn is_verbose() -> bool {
    current_level() == Level::Debug
}

/// Log a message with structured key-value pairs.
pub fn log(level: Level, message: &str, fields: &[(&str, &str)]) {
    if level.should_log(current_level()) {
        eprint!("[{}] {}", format!("{:?}", level).to_uppercase(), message);
        for (key, value) in fields {
            eprint!(" {}=\"{}\"", key, value);
        }
        eprintln!();
    }
}

/// Log a debug message.
pub fn debug(message: &str) {
    log(Level::Debug, message, &[]);
}

/// Log an info message.
pub fn info(message: &str) {
    log(Level::Info, message, &[]);
}

/// Log a warning message.
pub fn warn(message: &str) {
    log(Level::Warn, message, &[]);
}

/// Log an error message.
pub fn error(message: &str) {
    log(Level::Error, message, &[]);
}

/// Format a structured log entry as JSON for machine-readable output.
#[must_use]
pub fn format_json_entry(level: Level, message: &str, fields: &[(&str, &str)]) -> String {
    use serde_json::json;
    let mut map = serde_json::Map::new();
    map.insert(
        "level".to_string(),
        json!(format!("{:?}", level).to_lowercase()),
    );
    map.insert("message".to_string(), json!(message));
    map.insert(
        "timestamp".to_string(),
        json!(chrono::Utc::now().to_rfc3339()),
    );
    for (key, value) in fields {
        map.insert(key.to_string(), json!(value));
    }
    serde_json::to_string(&map).unwrap_or_else(|_| "{\"error\": \"log format failed\"}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_from_env_debug() {
        assert_eq!(Level::from_env("debug"), Level::Debug);
        assert_eq!(Level::from_env("DEBUG"), Level::Debug);
    }

    #[test]
    fn test_level_from_env_info() {
        assert_eq!(Level::from_env("info"), Level::Info);
        assert_eq!(Level::from_env("INFO"), Level::Info);
    }

    #[test]
    fn test_level_from_env_warn() {
        assert_eq!(Level::from_env("warn"), Level::Warn);
        assert_eq!(Level::from_env("warning"), Level::Warn);
        assert_eq!(Level::from_env("WARN"), Level::Warn);
    }

    #[test]
    fn test_level_from_env_error() {
        assert_eq!(Level::from_env("error"), Level::Error);
        assert_eq!(Level::from_env("err"), Level::Error);
        assert_eq!(Level::from_env("ERROR"), Level::Error);
    }

    #[test]
    fn test_level_from_env_invalid() {
        assert_eq!(Level::from_env("invalid"), Level::Info);
        assert_eq!(Level::from_env(""), Level::Info);
    }

    #[test]
    fn test_level_should_log() {
        assert!(Level::Debug.should_log(Level::Debug));
        assert!(Level::Info.should_log(Level::Debug));
        assert!(!Level::Debug.should_log(Level::Info));
        assert!(Level::Error.should_log(Level::Debug));
    }

    #[test]
    fn test_level_default() {
        assert_eq!(Level::default(), Level::Info);
    }

    #[test]
    fn test_format_json_entry() {
        let entry = format_json_entry(Level::Info, "test message", &[("key", "value")]);
        assert!(entry.contains("info"));
        assert!(entry.contains("test message"));
        assert!(entry.contains("timestamp"));
        assert!(entry.contains("key"));
        assert!(entry.contains("value"));
    }

    #[test]
    fn test_format_json_entry_empty_fields() {
        let entry = format_json_entry(Level::Error, "error occurred", &[]);
        assert!(entry.contains("error"));
        assert!(entry.contains("error occurred"));
    }
}
