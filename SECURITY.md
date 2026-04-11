# Security Policy

## Supported Versions

Only the latest release version is supported with security updates. If you are using an older version, please upgrade to the latest release.

| Version | Supported          |
| ------- | ------------------ |
| 0.7.x   | :white_check_mark: |
| < 0.7   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in netspeed-cli, please report it responsibly:

1. **Do not** open a public GitHub issue for security-related concerns.
2. Open a [GitHub Security Advisory](https://github.com/mapleDevJS/netspeed-cli/security/advisories/new) or contact the maintainer directly.
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if you have one)

You should receive a response within 48 hours. If you don't, please follow up via a GitHub issue (without disclosing security details).

## Security Practices

- The project uses `rustls` for TLS (no OpenSSL dependency)
- Dependencies are scanned for known vulnerabilities via `cargo audit` in CI
- No secrets, API keys, or credentials are stored by the tool
- Network communication is with speedtest.net infrastructure only

## Dependency Security

All dependencies are managed through `Cargo.lock` (committed to the repository) and automatically audited by [RUSTSEC](https://rustsec.org/) in the CI pipeline.
