use crate::theme::Theme;
use serde::Deserialize;
use std::path::PathBuf;

pub mod output;
pub mod source;
pub mod validate;

pub use output::{Format, OutputConfig};
pub use source::{ConfigSource, NetworkSource, OutputSource, ServerSource, TestSource};
pub use validate::{ValidationResult, get_config_path_internal, load_config_file, validate_config};

// ============================================================================
// Semantic config sub-structs (SRP: each struct has single responsibility)
// ============================================================================

/// Test execution configuration.
/// Controls which tests run and how (single vs multi-stream).
///
/// Named `TestSelection` to avoid collision with [`crate::test_config::TestConfig`]
/// which controls bandwidth measurement parameters (rounds, streams, payloads).
///
/// # Example
///
/// ```ignore
/// use netspeed_cli::config::{TestSelection, TestSource, File};
///
/// let source = TestSource {
///     no_download: Some(true),
///     single: Some(true),
///     ..Default::default()
/// };
/// let file_config = File::default();
/// let merge_bool = |cli: Option<bool>, file: Option<bool>| cli.or(file).unwrap_or(false);
///
/// let test = TestSelection::from_source(&source, &file_config, merge_bool);
/// assert!(test.no_download);
/// assert!(test.single);
/// assert!(!test.no_upload); // unset → false default
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestSelection {
    /// Do not perform download test
    pub no_download: bool,
    /// Do not perform upload test
    pub no_upload: bool,
    /// Use single connection instead of multiple
    pub single: bool,
}

impl TestSelection {
    /// Convert to merged test selection from CLI source and file config.
    #[must_use]
    pub(crate) fn from_source(
        source: &TestSource,
        file_config: &File,
        merge_bool: impl Fn(Option<bool>, Option<bool>) -> bool,
    ) -> Self {
        Self {
            no_download: merge_bool(source.no_download, file_config.no_download),
            no_upload: merge_bool(source.no_upload, file_config.no_upload),
            single: merge_bool(source.single, file_config.single),
        }
    }
}

/// Network and transport configuration.
/// Controls connection behavior, timeouts, and TLS settings.
///
/// # Example
///
/// ```ignore
/// use netspeed_cli::config::{NetworkConfig, NetworkSource, File};
///
/// let source = NetworkSource {
///     timeout: 30,
///     tls_version: Some("1.3".to_string()),
///     pin_certs: Some(true),
///     ..Default::default()
/// };
/// let file_config = File::default();
/// let merge_bool = |cli: Option<bool>, file: Option<bool>| cli.or(file).unwrap_or(false);
/// let merge_u64 = |cli: u64, file: Option<u64>, default: u64| {
///     if cli == default { file.unwrap_or(default) } else { cli }
/// };
///
/// let network = NetworkConfig::from_source(&source, &file_config, merge_bool, merge_u64);
/// assert_eq!(network.timeout, 30);
/// assert_eq!(network.tls_version, Some("1.3".to_string()));
/// assert!(network.pin_certs);
/// assert!(network.source.is_none()); // unset → None default
/// ```
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Source IP address to bind to
    pub source: Option<String>,
    /// HTTP request timeout in seconds
    pub timeout: u64,
    /// Path to custom CA certificate for TLS
    pub ca_cert: Option<String>,
    /// Minimum TLS version (1.2 or 1.3)
    pub tls_version: Option<String>,
    /// Restrict TLS connections to speedtest.net and ookla.com domains.
    pub pin_certs: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            source: None,
            timeout: 10,
            ca_cert: None,
            tls_version: None,
            pin_certs: false,
        }
    }
}

impl NetworkConfig {
    /// Convert from CLI source and file config to merged network config.
    #[must_use]
    pub(crate) fn from_source(
        source: &NetworkSource,
        file_config: &File,
        merge_bool: impl Fn(Option<bool>, Option<bool>) -> bool,
        merge_u64: impl Fn(u64, Option<u64>, u64) -> u64,
    ) -> Self {
        Self {
            source: source.source.clone(),
            timeout: merge_u64(source.timeout, file_config.timeout, 10),
            ca_cert: source.ca_cert.clone().or(file_config.ca_cert.clone()),
            tls_version: source
                .tls_version
                .clone()
                .or(file_config.tls_version.clone()),
            pin_certs: merge_bool(source.pin_certs, file_config.pin_certs),
        }
    }
}

/// Server selection configuration.
/// Controls which speedtest.net servers are used.
///
/// # Example
///
/// ```ignore
/// use netspeed_cli::config::{ServerSelection, ServerSource};
///
/// let source = ServerSource {
///     server_ids: vec!["1234".to_string(), "5678".to_string()],
///     exclude_ids: vec!["9999".to_string()],
/// };
///
/// let servers = ServerSelection::from_source(&source);
/// assert_eq!(servers.server_ids, vec!["1234", "5678"]);
/// assert_eq!(servers.exclude_ids, vec!["9999"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ServerSelection {
    /// Specific server IDs to use (empty = auto-select)
    pub server_ids: Vec<String>,
    /// Server IDs to exclude from selection
    pub exclude_ids: Vec<String>,
}

impl ServerSelection {
    /// Create from CLI source.
    #[must_use]
    pub(crate) fn from_source(source: &ServerSource) -> Self {
        Self {
            server_ids: source.server_ids.clone(),
            exclude_ids: source.exclude_ids.clone(),
        }
    }
}

// ============================================================================
// Main Config struct (composition of sub-structs)
// ============================================================================

#[derive(Debug, Default, Clone, Deserialize)]
pub struct File {
    pub no_download: Option<bool>,
    pub no_upload: Option<bool>,
    pub single: Option<bool>,
    pub bytes: Option<bool>,
    pub simple: Option<bool>,
    pub csv: Option<bool>,
    pub csv_delimiter: Option<char>,
    pub csv_header: Option<bool>,
    pub json: Option<bool>,
    pub timeout: Option<u64>,
    pub profile: Option<String>,
    pub theme: Option<String>,
    /// Custom user agent string (optional, defaults to browser-like UA).
    pub custom_user_agent: Option<String>,
    /// Enable strict config mode - invalid values cause warnings.
    pub strict: Option<bool>,
    /// Path to a custom CA certificate file for TLS verification.
    pub ca_cert: Option<String>,
    /// Minimum TLS version (1.2 or 1.3).
    pub tls_version: Option<String>,
    /// Restrict TLS connections to speedtest.net and ookla.com domains.
    pub pin_certs: Option<bool>,
}

/// Main configuration struct composed of semantic sub-structs.
///
/// Groups related configuration into cohesive units for better code organization:
/// - `OutputConfig` — output and display settings
/// - `TestSelection` — test execution controls
/// - `NetworkConfig` — network and TLS settings
/// - `ServerSelection` — server filtering options

#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Output and display configuration
    pub output: OutputConfig,
    /// Test execution controls
    pub test: TestSelection,
    /// Network and transport configuration
    pub network: NetworkConfig,
    /// Server selection criteria
    pub servers: ServerSelection,
    /// Custom user agent (file config only, not CLI)
    pub custom_user_agent: Option<String>,
    /// Strict validation mode
    pub strict: bool,
}

// ConfigProvider trait exposing read‑only config
pub trait ConfigProvider: Send + Sync {
    fn config(&self) -> &Config;
}

impl ConfigProvider for Config {
    fn config(&self) -> &Config {
        self
    }
}

impl Config {
    /// Build configuration from parsed CLI arguments and the config file.
    ///
    /// Converts `Args` into [`ConfigSource`] first (the sole CLI→config bridge),
    /// then builds sub-structs from the source abstraction.
    ///
    /// **Note:** This method does NOT call [`validate_and_report()`](Self::validate_and_report).
    /// For validation with warnings/errors, use
    /// [`from_args_with_file()`](Self::from_args_with_file) instead, which also
    /// avoids loading the config file twice.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clap::Parser;
    /// use netspeed_cli::cli::Args;
    /// use netspeed_cli::config::{Config, Format};
    ///
    /// // Parse CLI arguments (same as the main entry point)
    /// let args = Args::parse_from(["netspeed-cli", "--format", "json", "--timeout", "30"]);
    ///
    /// let config = Config::from_args(&args);
    /// assert_eq!(config.timeout(), 30);
    /// assert_eq!(config.format(), Some(Format::Json));
    /// assert!(!config.no_download()); // unset flags default to false
    /// ```
    #[allow(deprecated)]
    #[must_use]
    pub fn from_args(args: &crate::cli::Args) -> Self {
        let source = ConfigSource::from_args(args);
        Self::from_source(&source)
    }

    /// Build configuration from parsed CLI arguments with a pre-loaded config file.
    ///
    /// This is the preferred constructor in production code because it:
    /// 1. Eliminates double file loading (file config passed directly)
    /// 2. Returns a `ValidationResult` for reporting warnings/errors
    /// 3. Does NOT print or exit — callers control error handling
    ///
    /// For test code that doesn't need validation, use
    /// [`from_source()`](Self::from_source) with a hand-built [`ConfigSource`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clap::Parser;
    /// use netspeed_cli::cli::Args;
    /// use netspeed_cli::config::{Config, ConfigSource, ValidationResult};
    ///
    /// // Parse CLI arguments and convert to ConfigSource
    /// let args = Args::parse_from(["netspeed-cli", "--format", "json"]);
    /// let source = ConfigSource::from_args(&args);
    ///
    /// // Load config file (or pass None for defaults)
    /// let file_config = netspeed_cli::config::load_config_file();
    ///
    /// // Build config with validation results
    /// let (config, validation) = Config::from_args_with_file(&source, file_config);
    ///
    /// // Handle validation results
    /// for warning in &validation.warnings {
    ///     eprintln!("Warning: {warning}");
    /// }
    ///
    /// // Continue with config...
    /// assert_eq!(config.format(), Some(netspeed_cli::config::Format::Json));
    /// ```
    #[allow(deprecated)]
    #[must_use]
    pub fn from_args_with_file(
        source: &ConfigSource,
        file_config: Option<File>,
    ) -> (Self, ValidationResult) {
        let config = Self::from_source_with_file(source, file_config);

        // Check profile validation (produces warning, not error)
        let mut validation = ValidationResult::ok();
        if let Some(ref profile_name) = source.output.profile {
            if crate::profiles::UserProfile::validate(profile_name).is_err() {
                validation = validation.with_warning(format!(
                    "Unknown profile '{}'. Valid options: {}. Using 'power-user'.",
                    profile_name,
                    crate::profiles::UserProfile::VALID_NAMES.join(", ")
                ));
            }
        }

        (config, validation)
    }

    /// Build configuration from a [`ConfigSource`] and the config file.
    ///
    /// This constructor operates entirely within the config layer — no
    /// dependency on [`crate::cli::Args`]. Exposed as `pub` so that
    /// external test crates can construct a [`Config`] from a hand-built
    /// [`ConfigSource`] without going through CLI parsing.
    ///
    /// **Side-effect free**: This method does NOT print to stderr or exit.
    /// Call [`Config::validate_and_report()`](Self::validate_and_report) separately
    /// to emit validation warnings/errors, or use
    /// [`Config::from_args_with_file()`](Self::from_args_with_file) which handles
    /// validation automatically.
    ///
    /// # Merge Strategy
    ///
    /// Values are resolved with **CLI > file > hardcoded defaults** priority:
    ///
    /// - **`Option<bool>` fields** (e.g., `bytes`, `no_download`):
    ///   `cli.or(file).unwrap_or(false)` — CLI wins when `Some`, file is
    ///   the fallback, `false` when both are `None`.
    ///
    /// - **`Option<String>` fields** (e.g., `ca_cert`, `tls_version`):
    ///   `cli.or(file)` — CLI wins, file is the fallback, `None` when both
    ///   are absent.
    ///
    /// - **`u64` fields** (e.g., `timeout`):
    ///   If the CLI value equals the hardcoded default, the file value is
    ///   tried first; otherwise the CLI value wins. This lets `--timeout 10`
    ///   (the default) fall through to the file config while an explicit
    ///   `--timeout 30` always takes effect.
    ///
    /// - **`char` fields** (e.g., `csv_delimiter`):
    ///   If the CLI value equals the hardcoded default (`,`), the file value
    ///   is used; otherwise the CLI value wins.
    ///
    /// # Example
    ///
    /// External crates can build a [`Config`] without CLI parsing by
    /// constructing a [`ConfigSource`] from its sub-source structs:
    ///
    /// ```no_run
    /// use netspeed_cli::config::{
    ///     Config, ConfigSource, Format, NetworkSource, OutputSource,
    /// };
    ///
    /// let source = ConfigSource {
    ///     output: OutputSource {
    ///         format: Some(Format::Json),
    ///         quiet: Some(true),
    ///         ..Default::default()
    ///     },
    ///     network: NetworkSource {
    ///         timeout: 30,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// let config = Config::from_source(&source);
    /// assert_eq!(config.timeout(), 30);
    /// assert_eq!(config.format(), Some(Format::Json));
    /// ```
    #[must_use]
    pub fn from_source(source: &ConfigSource) -> Self {
        let file_config = load_config_file().unwrap_or_default();
        Self::from_source_with_file(source, Some(file_config))
    }

    /// Build configuration from a [`ConfigSource`] with a pre-loaded config file.
    ///
    /// This internal constructor accepts a pre-loaded file config to avoid
    /// redundant file loading. Use this when you already have the file config
    /// loaded (e.g., in `from_args_with_file()`).
    ///
    /// For test code without a pre-loaded config, use
    /// [`from_source()`](Self::from_source) which loads the file internally.
    #[must_use]
    pub(crate) fn from_source_with_file(source: &ConfigSource, file_config: Option<File>) -> Self {
        let file = file_config.unwrap_or_default();

        let strict = source.strict_config.unwrap_or(file.strict.unwrap_or(false));

        let merge_bool = |cli: Option<bool>, file: Option<bool>| cli.or(file).unwrap_or(false);
        let merge_u64 = |cli: u64, file: Option<u64>, default: u64| {
            if cli == default {
                file.unwrap_or(default)
            } else {
                cli
            }
        };

        // Build sub-structs from sub-sources and file config
        let output = OutputConfig::from_source(&source.output, &file, merge_bool);
        let test = TestSelection::from_source(&source.test, &file, merge_bool);
        let network = NetworkConfig::from_source(&source.network, &file, merge_bool, merge_u64);
        let servers = ServerSelection::from_source(&source.servers);

        Self {
            output,
            test,
            network,
            servers,
            custom_user_agent: file.custom_user_agent.clone(),
            strict,
        }
    }

    /// Validate configuration and emit warnings/errors.
    ///
    /// This method handles the side effects that were removed from
    /// [`from_source()`](Self::from_source): printing validation errors/warnings
    /// and exiting in strict mode. Call this after building config from CLI args.
    ///
    /// Returns the [`ValidationResult`] so callers can decide how to handle
    /// failures (e.g., exit in strict mode, log in normal mode).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use netspeed_cli::config::{Config, ConfigSource};
    ///
    /// let source = ConfigSource::default();
    /// let config = Config::from_source(&source);
    ///
    /// // Emit validation warnings/errors
    /// let result = config.validate_and_report(&source, None);
    /// for error in &result.errors {
    ///     eprintln!("Error: {error}");
    /// }
    /// for warning in &result.warnings {
    ///     eprintln!("Warning: {warning}");
    /// }
    ///
    /// // Exit if strict mode and validation failed
    /// if config.strict() && !result.valid {
    ///     std::process::exit(1);
    /// }
    /// ```
    #[must_use]
    pub fn validate_and_report(
        &self,
        source: &ConfigSource,
        file_config: Option<File>,
    ) -> ValidationResult {
        // Use pre-loaded file config if provided, otherwise load it
        let file = file_config.unwrap_or_else(|| load_config_file().unwrap_or_default());

        // Validate config file settings
        let mut validation = validate_config(&file);

        // Check profile validation (may differ from file config)
        if let Some(ref profile_name) = source.output.profile {
            if crate::profiles::UserProfile::validate(profile_name).is_err() {
                validation = validation.with_warning(format!(
                    "Unknown profile '{}'. Valid options: {}. Using 'power-user'.",
                    profile_name,
                    crate::profiles::UserProfile::VALID_NAMES.join(", ")
                ));
            }
        }

        validation
    }

    /// Whether test results should be saved to history.
    ///
    /// Machine-readable formats (JSON, JSONL, CSV) corrupt stdout when
    /// mixed with history output, so we skip saving in those cases.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::Config;
    ///
    /// let config = Config::default();
    /// // Default format is Detailed → should save history
    /// assert!(config.should_save_history());
    /// ```
    #[must_use]
    pub fn should_save_history(&self) -> bool {
        // Machine-readable formats corrupt stdout
        if self.format().is_some_and(|f| f.is_machine_readable()) {
            return false;
        }
        // Legacy format flags also skip history
        if self.json() || self.csv() {
            return false;
        }
        true
    }

    // ========================================================================
    // Output getters (delegates to output sub-struct)
    // ========================================================================

    // ========================================================================
    // Test execution getters (delegates to test sub-struct)
    // ========================================================================

    /// Whether to skip download test.
    #[must_use]
    pub fn no_download(&self) -> bool {
        self.test.no_download
    }

    /// Whether to skip upload test.
    #[must_use]
    pub fn no_upload(&self) -> bool {
        self.test.no_upload
    }

    /// Whether to use single connection mode.
    #[must_use]
    pub fn single(&self) -> bool {
        self.test.single
    }

    /// Whether to display values in bytes instead of bits.
    #[must_use]
    pub fn bytes(&self) -> bool {
        self.output.bytes
    }

    /// Whether to use simple output format.
    #[must_use]
    pub fn simple(&self) -> bool {
        self.output.simple
    }

    /// Whether to output in CSV format.
    #[must_use]
    pub fn csv(&self) -> bool {
        self.output.csv
    }

    /// Whether to output in JSON format.
    #[must_use]
    pub fn json(&self) -> bool {
        self.output.json
    }

    /// Whether to suppress all progress output.
    #[must_use]
    pub fn quiet(&self) -> bool {
        self.output.quiet
    }

    /// Whether to display server list and exit.
    #[must_use]
    pub fn list(&self) -> bool {
        self.output.list
    }

    /// Whether to use minimal ASCII-only output.
    #[must_use]
    pub fn minimal(&self) -> bool {
        self.output.minimal
    }

    /// The color theme for terminal output.
    #[must_use]
    pub fn theme(&self) -> Theme {
        self.output.theme
    }

    /// The CSV field delimiter.
    #[must_use]
    pub fn csv_delimiter(&self) -> char {
        self.output.csv_delimiter
    }

    /// Whether to include CSV headers.
    #[must_use]
    pub fn csv_header(&self) -> bool {
        self.output.csv_header
    }

    /// The user profile for customized output.
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.output.profile.as_deref()
    }

    /// The output format (supersedes legacy --json/--csv/--simple).
    #[must_use]
    pub fn format(&self) -> Option<Format> {
        self.output.format
    }

    // ========================================================================
    // Network getters (delegates to network sub-struct)
    // ========================================================================

    /// The HTTP request timeout in seconds.
    #[must_use]
    pub fn timeout(&self) -> u64 {
        self.network.timeout
    }

    /// The source IP address to bind to.
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.network.source.as_deref()
    }

    /// Path to custom CA certificate for TLS.
    #[must_use]
    pub fn ca_cert(&self) -> Option<&str> {
        self.network.ca_cert.as_deref()
    }

    /// Path to custom CA certificate as [`PathBuf`] (avoids double-allocation).
    ///
    /// Internal-only: external consumers should use [`ca_cert()`] which returns `Option<&str>`.
    #[must_use]
    pub(crate) fn ca_cert_path(&self) -> Option<PathBuf> {
        self.network.ca_cert.as_ref().map(PathBuf::from)
    }

    /// Minimum TLS version (1.2 or 1.3).
    #[must_use]
    pub fn tls_version(&self) -> Option<&str> {
        self.network.tls_version.as_deref()
    }

    /// Whether speedtest.net/ookla.com TLS domain restriction is enabled.
    #[must_use]
    pub fn pin_certs(&self) -> bool {
        self.network.pin_certs
    }

    // ========================================================================
    // Server selection getters (delegates to servers sub-struct)
    // ========================================================================

    /// Specific server IDs to use (empty = auto-select).
    #[must_use]
    pub fn server_ids(&self) -> &[String] {
        &self.servers.server_ids
    }

    /// Server IDs to exclude from selection.
    #[must_use]
    pub fn exclude_ids(&self) -> &[String] {
        &self.servers.exclude_ids
    }

    // ========================================================================
    // Top-level getters
    // ========================================================================

    /// Custom user agent string (file config only).
    #[must_use]
    pub fn custom_user_agent(&self) -> Option<&str> {
        self.custom_user_agent.as_deref()
    }

    /// Whether strict validation mode is enabled.
    #[must_use]
    pub fn strict(&self) -> bool {
        self.strict
    }
}

#[cfg(test)]
mod tests;
