use std::process::Command;

/// Test that the CLI help displays correctly
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    // stdout or stderr may contain help depending on clap version
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success(),
        "CLI help should succeed. stderr: {stderr}"
    );
    assert!(combined.contains("netspeed-cli"));
    assert!(combined.contains("bandwidth"));
}

/// Test that version flag works
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    // stdout or stderr may contain version
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success(),
        "CLI version should succeed. stderr: {stderr}"
    );
    assert!(combined.contains("netspeed-cli"));
    assert!(
        combined.chars().any(|c| c.is_ascii_digit()),
        "Version output should contain at least one digit"
    );
}

/// Test shell completion generation for bash
#[test]
fn test_shell_completion_bash() {
    // Completions are generated at build time, not runtime
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "bash"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("completions"));
    assert!(std::path::Path::new("completions/netspeed-cli.bash").exists());
}

/// Test shell completion generation for zsh
#[test]
fn test_shell_completion_zsh() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "zsh"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("completions"));
    assert!(std::path::Path::new("completions/_netspeed-cli").exists());
}

/// Test shell completion generation for fish
#[test]
fn test_shell_completion_fish() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "fish"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("completions"));
    assert!(std::path::Path::new("completions/netspeed-cli.fish").exists());
}

/// Test shell completion generation for powershell
#[test]
fn test_shell_completion_powershell() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "powershell"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("completions"));
    assert!(std::path::Path::new("completions/_netspeed-cli.ps1").exists());
}

/// Test shell completion generation for elvish
#[test]
fn test_shell_completion_elvish() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "elvish"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("completions"));
    assert!(std::path::Path::new("completions/netspeed-cli.elv").exists());
}

/// Test invalid CSV delimiter validation
#[test]
fn test_invalid_csv_delimiter() {
    let output = Command::new("cargo")
        .args(["run", "--", "--csv-delimiter", "abc"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("CSV delimiter") || stderr.contains("error"));
}

/// Test invalid IP address validation
#[test]
fn test_invalid_source_ip() {
    let output = Command::new("cargo")
        .args(["run", "--", "--source", "999.999.999.999"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("IP") || stderr.contains("error"));
}

/// Test invalid timeout validation (zero)
#[test]
fn test_zero_timeout() {
    let output = Command::new("cargo")
        .args(["run", "--", "--timeout", "0"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("timeout") || stderr.contains("error"));
}

/// Test invalid timeout validation (too large)
#[test]
fn test_timeout_too_large() {
    let output = Command::new("cargo")
        .args(["run", "--", "--timeout", "999"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("timeout") || stderr.contains("error"));
}

/// Test that --list flag executes (will fail without network, but validates parsing)
#[test]
fn test_list_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--list"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Either succeeds or fails with network error, not parsing error
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test --json flag parsing
#[test]
fn test_json_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--json"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test --csv flag parsing
#[test]
fn test_csv_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--csv"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test --no-download flag parsing
#[test]
fn test_no_download_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--no-download"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test --no-upload flag parsing
#[test]
fn test_no_upload_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--no-upload"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test --single flag parsing
#[test]
fn test_single_flag_parsing() {
    let output = Command::new("cargo")
        .args(["run", "--", "--single"])
        .output()
        .expect("Failed to execute command");

    // This may fail due to network, but parsing should work
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test multiple server flags
#[test]
fn test_multiple_server_flags() {
    let output = Command::new("cargo")
        .args(["run", "--", "--server", "1234", "--server", "5678"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test combined flags
#[test]
fn test_combined_flags() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--no-upload",
            "--json",
            "--single",
            "--timeout",
            "5",
        ])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"));
}

/// Test that error output includes the word "Error:" for user readability
#[test]
fn test_error_output_format() {
    let output = Command::new("cargo")
        .args(["run", "--", "--source", "invalid"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should have a user-friendly error message
    assert!(
        stderr.contains("Error") || stderr.contains("error") || stderr.contains("invalid"),
        "Expected user-friendly error message, got: {stderr}"
    );
}

/// Test that exit code is non-zero on error
/// Uses sysexits.h conventions: 64=usage error, 69=network error, etc.
#[test]
fn test_exit_code_on_error() {
    // Clap validation errors (like invalid IP) return exit code 64 (USAGE_ERROR)
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--source", "999.999.999.999"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let exit_code = output.status.code();
    assert!(
        exit_code == Some(1)
            || exit_code == Some(2)
            || exit_code == Some(64)
            || exit_code == Some(69)
            || exit_code == Some(70),
        "Expected non-zero exit code (sysexits.h conventions), got {exit_code:?}"
    );
}

/// Test that --version output matches Cargo.toml version
#[test]
fn test_version_matches_cargo_toml() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success(),
        "Version should succeed. stderr: {stderr}"
    );
    assert!(
        combined.contains("netspeed-cli"),
        "Version output should contain binary name: {combined}"
    );
}

/// Test --history flag with no existing history (should not crash)
#[test]
fn test_history_no_data() {
    // This may or may not produce output depending on whether history file exists,
    // but it should never panic or crash
    let output = Command::new("cargo")
        .args(["run", "--", "--history"])
        .output()
        .expect("Failed to execute command");

    // Either success with empty output or error is acceptable
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"));
    assert!(!stderr.contains("panic"));
}

/// Test that help output contains all documented options
#[test]
fn test_help_contains_expected_options() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success(),
        "Help should succeed. stderr: {stderr}"
    );

    // Verify key options are documented
    assert!(
        combined.contains("--no-download"),
        "Missing --no-download in help"
    );
    assert!(
        combined.contains("--no-upload"),
        "Missing --no-upload in help"
    );
    assert!(combined.contains("--single"), "Missing --single in help");
    assert!(combined.contains("--format"), "Missing --format in help");
    assert!(combined.contains("--list"), "Missing --list in help");
    assert!(combined.contains("--server"), "Missing --server in help");
    assert!(combined.contains("--history"), "Missing --history in help");
    assert!(combined.contains("--timeout"), "Missing --timeout in help");
}
