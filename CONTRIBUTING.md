# Contributing to netspeed-cli

Thank you for your interest in contributing to netspeed-cli! This document provides guidelines for contributing to the project.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
In short: be respectful, constructive, and inclusive in all interactions.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the behavior
- **Expected vs actual behavior**
- **Environment**: OS, Rust version, netspeed-cli version
- **Error output** if applicable

### Suggesting Enhancements

Enhancement suggestions should include:

- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives considered**: Other approaches you've thought about

### Pull Requests

1. **Fork** the repository
2. **Create a branch** from `main` (`git checkout -b feature/my-feature`)
3. **Make your changes**
4. **Run tests**: `cargo test`
5. **Run clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
6. **Format code**: `cargo fmt --all`
7. **Commit** with clear messages (see below)
8. **Open a Pull Request**

## Development Setup

```bash
# Clone the repository
git clone https://github.com/alexeyivanov/netspeed-cli.git
cd netspeed-cli

# Build in development mode
cargo build

# Run tests
cargo test

# Run with sample args
cargo run -- --help
```

## Code Style

- **Follow Rust conventions**: Use `cargo clippy` and `cargo fmt`
- **Document public APIs**: All `pub` items should have `///` doc comments
- **Write tests**: New features should include tests
- **Error handling**: Use `thiserror` for error types, avoid `unwrap()` in production code

### Commit Message Format

```
type: short description

Optional longer description

- type: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `ci`, `deps`
- Keep subject line under 72 characters
- Use imperative mood ("Add feature" not "Added feature")
```

Examples:
```
fix: handle empty server list gracefully
feat: add CSV output with custom delimiter
deps: update reqwest to 0.12
test: add integration tests for server discovery
```

## Testing

- **Unit tests**: For individual functions/modules
- **Integration tests**: In `tests/` directory
- **Mock HTTP**: Use `wiremock` for network-dependent tests

Run all tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test test_name
```

## Code Review

All PRs require review before merging. Reviewers will check:

- [ ] Tests pass
- [ ] Clippy warnings resolved
- [ ] Code is formatted
- [ ] Doc comments present
- [ ] Logic is correct and efficient
- [ ] No security issues

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
