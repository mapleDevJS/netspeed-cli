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

/// Helper to create a unique temp certificate file path.
fn temp_cert_path() -> std::path::PathBuf {
    let pid = std::process::id();
    std::env::temp_dir().join(format!("netspeed_test_cert_{}.pem", pid))
}

/// Helper to create a test certificate file.
fn create_test_cert(path: &std::path::Path) {
    let cert_content = "-----BEGIN CERTIFICATE-----\nMIIDXTCCAkWgAwIBAgIJAKJ8h5L7V3R2MA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV\nBAYTAlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNp\nc2NvMRUwEwYDVQQKDAxUZXN0IERlbW8gQ0EwHhcNMjMwMTAxMDAwMDAwWhcNMjQw\nMTAxMDAwMDAwWjBFMQswCQYDVQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTEW\nMBQGA1UEBwwNU2FuIEZyYW5jaXNjbzEVMBMGA1UECgwMVGVzdCBEZW1vIENBMIIB\nIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw1s3xfn8Z8c3R1hL+8jK2w0F\nkZmJkZnJmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm\nZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm\nZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm\nAgMBAAGjUzBRMB0GA1UdDgQWBBQYT9W7dF2R2P7L5D3K9Z2Y5Q3Z8DAfBgNVHSME\nGDAWgBQYT9W7dF2R2P7L5D3K9Z2Y5Q3Z8DAPBgNVHRMBAf8EBTADAQH/MA0GCSqG\nSIb3DQEBCwUAA4IBAQCQ4e1H8gZ+8f3F5N3F6hK7L5J2N4L9K8Q0L1M2N3O4P5Q6\nR7S8T9U0V1W2X3Y4Z5A6B7C8D9E0F1G2H3I4J5K6L7M8N9O0P1Q2R3S4T5U6V7W8\n-----END CERTIFICATE-----".as_bytes();
    fs::write(path, cert_content).expect("Failed to write test cert");
}

// ── Help Documentation Tests ─────────────────────────────────────────

#[test]
fn test_ca_cert_in_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(combined.contains("--ca-cert"), "--ca-cert should be documented in help");
}

#[test]
fn test_pin_certs_in_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(combined.contains("--pin-certs"), "--pin-certs should be documented in help");
}

#[test]
fn test_tls_version_in_help() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(combined.contains("--tls-version"), "--tls-version should be documented in help");
}

// ── Path Validation Tests ────────────────────────────────────────────

#[test]
fn test_ca_cert_accepts_valid_pem_file() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", cert_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("does not exist") && !stderr.contains("is a directory"),
        "Valid cert path should be accepted. stderr: {stderr}");
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_ca_cert_rejects_nonexistent_path() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", "/nonexistent/path/to/cert.pem"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Non-existent CA cert path should be rejected");
    assert!(stderr.contains("not found") || stderr.contains("does not exist"),
        "Error should mention file not found. stderr: {stderr}");
}

#[test]
fn test_ca_cert_rejects_directory() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", "/tmp"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Directory path for --ca-cert should be rejected");
    assert!(stderr.contains("not a file") || stderr.contains("is a directory"),
        "Error should mention it's a directory. stderr: {stderr}");
}

// ── TLS Version Tests ────────────────────────────────────────────────

#[test]
fn test_tls_version_rejects_invalid() {
    for version in ["2.0", "1.1", "3.0", "TLSv1.2"] {
        let output = Command::new("cargo")
            .args(["run", "--quiet", "--", "--tls-version", version])
            .output()
            .expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "Invalid TLS version '{}' should be rejected", version);
        assert!(stderr.contains("1.2") && stderr.contains("1.3"),
            "Error should mention valid TLS versions. stderr: {stderr}");
    }
}

#[test]
fn test_tls_version_accepts_1_2() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--tls-version", "1.2"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: invalid value"),
        "Valid TLS version 1.2 should be accepted. stderr: {stderr}");
}

#[test]
fn test_tls_version_accepts_1_3() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--tls-version", "1.3"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: invalid value"),
        "Valid TLS version 1.3 should be accepted. stderr: {stderr}");
}

// ── Pin Certs Tests ──────────────────────────────────────────────────

#[test]
fn test_pin_certs_flag_accepted() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--pin-certs"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "--pin-certs should be accepted. stderr: {stderr}");
}

#[test]
fn test_pin_certs_combined_with_json() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--pin-certs", "--json"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "--pin-certs with --json should parse successfully. stderr: {stderr}");
}

#[test]
fn test_pin_certs_with_format_dashboard() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--pin-certs", "--format", "dashboard"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "--pin-certs with --format dashboard should parse successfully. stderr: {stderr}");
}

// ── Combination Tests ────────────────────────────────────────────────

#[test]
fn test_ca_cert_combined_with_tls_version() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", cert_path.to_str().unwrap(), "--tls-version", "1.3"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "--ca-cert combined with --tls-version should parse successfully. stderr: {stderr}");
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_pin_certs_with_ca_cert_shows_warning() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", cert_path.to_str().unwrap(), "--pin-certs"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Warning") && stderr.contains("--ca-cert") && stderr.contains("--pin-certs"),
        "Warning should mention conflict between --ca-cert and --pin-certs. stderr: {stderr}");
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_all_tls_options_together() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", cert_path.to_str().unwrap(), "--tls-version", "1.2", "--pin-certs", "--json"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "All TLS options combined should parse successfully. stderr: {stderr}");
    fs::remove_file(&cert_path).ok();
}

#[test]
fn test_tls_options_with_other_flags() {
    let cert_path = temp_cert_path();
    create_test_cert(&cert_path);
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", cert_path.to_str().unwrap(), "--no-download", "--json"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: unexpected argument"),
        "TLS options with --no-download should parse successfully. stderr: {stderr}");
    fs::remove_file(&cert_path).ok();
}

// ── Error Message Tests ──────────────────────────────────────────────

#[test]
fn test_ca_cert_error_message_format() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--ca-cert", "/nonexistent/cert.pem"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("error") || stderr.contains("invalid"),
        "Expected user-friendly error message, got: {stderr}");
}

#[test]
fn test_tls_version_error_lists_valid_options() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--tls-version", "2.0"])
        .output()
        .expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("1.2") && stderr.contains("1.3"),
        "Error should list valid TLS versions. stderr: {stderr}");
}
