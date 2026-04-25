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

    // ── Level enum tests ────────────────────────────────────────────────────

    #[test]
    fn test_level_from_env_debug() {
        assert_eq!(Level::from_env("debug"), Level::Debug);
        assert_eq!(Level::from_env("DEBUG"), Level::Debug);
        assert_eq!(Level::from_env("Debug"), Level::Debug);
    }

    #[test]
    fn test_level_from_env_info() {
        assert_eq!(Level::from_env("info"), Level::Info);
        assert_eq!(Level::from_env("INFO"), Level::Info);
        assert_eq!(Level::from_env("Info"), Level::Info);
    }

    #[test]
    fn test_level_from_env_warn() {
        assert_eq!(Level::from_env("warn"), Level::Warn);
        assert_eq!(Level::from_env("warning"), Level::Warn);
        assert_eq!(Level::from_env("WARN"), Level::Warn);
        assert_eq!(Level::from_env("WARNING"), Level::Warn);
        assert_eq!(Level::from_env("Warn"), Level::Warn);
    }

    #[test]
    fn test_level_from_env_error() {
        assert_eq!(Level::from_env("error"), Level::Error);
        assert_eq!(Level::from_env("err"), Level::Error);
        assert_eq!(Level::from_env("ERROR"), Level::Error);
        assert_eq!(Level::from_env("Err"), Level::Error);
    }

    #[test]
    fn test_level_from_env_invalid() {
        assert_eq!(Level::from_env("invalid"), Level::Info);
        assert_eq!(Level::from_env("trash"), Level::Info);
        assert_eq!(Level::from_env(""), Level::Info);
        assert_eq!(Level::from_env("123"), Level::Info);
    }

    #[test]
    fn test_level_should_log() {
        // Debug should_log tests
        assert!(Level::Debug.should_log(Level::Debug));
        assert!(Level::Info.should_log(Level::Debug)); // Info >= Debug threshold
        assert!(Level::Warn.should_log(Level::Debug));
        assert!(Level::Error.should_log(Level::Debug));

        // Info should_log tests
        assert!(!Level::Debug.should_log(Level::Info)); // Debug < Info threshold
        assert!(Level::Info.should_log(Level::Info));
        assert!(Level::Warn.should_log(Level::Info));
        assert!(Level::Error.should_log(Level::Info));

        // Warn should_log tests
        assert!(!Level::Debug.should_log(Level::Warn));
        assert!(!Level::Info.should_log(Level::Warn));
        assert!(Level::Warn.should_log(Level::Warn));
        assert!(Level::Error.should_log(Level::Warn));

        // Error should_log tests
        assert!(!Level::Debug.should_log(Level::Error));
        assert!(!Level::Info.should_log(Level::Error));
        assert!(!Level::Warn.should_log(Level::Error));
        assert!(Level::Error.should_log(Level::Error));
    }

    #[test]
    fn test_level_default() {
        assert_eq!(Level::default(), Level::Info);
    }

    #[test]
    fn test_level_debug_trait() {
        let debug_str = format!("{:?}", Level::Debug);
        assert!(debug_str.contains("Debug"));

        let debug_str = format!("{:?}", Level::Info);
        assert!(debug_str.contains("Info"));

        let debug_str = format!("{:?}", Level::Warn);
        assert!(debug_str.contains("Warn"));

        let debug_str = format!("{:?}", Level::Error);
        assert!(debug_str.contains("Error"));
    }

    #[test]
    fn test_level_clone() {
        let level = Level::Debug;
        let cloned = level.clone();
        assert_eq!(cloned, level);
    }

    #[test]
    fn test_level_copy() {
        let level = Level::Error;
        let copied = level; // Copy, not clone
        assert_eq!(copied, level);
    }

    #[test]
    fn test_level_eq() {
        assert_eq!(Level::Debug, Level::Debug);
        assert_eq!(Level::Info, Level::Info);
        assert_eq!(Level::Warn, Level::Warn);
        assert_eq!(Level::Error, Level::Error);
        assert_ne!(Level::Debug, Level::Info);
        assert_ne!(Level::Info, Level::Error);
    }

    #[test]
    fn test_level_partial_eq() {
        assert!(Level::Debug == Level::Debug);
        assert!(Level::Debug != Level::Error);
    }

    #[test]
    fn test_level_env_var() {
        assert_eq!(Level::env_var(), "NETSPEED_LOG");
    }

    // ── current_level tests ─────────────────────────────────────────────────

    #[test]
    fn test_current_level_returns_info_by_default() {
        // Without NETSPEED_LOG set, should return Info
        let level = current_level();
        assert_eq!(level, Level::Info);
    }

    // ── is_verbose tests ────────────────────────────────────────────────────

    #[test]
    fn test_is_verbose_by_default() {
        // Without NETSPEED_LOG set to debug, should not be verbose
        let verbose = is_verbose();
        assert!(!verbose);
    }

    // ── log function tests ──────────────────────────────────────────────────

    #[test]
    fn test_log_empty_fields() {
        // Should not panic and should produce output
        log(Level::Info, "test message", &[]);
    }

    #[test]
    fn test_log_with_fields() {
        log(
            Level::Debug,
            "test",
            &[("key1", "value1"), ("key2", "value2")],
        );
    }

    #[test]
    fn test_log_special_characters_in_fields() {
        log(Level::Info, "test", &[("key", "value with spaces")]);
        log(Level::Info, "test", &[("key", "value\"with\"quotes")]);
        log(Level::Info, "test", &[("key", "")]);
    }

    // ── shortcut logging functions tests ────────────────────────────────────

    #[test]
    fn test_debug_function() {
        debug("debug message");
    }

    #[test]
    fn test_info_function() {
        info("info message");
    }

    #[test]
    fn test_warn_function() {
        warn("warning message");
    }

    #[test]
    fn test_error_function() {
        error("error message");
    }

    // ── format_json_entry tests ─────────────────────────────────────────────

    #[test]
    fn test_format_json_entry_debug() {
        let entry = format_json_entry(Level::Debug, "debug message", &[("key", "value")]);
        assert!(entry.contains("debug"));
        assert!(entry.contains("debug message"));
        assert!(entry.contains("timestamp"));
        assert!(entry.contains("key"));
        assert!(entry.contains("value"));
    }

    #[test]
    fn test_format_json_entry_info() {
        let entry = format_json_entry(Level::Info, "test message", &[("key", "value")]);
        assert!(entry.contains("info"));
        assert!(entry.contains("test message"));
        assert!(entry.contains("timestamp"));
        assert!(entry.contains("key"));
        assert!(entry.contains("value"));
    }

    #[test]
    fn test_format_json_entry_warn() {
        let entry = format_json_entry(Level::Warn, "warning message", &[]);
        assert!(entry.contains("warn"));
        assert!(entry.contains("warning message"));
    }

    #[test]
    fn test_format_json_entry_error() {
        let entry = format_json_entry(Level::Error, "error occurred", &[]);
        assert!(entry.contains("error"));
        assert!(entry.contains("error occurred"));
    }

    #[test]
    fn test_format_json_entry_empty_fields() {
        let entry = format_json_entry(Level::Error, "error occurred", &[]);
        assert!(entry.contains("error"));
        assert!(entry.contains("error occurred"));
        // Should still have timestamp even with no fields
        assert!(entry.contains("timestamp"));
    }

    #[test]
    fn test_format_json_entry_multiple_fields() {
        let entry = format_json_entry(
            Level::Info,
            "multi-field message",
            &[
                ("field1", "value1"),
                ("field2", "value2"),
                ("field3", "value3"),
            ],
        );
        assert!(entry.contains("field1"));
        assert!(entry.contains("value1"));
        assert!(entry.contains("field2"));
        assert!(entry.contains("value2"));
        assert!(entry.contains("field3"));
        assert!(entry.contains("value3"));
    }

    #[test]
    fn test_format_json_entry_is_valid_json() {
        let entry = format_json_entry(Level::Info, "test", &[("key", "value")]);
        // Should be parseable as JSON
        let parsed: serde_json::Value = serde_json::from_str(&entry).unwrap();
        assert_eq!(parsed["level"], "info");
        assert_eq!(parsed["message"], "test");
        assert!(parsed.get("timestamp").is_some());
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_format_json_entry_timestamp_format() {
        let entry = format_json_entry(Level::Info, "test", &[]);
        let parsed: serde_json::Value = serde_json::from_str(&entry).unwrap();
        let timestamp = parsed["timestamp"].as_str().unwrap();
        // Should be RFC3339 format (contains date-time separator T or space)
        // chrono::Utc::now().to_rfc3339() produces format like "2024-01-01T12:00:00Z"
        assert!(!timestamp.is_empty());
        // Should contain year, month, day
        assert!(timestamp.contains("-"));
    }
}
