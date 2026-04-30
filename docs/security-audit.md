# Security Audit Report

This document provides a comprehensive security audit framework for netspeed-cli. It documents the threat model, attack surface, existing security controls, and testing procedures.

## Threat Model

### Assumptions

1. **Network Communication**: All network traffic is untrusted. The CLI communicates exclusively with speedtest.net infrastructure.
2. **Local File System**: The system running the CLI is trusted. A local attacker would have full access to the tool's data.
3. **No Sensitive Data**: The tool does not store secrets, API keys, or credentials.
4. **User Environment**: The CLI runs in a standard terminal environment with user-level permissions.

### Assets to Protect

| Asset | Classification | Protection Required |
|-------|---------------|---------------------|
| Test results history | PII (IP addresses) | File permissions, no transmission |
| Config file | User preference | File permissions |
| TLS configuration | Security-critical | Certificate validation |
| Network traffic | Security-critical | TLS encryption |

### Attack Surface

#### Network Attack Vectors

1. **Man-in-the-Middle (MITM)**: An attacker intercepting traffic to speedtest.net servers.
2. **DNS Spoofing**: An attacker redirecting traffic to a malicious server.
3. **TLS Downgrade**: An attacker forcing a weaker TLS version.
4. **Certificate Forgery**: An attacker presenting a forged certificate.

#### Input Attack Vectors

1. **Server URL Injection**: Malicious server URLs from the speedtest.net XML feed.
2. **Config File Injection**: Malicious configuration values.
3. **CLI Argument Injection**: Malicious command-line arguments.

#### Output Attack Vectors

1. **Log Injection**: Malicious data in logs (if stdout is parsed by other tools).
2. **JSON/CSV Injection**: Malicious data in machine-readable output.

## Security Controls

### Transport Layer Security

| Control | Implementation | Status |
|---------|---------------|--------|
| TLS 1.3 support | `rustls` v0.23 | ✅ Implemented |
| TLS 1.2 support | `rustls` v0.23 | ✅ Implemented |
| Domain-restricted TLS | `PinningVerifier` wrapper around rustls webpki validation | ✅ Implemented |
| Custom CA support | `--ca-cert` flag | ✅ Implemented |
| Root CA trust store | `webpki-roots` | ✅ Implemented |

### Input Validation

| Control | Implementation | Status |
|---------|---------------|--------|
| Server URL normalization | `endpoints.rs` | ✅ Implemented |
| IP address validation | `common::is_valid_ipv4()` | ✅ Implemented |
| Config file validation | `config.rs::validate_config()` | ✅ Implemented |
| CSV delimiter validation | `config.rs::validate_csv_delimiter_config()` | ✅ Implemented |

### File System Security

| Control | Implementation | Status |
|---------|---------------|--------|
| History file permissions | `0o600` (Unix only) | ✅ Implemented |
| Atomic file writes | Temp file + rename | ✅ Implemented |
| Backup rotation | `.bak` and `.corrupt` files | ✅ Implemented |

### Dependency Security

| Control | Implementation | Status |
|---------|---------------|--------|
| Dependency audit | `cargo-deny` in CI | ✅ Implemented |
| License audit | `cargo-deny` in CI | ✅ Implemented |
| Advisory database | RUSTSEC | ✅ Configured |
| Lock file committed | `Cargo.lock` | ✅ Required |

## Testing Procedures

### Static Analysis

```bash
# Run clippy with all lints enabled
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt -- --check

# Check for unsafe code
cargo geiger

# Check for dependency updates
cargo outdated
```

### Security Testing

```bash
# Run cargo-deny security audit
cargo deny check

# Check for vulnerabilities
cargo audit

# Check for unmaintained dependencies
cargo audit --fetch-index
```

### Fuzz Testing

```bash
# Run fuzz tests (requires corpus)
cargo fuzz run parse_ip_from_xml
cargo fuzz run parse_server_url
```

### Integration Testing

```bash
# Run all integration tests
cargo test --test integration_test

# Run mock network tests
cargo test --test mock_network_test -- --ignored --nocapture

# Run socket-binding tests
cargo test --test e2e_test -- --ignored --nocapture
```

## Known Security Considerations

### Domain-restricted TLS (`--pin-certs`)

The `--pin-certs` option is intentionally implemented as domain-restricted TLS rather than raw certificate-hash pinning. This means:

- ✅ Only connections to `*.speedtest.net` and `*.ookla.com` are allowed
- ✅ The normal rustls/webpki certificate chain, validity, and hostname checks still run
- ✅ The custom verifier delegates TLS signature verification to rustls
- ⚠️ It does not pin SPKI/certificate hashes because speedtest.net server certificates are operationally rotated

If stable SPKI pins become available, add them only after normal webpki verification succeeds.

### No TLS Certificate Revocation

The current implementation does not check certificate revocation lists (CRL) or use OCSP stapling. This is a known limitation for a CLI tool where:
- Certificate lifetime is typically short
- CRL distribution points may not be accessible
- Speedtest.net infrastructure uses certificates from major CAs

### History File Contains IP Addresses

Test results include the client's IP address. While this data stays local, users should be aware:
- History files are created with `0o600` permissions on Unix
- The data is not transmitted to any server other than speedtest.net
- Users can disable history saving by using `--json` or `--csv` output

## Vulnerability Disclosure

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue for security-related concerns.
2. Open a [GitHub Security Advisory](https://github.com/mapleDevJS/netspeed-cli/security/advisories/new).
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

Response time: Within 48 hours for initial acknowledgment.

## Security Audit Schedule

| Frequency | Activity |
|-----------|----------|
| Per-PR | `cargo clippy`, `cargo fmt --check`, `cargo deny check` |
| Per-merge | `cargo audit`, dependency review |
| Monthly | Full security review, dependency update check |
| Quarterly | Third-party security assessment |

## Appendix: Security-related Configuration

### TLS Version Configuration

```toml
# config.toml
tls_version = "1.3"  # Require TLS 1.3 (or use 1.2)
```

### Domain-restricted TLS

```bash
# Add speedtest.net/ookla.com domain restriction on top of normal TLS validation
netspeed-cli --pin-certs

# Use custom CA certificate
netspeed-cli --ca-cert /path/to/ca.pem
```

### Network Security

```bash
# Set HTTP timeout (affects TLS handshake timeout)
netspeed-cli --timeout 30

# Bind to specific source IP
netspeed-cli --source 192.168.1.100
```
