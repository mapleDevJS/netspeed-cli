//! Integration tests for config validation with strict mode.
//!
//! These tests verify that invalid config file values are handled correctly
//! depending on whether strict mode is enabled. They use the library's internal
//! functions directly for reliability.

use netspeed_cli::config::{validate_config, File, ValidationResult};

// ── Test Helper ───────────────────────────────────────────────────────

/// Simulates what happens when config is loaded and validated.
/// Returns the validation result and whether strict mode would cause exit.
fn validate_with_strict(file_config: &File, strict: bool) -> (ValidationResult, bool) {
    let validation = validate_config(file_config);
    let should_exit = strict && !validation.valid;
    (validation, should_exit)
}

// ── Invalid Profile Tests ────────────────────────────────────────────

#[test]
fn test_config_invalid_profile_non_strict() {
    let file_config = File {
        profile: Some("bad_profile".to_string()),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(!result.valid, "Invalid profile should make result invalid");
    assert!(!result.errors.is_empty(), "Should have errors");
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Invalid profile") || e.contains("bad_profile")),
        "Error should mention invalid profile: {:?}",
        result.errors
    );
    assert!(!should_exit, "Non-strict mode should not exit");
}

#[test]
fn test_config_invalid_profile_strict() {
    let file_config = File {
        profile: Some("invalid_profile".to_string()),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, true);

    assert!(!result.valid);
    assert!(should_exit, "Strict mode should exit on invalid config");
}

#[test]
fn test_config_strict_in_file() {
    // Note: validate_config doesn't read the 'strict' field from File.
    // The strict flag is handled in Config::from_args.
    // This test just verifies that invalid profile causes validation failure.
    let file_config = File {
        profile: Some("bad_one".to_string()),
        strict: Some(true), // This field is not used by validate_config
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false); // strict comes from CLI, not File

    assert!(!result.valid);
    // should_exit is false because we passed strict=false (no CLI override)
    assert!(
        !should_exit,
        "Without CLI --strict-config, no exit even if config has strict=true"
    );

    // The actual strict-from-file behavior would be tested at integration level
    // where Config::from_args reads both args.strict_config and file_config.strict
}

#[test]
fn test_config_valid_profile_no_errors() {
    let file_config = File {
        profile: Some("gamer".to_string()),
        ..Default::default()
    };
    let (result, _) = validate_with_strict(&file_config, false);

    assert!(result.valid, "Valid profile should make result valid");
    assert!(result.errors.is_empty(), "Should have no errors");
}

// ── Invalid Theme Tests ──────────────────────────────────────────────

#[test]
fn test_config_invalid_theme_non_strict() {
    let file_config = File {
        theme: Some("neon".to_string()),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Invalid theme") || e.contains("neon")),
        "Error should mention invalid theme: {:?}",
        result.errors
    );
    assert!(!should_exit, "Non-strict mode should not exit");
}

#[test]
fn test_config_invalid_theme_strict() {
    let file_config = File {
        theme: Some("flashy".to_string()),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, true);

    assert!(!result.valid);
    assert!(should_exit, "Strict mode should exit on invalid theme");
}

// ── Invalid CSV Delimiter Tests ──────────────────────────────────────

#[test]
fn test_config_invalid_csv_delimiter_non_strict() {
    let file_config = File {
        csv_delimiter: Some('X'),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Invalid CSV delimiter") || e.contains("X")),
        "Error should mention invalid CSV delimiter: {:?}",
        result.errors
    );
    assert!(!should_exit, "Non-strict mode should not exit");
}

#[test]
fn test_config_invalid_csv_delimiter_strict() {
    let file_config = File {
        csv_delimiter: Some('@'),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, true);

    assert!(!result.valid);
    assert!(should_exit, "Strict mode should exit on invalid delimiter");
}

// ── Deprecated Options Tests ─────────────────────────────────────────

#[test]
fn test_config_deprecated_simple_warning() {
    let file_config = File {
        simple: Some(true),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(result.valid, "Deprecated options are warnings, not errors");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("deprecated") && w.contains("simple")),
        "Should warn about deprecated 'simple' option: {:?}",
        result.warnings
    );
    assert!(!should_exit, "Warnings should not cause exit");
}

#[test]
fn test_config_deprecated_csv_warning() {
    let file_config = File {
        csv: Some(true),
        ..Default::default()
    };
    let (result, _) = validate_with_strict(&file_config, false);

    assert!(result.valid);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("deprecated") && w.contains("csv")),
        "Should warn about deprecated 'csv' option"
    );
}

#[test]
fn test_config_deprecated_json_warning() {
    let file_config = File {
        json: Some(true),
        ..Default::default()
    };
    let (result, _) = validate_with_strict(&file_config, false);

    assert!(result.valid);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("deprecated") && w.contains("json")),
        "Should warn about deprecated 'json' option"
    );
}

#[test]
fn test_config_deprecated_not_fatal_in_strict() {
    let file_config = File {
        simple: Some(true),
        csv: Some(true),
        json: Some(true),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, true);

    assert!(result.valid, "Deprecated options are warnings, not errors");
    assert!(
        !should_exit,
        "Deprecated options should not cause exit in strict mode"
    );
}

// ── Multiple Validation Issues Tests ────────────────────────────────

#[test]
fn test_config_multiple_issues_all_reported() {
    let file_config = File {
        profile: Some("bad_profile".to_string()),
        theme: Some("neon".to_string()),
        csv_delimiter: Some('!'),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(!result.valid, "Multiple issues should make result invalid");
    assert!(
        result.errors.len() >= 3,
        "Should have at least 3 errors, got: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("profile") || e.contains("bad_profile")),
        "Should have profile error: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("theme") || e.contains("neon")),
        "Should have theme error: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("CSV delimiter") || e.contains("!")),
        "Should have delimiter error: {:?}",
        result.errors
    );
    assert!(!should_exit, "Non-strict mode should not exit");
}

#[test]
fn test_config_multiple_issues_strict_exit() {
    let file_config = File {
        profile: Some("invalid".to_string()),
        theme: Some("weird".to_string()),
        csv_delimiter: Some('#'),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, true);

    assert!(!result.valid);
    assert!(
        should_exit,
        "Multiple issues should cause exit in strict mode"
    );
}

// ── Valid Config Tests ───────────────────────────────────────────────

#[test]
fn test_config_valid_no_warnings() {
    let file_config = File {
        profile: Some("gamer".to_string()),
        theme: Some("dark".to_string()),
        csv_delimiter: Some(','),
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(result.valid, "Valid config should be valid");
    assert!(result.errors.is_empty(), "Should have no errors");
    assert!(result.warnings.is_empty(), "Should have no warnings");
    assert!(!should_exit, "Should not exit");
}

#[test]
fn test_config_all_valid_themes() {
    let valid_themes = ["dark", "light", "high-contrast", "monochrome"];

    for theme in valid_themes {
        let file_config = File {
            theme: Some(theme.to_string()),
            ..Default::default()
        };
        let (result, _) = validate_with_strict(&file_config, false);

        assert!(result.valid, "Theme '{}' should be valid", theme);
        assert!(
            result.errors.is_empty(),
            "Theme '{}' should have no errors",
            theme
        );
    }
}

#[test]
fn test_config_empty_valid() {
    let file_config = File::default();
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(result.valid, "Empty config should be valid");
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
    assert!(!should_exit);
}

// ── Edge Cases ───────────────────────────────────────────────────────

#[test]
fn test_config_mixed_deprecated_and_invalid() {
    let file_config = File {
        simple: Some(true),               // deprecated warning
        profile: Some("bad".to_string()), // invalid error
        ..Default::default()
    };
    let (result, should_exit) = validate_with_strict(&file_config, false);

    assert!(!result.valid, "Invalid profile makes result invalid");
    assert!(
        !result.warnings.is_empty(),
        "Should have deprecation warning"
    );
    assert!(!result.errors.is_empty(), "Should have validation errors");
    assert!(!should_exit, "Non-strict should not exit");
}

#[test]
fn test_config_strict_mode_priority() {
    // When CLI sets --strict-config (true) it should override config file's strict (false)
    // This is tested by passing strict=true to our helper (simulating CLI override)
    let file_config = File {
        strict: Some(false), // config file says not strict
        ..Default::default()
    };
    let (_result, _should_exit) = validate_with_strict(&file_config, true); // CLI overrides

    // With strict=true from CLI, and invalid profile, should exit
    let file_config_invalid = File {
        profile: Some("bad".to_string()),
        strict: Some(false),
        ..Default::default()
    };
    let (_, cli_should_exit) = validate_with_strict(&file_config_invalid, true);
    assert!(cli_should_exit, "CLI --strict-config should take priority");
}

#[test]
fn test_config_all_valid_profiles() {
    let valid_profiles = ["gamer", "streamer", "remote-worker", "power-user", "casual"];

    for profile in valid_profiles {
        let file_config = File {
            profile: Some(profile.to_string()),
            ..Default::default()
        };
        let (result, _) = validate_with_strict(&file_config, false);

        assert!(result.valid, "Profile '{}' should be valid", profile);
    }
}

#[test]
fn test_config_all_valid_csv_delimiters() {
    let valid_delimiters = [',', ';', '|', '\t'];

    for delimiter in valid_delimiters {
        let file_config = File {
            csv_delimiter: Some(delimiter),
            ..Default::default()
        };
        let (result, _) = validate_with_strict(&file_config, false);

        assert!(result.valid, "Delimiter '{:?}' should be valid", delimiter);
    }
}
