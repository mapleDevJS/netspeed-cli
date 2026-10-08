//! Integration tests for TLS CLI options.
//!
//! Tests cover:
//! - `--ca-cert` path validation and file format handling
//! - `--pin-certs` flag acceptance
//! - `--tls-version` parsing (1.2, 1.3)
//! - Combination of TLS options with other CLI flags
//! - Error cases and warnings

use std::fs;
use std::process::Command;

// Execute the built binary; dry-run validates configuration without network I/O.
fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_netspeed-cli"));
    command.arg("--dry-run").env("NO_COLOR", "1");
    command
}

/// Helper to create a unique temp certificate file path.
fn temp_cert_path() -> std::path::PathBuf {
    tempfile::NamedTempFile::new().unwrap().keep().unwrap().1
}

/// Returns a path that should not exist (cross-platform).
fn nonexistent_path() -> std::path::PathBuf {
    let temp = std::env::temp_dir();
    temp.join(format!("netspeed_nonexistent_{}.pem", std::process::id()))
}

/// Returns a path to an existing directory (cross-platform).
fn existing_directory_path() -> std::path::PathBuf {
    std::env::temp_dir()
}

/// Helper to create a test certificate file.
fn create_test_cert(path: &std::path::Path) {
    fs::write(path, include_bytes!("fixtures/tls-cert.pem")).expect("Failed to write test cert");
}

// ── Help Documentation Tests ─────────────────────────────────────────

#[test]
fn test_ca_cert_in_help() {
    let output = cli()
        .args(["--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        combined.contains("--ca-cert"),
        "--ca-cert should be documented in help"
    );
}

#[test]
fn test_pin_certs_in_help() {
    let output = cli()
        .args(["--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        combined.contains("--pin-certs"),
        "--pin-certs should be documented in help"
    );
}

#[test]
fn test_tls_version_in_help() {
    let output = cli()
        .args(["--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        combined.contains("--tls-version"),
        "--tls-version should be documented in help"
    );
}

// ── Path Validation Tests ────────────────────────────────────────────

#[test]
fn test_ca_cert_accepts_valid_pem_file() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = cli()
        .args(["--ca-cert", cert_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        !stderr.contains("does not exist") && !stderr.contains("is a directory"),
        "Valid cert path should be accepted. stderr: {stderr}"
    );
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_ca_cert_rejects_nonexistent_path() {
    let cert_path = nonexistent_path();
    let output = cli()
        .args(["--ca-cert", cert_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Non-existent CA cert path should be rejected"
    );
    assert!(
        stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("No such file"),
        "Error should mention file not found. stderr: {stderr}"
    );
}

#[test]
fn test_ca_cert_rejects_directory() {
    let dir_path = existing_directory_path();
    let output = cli()
        .args(["--ca-cert", dir_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "Directory path for --ca-cert should be rejected"
    );
    assert!(
        stderr.contains("not a file") || stderr.contains("is a directory"),
        "Error should mention it's a directory. stderr: {stderr}"
    );
}

// ── TLS Version Tests ────────────────────────────────────────────────

#[test]
fn test_tls_version_rejects_invalid() {
    for version in ["2.0", "1.1", "3.0", "TLSv1.2"] {
        let output = cli()
            .args(["--tls-version", version])
            .output()
            .expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "Invalid TLS version '{}' should be rejected",
            version
        );
        assert!(
            stderr.contains("1.2") && stderr.contains("1.3"),
            "Error should mention valid TLS versions. stderr: {stderr}"
        );
    }
}

#[test]
fn test_tls_version_accepts_1_2() {
    let output = cli()
        .args(["--tls-version", "1.2"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "Valid TLS version 1.2 should be accepted. stderr: {stderr}"
    );
}

#[test]
fn test_tls_version_accepts_1_3() {
    let output = cli()
        .args(["--tls-version", "1.3"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "Valid TLS version 1.3 should be accepted. stderr: {stderr}"
    );
}

// ── Pin Certs Tests ──────────────────────────────────────────────────

#[test]
fn test_pin_certs_flag_accepted() {
    let output = cli()
        .args(["--pin-certs"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "--pin-certs should be accepted. stderr: {stderr}"
    );
}

#[test]
fn test_pin_certs_combined_with_json() {
    let output = cli()
        .args(["--pin-certs", "--json"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "--pin-certs with --json should parse successfully. stderr: {stderr}"
    );
}

#[test]
fn test_pin_certs_with_format_dashboard() {
    let output = cli()
        .args(["--pin-certs", "--format", "dashboard"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "--pin-certs with --format dashboard should parse successfully. stderr: {stderr}"
    );
}

// ── Combination Tests ────────────────────────────────────────────────

#[test]
fn test_ca_cert_combined_with_tls_version() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = cli()
        .args([
            "--ca-cert",
            cert_path.to_str().unwrap(),
            "--tls-version",
            "1.3",
        ])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "--ca-cert combined with --tls-version should parse successfully. stderr: {stderr}"
    );
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_missing_ca_is_rejected_with_pin_certs() {
    let output = cli()
        .args(["--ca-cert", "/some/path.pem", "--pin-certs"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "A missing CA file must be rejected"
    );
    assert!(
        stderr.contains("not found"),
        "Error should identify the missing CA file. stderr: {stderr}"
    );
}

#[test]
fn test_all_tls_options_together() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = cli()
        .args([
            "--ca-cert",
            cert_path.to_str().unwrap(),
            "--tls-version",
            "1.2",
            "--pin-certs",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "All TLS options combined should parse successfully. stderr: {stderr}"
    );
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_tls_options_with_other_flags() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = cli()
        .args([
            "--ca-cert",
            cert_path.to_str().unwrap(),
            "--no-download",
            "--json",
        ])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "valid configuration must succeed: {stderr}"
    );
    assert!(
        output.status.success(),
        "TLS options with --no-download should parse successfully. stderr: {stderr}"
    );
    fs::remove_file(&cert_path).ok();
}

// ── Error Message Tests ──────────────────────────────────────────────

#[test]
fn test_ca_cert_error_message_format() {
    let cert_path = nonexistent_path();
    let output = cli()
        .args(["--ca-cert", cert_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error") || stderr.contains("invalid"),
        "Expected user-friendly error message, got: {stderr}"
    );
}

#[test]
fn test_tls_version_error_lists_valid_options() {
    let output = cli()
        .args(["--tls-version", "2.0"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1.2") && stderr.contains("1.3"),
        "Error should list valid TLS versions. stderr: {stderr}"
    );
}
