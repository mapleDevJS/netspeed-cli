# Publishing to Homebrew

This guide explains how to publish `netspeed-cli` to Homebrew so users can install it with `brew install netspeed-cli`.

## Prerequisites

1. A GitHub repository: `https://github.com/mapleDevJS/netspeed-cli`
2. A versioned release (tag) on GitHub
3. The Homebrew formula file: `netspeed-cli.rb`

## Step-by-Step Guide

### Option 1: Create a Homebrew Tap (Recommended for new projects)

A Homebrew tap is a GitHub repository that hosts custom formulas.

#### 1. Create a tap repository

Create a new GitHub repository named `homebrew-netspeed-cli`:
```bash
# Create repository on GitHub
# URL: https://github.com/mapleDevJS/homebrew-netspeed-cli
```

The repository **must** be named `homebrew-<something>` for Homebrew to recognize it as a tap.

#### 2. Add the formula to the tap

Copy the `netspeed-cli.rb` file to the new repository:
```bash
cd /path/to/homebrew-netspeed-cli
cp /path/to/netspeed-cli/netspeed-cli.rb .
git add netspeed-cli.rb
git commit -m "Add netspeed-cli formula"
git push origin main
```

#### 3. Create a GitHub release

Create a tagged release on GitHub:
```bash
# In your netspeed-cli repository
git tag v0.1.0
git push origin v0.1.0
```

Or use the GitHub UI to create a release with tag `v0.1.0`.

#### 4. Calculate the SHA256 checksum

Homebrew needs the SHA256 of the source tarball:
```bash
# Download the tarball
curl -L "https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/v0.1.0.tar.gz" -o netspeed-cli.tar.gz

# Calculate SHA256
sha256sum netspeed-cli.tar.gz
# or on macOS:
shasum -a 256 netspeed-cli.tar.gz
```

Update the `sha256` field in `netspeed-cli.rb` with the calculated value.

#### 5. Users can now install via your tap

```bash
brew tap mapleDevJS/netspeed-cli
brew install netspeed-cli
```

### Option 2: Submit to homebrew-core (Official repository)

For wider distribution, submit your formula to the official homebrew-core repository.

#### Requirements

- Your project must be stable and well-tested
- The formula must follow [Homebrew's formula requirements](https://docs.brew.sh/Acceptable-Formulae)
- Your project should have a recognizable user base

#### Steps

1. Fork [homebrew-core](https://github.com/Homebrew/homebrew-core)
2. Add your formula to `Formula/n/netspeed-cli.rb`
3. Submit a pull request
4. Follow the review process

## Testing the Formula Locally

Before publishing, test the formula locally:

```bash
# Create a local tap directory
mkdir -p /tmp/homebrew-netspeed-cli
cp netspeed-cli.rb /tmp/homebrew-netspeed-cli/

# Tap the local repository
brew tap --custom-remote --force-auto-update mapleDevJS/netspeed-cli file:///tmp/homebrew-netspeed-cli

# Install from the tap
brew install netspeed-cli

# Test the installation
netspeed-cli --version
```

## Updating the Formula

When releasing a new version:

1. Create a new GitHub release with an updated tag (e.g., `v0.2.0`)
2. Calculate the new SHA256
3. Update the `url` and `sha256` fields in `netspeed-cli.rb`
4. Commit and push the updated formula to your tap repository

```bash
# Example update
url "https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/v0.2.0.tar.gz"
sha256 "new_sha256_value_here"
```

## Troubleshooting

### "sha256 mismatch" error

This means the SHA256 in the formula doesn't match the downloaded tarball. Recalculate and update it.

### Build failures

Check that all build dependencies are listed in the formula. For Rust projects, ensure `rust` is listed as a build dependency.

### Formula audit warnings

Run `brew audit --strict netspeed-cli` to check for issues before submitting to homebrew-core.

## Useful Commands

```bash
# Test formula syntax
brew style netspeed-cli.rb

# Audit formula
brew audit --strict --online netspeed-cli

# Install from local formula file
brew install --build-from-source ./netspeed-cli.rb

# Uninstall
brew uninstall netspeed-cli
untap mapleDevJS/netspeed-cli
```

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Taps Documentation](https://docs.brew.sh/Taps)
- [Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
