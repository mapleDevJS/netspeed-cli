use std::process::Command;

/// Test that the CLI help displays correctly
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
    assert!(stdout.contains("bandwidth"));
}

/// Test that version flag works
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("0.1.0"));
}

/// Test shell completion generation for bash
#[test]
fn test_shell_completion_bash() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "bash"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
}

/// Test shell completion generation for zsh
#[test]
fn test_shell_completion_zsh() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "zsh"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
}

/// Test shell completion generation for fish
#[test]
fn test_shell_completion_fish() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "fish"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
}

/// Test shell completion generation for powershell
#[test]
fn test_shell_completion_powershell() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "power-shell"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
}

/// Test shell completion generation for elvish
#[test]
fn test_shell_completion_elvish() {
    let output = Command::new("cargo")
        .args(["run", "--", "--generate-completion", "elvish"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("netspeed-cli"));
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

/// Test invalid URL validation
#[test]
fn test_invalid_mini_url() {
    let output = Command::new("cargo")
        .args(["run", "--", "--mini", "invalid-url"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("URL") || stderr.contains("error"));
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
