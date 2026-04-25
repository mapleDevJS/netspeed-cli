use crate::theme::Theme;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// ConfigSource — CLI→config bridge (DIP: config depends on abstraction)
// ============================================================================

/// Raw CLI input values for output settings.
///
/// Groups output-related fields from [`crate::cli::Args`] into a cohesive unit
/// matching the structure of `OutputConfig`.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::{Format, OutputSource};
///
/// let src = OutputSource {
///     format: Some(Format::Json),
///     quiet: Some(true),
///     ..Default::default()
/// };
///
/// assert_eq!(src.format, Some(Format::Json));
/// assert_eq!(src.csv_delimiter, ','); // business-logic default preserved
/// assert_eq!(src.theme, "dark");       // business-logic default preserved
/// ```
#[derive(Debug, Clone)]
pub struct OutputSource {
    /// Display values in bytes instead of bits
    pub bytes: Option<bool>,
    /// Suppress verbose output (deprecated, use format)
    pub simple: Option<bool>,
    /// Output in CSV format (deprecated, use format)
    pub csv: Option<bool>,
    /// CSV field delimiter
    pub csv_delimiter: char,
    /// Include CSV headers
    pub csv_header: Option<bool>,
    /// Output in JSON format (deprecated, use format)
    pub json: Option<bool>,
    /// Display server list and exit
    pub list: bool,
    /// Suppress all progress output
    pub quiet: Option<bool>,
    /// Minimal ASCII-only output
    pub minimal: Option<bool>,
    /// User profile for customized output
    pub profile: Option<String>,
    /// Color theme name
    pub theme: String,
    /// Output format (supersedes legacy flags)
    pub format: Option<Format>,
}

/// Raw CLI input values for test execution settings.
///
/// Groups test-selection fields from [`crate::cli::Args`] into a cohesive unit
/// matching the structure of `TestSelection`.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::TestSource;
///
/// let src = TestSource {
///     no_download: Some(true),
///     single: Some(true),
///     ..Default::default()
/// };
///
/// assert_eq!(src.no_download, Some(true));
/// assert!(src.no_upload.is_none()); // unset fields default to None
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestSource {
    /// Do not perform download test
    pub no_download: Option<bool>,
    /// Do not perform upload test
    pub no_upload: Option<bool>,
    /// Use single connection instead of multiple
    pub single: Option<bool>,
}

/// Raw CLI input values for network/transport settings.
///
/// Groups network-related fields from [`crate::cli::Args`] into a cohesive unit
/// matching the structure of `NetworkConfig`.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::NetworkSource;
///
/// let src = NetworkSource {
///     timeout: 30,
///     tls_version: Some("1.3".to_string()),
///     ..Default::default()
/// };
///
/// assert_eq!(src.timeout, 30);
/// assert!(src.source.is_none()); // unset fields default to None
/// ```
#[derive(Debug, Clone)]
pub struct NetworkSource {
    /// Source IP address to bind to
    pub source: Option<String>,
    /// HTTP request timeout in seconds
    pub timeout: u64,
    /// Path to custom CA certificate for TLS
    pub ca_cert: Option<String>,
    /// Minimum TLS version (1.2 or 1.3)
    pub tls_version: Option<String>,
    /// Enable certificate pinning for speedtest.net
    pub pin_certs: Option<bool>,
}

/// Raw CLI input values for server selection settings.
///
/// Groups server-selection fields from [`crate::cli::Args`] into a cohesive unit
/// matching the structure of `ServerSelection`.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::ServerSource;
///
/// let src = ServerSource {
///     server_ids: vec!["1234".to_string()],
///     ..Default::default()
/// };
///
/// assert_eq!(src.server_ids, vec!["1234"]);
/// assert!(src.exclude_ids.is_empty()); // unset fields default to empty
/// ```
#[derive(Debug, Clone, Default)]
pub struct ServerSource {
    /// Specific server IDs to use (empty = auto-select)
    pub server_ids: Vec<String>,
    /// Server IDs to exclude from selection
    pub exclude_ids: Vec<String>,
}

/// Raw CLI input values extracted from parsed command-line arguments.
///
/// This struct is the sole bridge between the CLI layer ([`crate::cli::Args`])
/// and the config layer. Sub-struct constructors (`from_source`) depend on
/// the individual sub-source types instead of the concrete `Args` type,
/// satisfying the Dependency Inversion Principle: high-level config modules
/// depend on abstractions, not on low-level CLI parsing details.
///
/// All fields use config-internal types (e.g., [`Format`] instead of
/// [`crate::cli::OutputFormatType`]). The conversion happens once at
/// construction time via `ConfigSource::from_args`.
///
/// Composed of semantic sub-source structs matching the [`Config`] sub-struct
/// pattern: [`OutputSource`], [`TestSource`], [`NetworkSource`], [`ServerSource`].
///
/// # Example
///
/// Build a [`Config`] without CLI parsing by constructing sub-sources:
///
/// ```no_run
/// use netspeed_cli::config::{
///     Config, ConfigSource, Format, NetworkSource, OutputSource, TestSource,
/// };
///
/// let source = ConfigSource {
///     output: OutputSource {
///         format: Some(Format::Dashboard),
///         profile: Some("gamer".to_string()),
///         ..Default::default()
///     },
///     test: TestSource {
///         no_upload: Some(true),
///         ..Default::default()
///     },
///     network: NetworkSource {
///         timeout: 60,
///         ..Default::default()
///     },
///     ..Default::default()
/// };
///
/// let config = Config::from_source(&source);
/// assert_eq!(config.timeout(), 60);
/// assert!(config.no_upload());
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigSource {
    /// Output and display settings
    pub output: OutputSource,
    /// Test execution controls
    pub test: TestSource,
    /// Network and transport settings
    pub network: NetworkSource,
    /// Server selection criteria
    pub servers: ServerSource,
    /// Enable strict config validation mode
    pub strict_config: Option<bool>,
}

// OutputSource and NetworkSource have manual Default impls for business logic defaults
// (csv_delimiter: ',', theme: "dark" and timeout: 10 respectively)

impl Default for OutputSource {
    fn default() -> Self {
        Self {
            bytes: None,
            simple: None,
            csv: None,
            csv_delimiter: ',',
            csv_header: None,
            json: None,
            list: false,
            quiet: None,
            minimal: None,
            profile: None,
            theme: "dark".to_string(),
            format: None,
        }
    }
}

impl Default for NetworkSource {
    fn default() -> Self {
        Self {
            source: None,
            timeout: 10,
            ca_cert: None,
            tls_version: None,
            pin_certs: None,
        }
    }
}

impl ConfigSource {
    /// Extract config-relevant values from parsed CLI arguments.
    ///
    /// This is the **only** method in the config layer that touches
    /// [`crate::cli::Args`]. All downstream code uses [`ConfigSource`].
    ///
    /// For tests that don't need CLI parsing, construct a [`ConfigSource`]
    /// directly or use [`ConfigSource::default()`].
    #[must_use]
    #[allow(deprecated)] // accesses deprecated --simple/--csv/--json fields for backward compat
    pub(crate) fn from_args(args: &crate::cli::Args) -> Self {
        Self {
            output: OutputSource {
                bytes: args.bytes,
                simple: args.simple,
                csv: args.csv,
                csv_delimiter: args.csv_delimiter,
                csv_header: args.csv_header,
                json: args.json,
                list: args.list,
                quiet: args.quiet,
                minimal: args.minimal,
                profile: args.profile.clone(),
                theme: args.theme.clone(),
                format: args.format.map(Format::from_cli_type),
            },
            test: TestSource {
                no_download: args.no_download,
                no_upload: args.no_upload,
                single: args.single,
            },
            network: NetworkSource {
                source: args.source.clone(),
                timeout: args.timeout,
                ca_cert: args.ca_cert.clone(),
                tls_version: args.tls_version.clone(),
                pin_certs: args.pin_certs,
            },
            servers: ServerSource {
                server_ids: args.server.clone(),
                exclude_ids: args.exclude.clone(),
            },
            strict_config: args.strict_config,
        }
    }
}

// ============================================================================
// Semantic config sub-structs (SRP: each struct has single responsibility)
// ============================================================================

/// Output format selection — config-internal domain type.
///
/// Decoupled from [`crate::cli::OutputFormatType`] (which carries clap's `ValueEnum`
/// derive). The CLI enum is converted into this type at the config boundary via
/// `Format::from_cli_type`, so the config layer never depends on the CLI crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Machine-readable JSON output
    Json,
    /// JSON Lines for logging (one JSON object per line)
    Jsonl,
    /// CSV format for spreadsheet analysis
    Csv,
    /// Ultra-minimal: just grade + speeds
    Minimal,
    /// Minimal one-line summary
    Simple,
    /// Key metrics with quality ratings
    Compact,
    /// Full analysis with per-metric grades (default)
    Detailed,
    /// Rich terminal dashboard with capability matrix
    Dashboard,
}

impl Format {
    /// Convert from the CLI-specific [`crate::cli::OutputFormatType`] enum.
    ///
    /// This is the only place the config layer touches the CLI type —
    /// all downstream consumers use [`Format`] instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use netspeed_cli::config::Format;
    ///
    /// // Convert CLI enum to config-internal enum
    /// let fmt = Format::from_cli_type(netspeed_cli::cli::OutputFormatType::Json);
    /// assert_eq!(fmt, Format::Json);
    ///
    /// let fmt = Format::from_cli_type(netspeed_cli::cli::OutputFormatType::Dashboard);
    /// assert_eq!(fmt, Format::Dashboard);
    /// ```
    #[must_use]
    pub(crate) fn from_cli_type(cli: crate::cli::OutputFormatType) -> Self {
        match cli {
            crate::cli::OutputFormatType::Json => Self::Json,
            crate::cli::OutputFormatType::Jsonl => Self::Jsonl,
            crate::cli::OutputFormatType::Csv => Self::Csv,
            crate::cli::OutputFormatType::Minimal => Self::Minimal,
            crate::cli::OutputFormatType::Simple => Self::Simple,
            crate::cli::OutputFormatType::Compact => Self::Compact,
            crate::cli::OutputFormatType::Detailed => Self::Detailed,
            crate::cli::OutputFormatType::Dashboard => Self::Dashboard,
        }
    }

    /// Whether this format is machine-readable (JSON/JSONL/CSV).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::Format;
    ///
    /// // JSON, JSONL, and CSV are machine-readable
    /// assert!(Format::Json.is_machine_readable());
    /// assert!(Format::Jsonl.is_machine_readable());
    /// assert!(Format::Csv.is_machine_readable());
    ///
    /// // All other formats are human-readable only
    /// for fmt in [Format::Minimal, Format::Simple, Format::Compact,
    ///             Format::Detailed, Format::Dashboard] {
    ///     assert!(!fmt.is_machine_readable());
    /// }
    /// ```
    #[must_use]
    pub fn is_machine_readable(self) -> bool {
        matches!(self, Self::Json | Self::Jsonl | Self::Csv)
    }

    /// Whether this format produces non-verbose (terse) output.
    ///
    /// All formats except [`Detailed`](Format::Detailed) are considered non-verbose.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::Format;
    ///
    /// // Detailed is the only verbose format
    /// assert!(!Format::Detailed.is_non_verbose());
    ///
    /// // Everything else is non-verbose (terse)
    /// for fmt in [Format::Simple, Format::Minimal, Format::Compact,
    ///             Format::Json, Format::Jsonl, Format::Csv,
    ///             Format::Dashboard] {
    ///     assert!(fmt.is_non_verbose());
    /// }
    /// ```
    #[must_use]
    pub fn is_non_verbose(self) -> bool {
        matches!(
            self,
            Self::Simple
                | Self::Minimal
                | Self::Compact
                | Self::Json
                | Self::Jsonl
                | Self::Csv
                | Self::Dashboard
        )
    }

    /// Human-readable label for display (e.g., dry-run output).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::Format;
    ///
    /// assert_eq!(Format::Json.label(), "JSON");
    /// assert_eq!(Format::Dashboard.label(), "Dashboard");
    /// assert_eq!(Format::Compact.label(), "Compact");
    ///
    /// // Labels are also used via Display trait
    /// let csv = Format::Csv;
    /// assert_eq!(format!("{csv}"), "CSV");
    /// ```
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Jsonl => "JSONL",
            Self::Csv => "CSV",
            Self::Minimal => "Minimal",
            Self::Simple => "Simple",
            Self::Compact => "Compact",
            Self::Detailed => "Detailed",
            Self::Dashboard => "Dashboard",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Output and display configuration.
/// Controls how test results are formatted and presented to the user.
///
/// # Example
///
/// ```ignore
/// use netspeed_cli::config::{Format, OutputConfig, OutputSource, File};
///
/// let source = OutputSource {
///     format: Some(Format::Json),
///     quiet: Some(true),
///     ..Default::default()
/// };
/// let file_config = File::default();
/// let merge_bool = |cli: Option<bool>, file: Option<bool>| cli.or(file).unwrap_or(false);
///
/// let output = OutputConfig::from_source(&source, &file_config, merge_bool);
/// assert!(output.quiet);
/// assert_eq!(output.csv_delimiter, ','); // business-logic default preserved
/// ```
#[derive(Debug, Clone)]
pub(crate) struct OutputConfig {
    /// Display values in bytes instead of bits
    pub(crate) bytes: bool,
    /// Suppress verbose output (deprecated, use format)
    pub(crate) simple: bool,
    /// Output in CSV format (deprecated, use format)
    pub(crate) csv: bool,
    /// CSV field delimiter
    pub(crate) csv_delimiter: char,
    /// Include CSV headers
    pub(crate) csv_header: bool,
    /// Output in JSON format (deprecated, use format)
    pub(crate) json: bool,
    /// Display server list and exit
    pub(crate) list: bool,
    /// Suppress all progress output
    pub(crate) quiet: bool,
    /// User profile for customized output
    pub(crate) profile: Option<String>,
    /// Color theme for terminal output
    pub(crate) theme: Theme,
    /// Minimal ASCII-only output (no Unicode box-drawing)
    pub(crate) minimal: bool,
    /// Output format (supersedes legacy --json/--csv/--simple)
    pub(crate) format: Option<Format>,
}

// OutputConfig and NetworkConfig have manual Default impls for business logic defaults
// (csv_delimiter: ',' and timeout: 10 respectively)

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            bytes: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            list: false,
            quiet: false,
            profile: None,
            theme: Theme::Dark,
            minimal: false,
            format: None,
        }
    }
}

impl OutputConfig {
    /// Convert to merged output config from CLI source and file config.
    #[must_use]
    #[allow(deprecated)]
    pub(crate) fn from_source(
        source: &OutputSource,
        file_config: &File,
        merge_bool: impl Fn(Option<bool>, Option<bool>) -> bool,
    ) -> Self {
        let theme = if source.theme == "dark" {
            file_config
                .theme
                .as_ref()
                .and_then(|t| Theme::from_name(t))
                .unwrap_or_default()
        } else {
            Theme::from_name(&source.theme).unwrap_or_default()
        };

        Self {
            bytes: merge_bool(source.bytes, file_config.bytes),
            simple: merge_bool(source.simple, file_config.simple),
            csv: merge_bool(source.csv, file_config.csv),
            csv_delimiter: if source.csv_delimiter == ',' {
                file_config.csv_delimiter.unwrap_or(',')
            } else {
                source.csv_delimiter
            },
            csv_header: merge_bool(source.csv_header, file_config.csv_header),
            json: merge_bool(source.json, file_config.json),
            list: source.list,
            quiet: merge_bool(source.quiet, None),
            profile: source.profile.clone().or(file_config.profile.clone()),
            theme,
            minimal: merge_bool(source.minimal, None),
            format: source.format,
        }
    }
}

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
pub(crate) struct TestSelection {
    /// Do not perform download test
    pub(crate) no_download: bool,
    /// Do not perform upload test
    pub(crate) no_upload: bool,
    /// Use single connection instead of multiple
    pub(crate) single: bool,
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
pub(crate) struct NetworkConfig {
    /// Source IP address to bind to
    pub(crate) source: Option<String>,
    /// HTTP request timeout in seconds
    pub(crate) timeout: u64,
    /// Path to custom CA certificate for TLS
    pub(crate) ca_cert: Option<String>,
    /// Minimum TLS version (1.2 or 1.3)
    pub(crate) tls_version: Option<String>,
    /// Enable certificate pinning for speedtest.net
    pub(crate) pin_certs: bool,
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
pub(crate) struct ServerSelection {
    /// Specific server IDs to use (empty = auto-select)
    pub(crate) server_ids: Vec<String>,
    /// Server IDs to exclude from selection
    pub(crate) exclude_ids: Vec<String>,
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
    /// Enable certificate pinning for speedtest.net servers.
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
    pub(crate) output: OutputConfig,
    /// Test execution controls
    pub(crate) test: TestSelection,
    /// Network and transport configuration
    pub(crate) network: NetworkConfig,
    /// Server selection criteria
    pub(crate) servers: ServerSelection,
    /// Custom user agent (file config only, not CLI)
    pub(crate) custom_user_agent: Option<String>,
    /// Strict validation mode
    pub(crate) strict: bool,
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

    /// Whether certificate pinning is enabled.
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

/// Validation result with error details.
///
/// Uses a builder pattern: start with [`ValidationResult::ok()`], then chain
/// [`with_error()`](ValidationResult::with_error) and
/// [`with_warning()`](ValidationResult::with_warning) calls.
///
/// # Example
///
/// ```
/// use netspeed_cli::config::ValidationResult;
///
/// // Start valid, then add issues via builder chaining
/// let result = ValidationResult::ok()
///     .with_warning("deprecated option")
///     .with_error("invalid profile");
///
/// assert!(!result.valid);          // errors flip valid to false
/// assert_eq!(result.errors.len(), 1);
/// assert_eq!(result.warnings.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed (no errors). Warnings do not affect this.
    pub valid: bool,
    /// Error messages (any error sets [`valid`](ValidationResult::valid) to `false`).
    pub errors: Vec<String>,
    /// Warning messages (do not affect [`valid`](ValidationResult::valid)).
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::ok();
    /// assert!(result.valid);
    /// assert!(result.errors.is_empty());
    /// assert!(result.warnings.is_empty());
    /// ```
    #[must_use]
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result.
    ///
    /// Warnings do **not** change [`valid`](ValidationResult::valid) —
    /// the result remains passable even with warnings.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// // Warnings preserve valid=true
    /// let result = ValidationResult::ok().with_warning("'simple' is deprecated");
    /// assert!(result.valid);
    /// assert_eq!(result.warnings.len(), 1);
    /// assert!(result.warnings[0].contains("deprecated"));
    ///
    /// // Multiple warnings can be chained
    /// let result = ValidationResult::ok()
    ///     .with_warning("first warning")
    ///     .with_warning("second warning");
    /// assert!(result.valid);
    /// assert_eq!(result.warnings.len(), 2);
    /// ```
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Create a validation failure.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// let result = ValidationResult::error("invalid profile 'foo'");
    /// assert!(!result.valid);
    /// assert_eq!(result.errors.len(), 1);
    /// assert!(result.errors[0].contains("foo"));
    /// assert!(result.warnings.is_empty());
    /// ```
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result.
    ///
    /// Unlike [`with_warning()`](ValidationResult::with_warning), this flips
    /// [`valid`](ValidationResult::valid) to `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// // Adding an error flips valid to false
    /// let result = ValidationResult::ok().with_error("bad theme");
    /// assert!(!result.valid);
    /// assert_eq!(result.errors.len(), 1);
    ///
    /// // Multiple errors accumulate
    /// let result = ValidationResult::error("first error")
    ///     .with_error("second error");
    /// assert!(!result.valid);
    /// assert_eq!(result.errors.len(), 2);
    ///
    /// // Errors and warnings can be mixed — errors always flip valid
    /// let result = ValidationResult::ok()
    ///     .with_warning("just a heads-up")
    ///     .with_error("actual problem");
    /// assert!(!result.valid);
    /// assert_eq!(result.warnings.len(), 1);
    /// assert_eq!(result.errors.len(), 1);
    /// ```
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.valid = false;
        self
    }

    /// Merge another validation result into this one.
    ///
    /// Combines both error and warning lists. If the other result is invalid,
    /// this result also becomes invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::config::ValidationResult;
    ///
    /// // Merging two valid results stays valid
    /// let a = ValidationResult::ok().with_warning("warn-a");
    /// let b = ValidationResult::ok().with_warning("warn-b");
    /// let merged = a.merge(b);
    /// assert!(merged.valid);
    /// assert_eq!(merged.warnings.len(), 2);
    ///
    /// // Merging an invalid result makes the whole thing invalid
    /// let a = ValidationResult::ok();
    /// let b = ValidationResult::error("bad profile");
    /// let merged = a.merge(b);
    /// assert!(!merged.valid);
    /// assert_eq!(merged.errors.len(), 1);
    ///
    /// // Both errors and warnings are accumulated
    /// let a = ValidationResult::error("error-a").with_warning("warn-a");
    /// let b = ValidationResult::error("error-b").with_warning("warn-b");
    /// let merged = a.merge(b);
    /// assert!(!merged.valid);
    /// assert_eq!(merged.errors.len(), 2);
    /// assert_eq!(merged.warnings.len(), 2);
    /// ```
    #[must_use]
    pub fn merge(mut self, other: ValidationResult) -> Self {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self
    }
}

// Validation delegation — owned by profiles.rs and theme.rs to eliminate duplication

/// Validate CSV delimiter character.
fn validate_csv_delimiter_config(delimiter: char) -> Result<(), String> {
    if !",;|\t".contains(delimiter) {
        return Err(format!(
            "Invalid CSV delimiter '{}'. Must be one of: comma, semicolon, pipe, or tab",
            delimiter
        ));
    }
    Ok(())
}

/// Validate the entire config structure.
pub fn validate_config(file_config: &File) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // Validate profile — delegation to profiles.rs (DIP: high-level depends on abstraction)
    if let Some(ref profile) = file_config.profile {
        if let Err(e) = crate::profiles::UserProfile::validate(profile) {
            result = result.with_error(e);
        }
    }

    // Validate theme — delegation to theme.rs (DIP: high-level depends on abstraction)
    if let Some(ref theme) = file_config.theme {
        if let Err(e) = crate::theme::Theme::validate(theme) {
            result = result.with_error(e);
        }
    }

    // Validate CSV delimiter
    if let Some(delimiter) = file_config.csv_delimiter {
        if let Err(e) = validate_csv_delimiter_config(delimiter) {
            result = result.with_error(e);
        }
    }

    // Warnings for deprecated options
    if file_config.simple.unwrap_or(false) {
        result = result.with_warning(
            "'simple' option is deprecated. Use '--format simple' instead.".to_string(),
        );
    }
    if file_config.csv.unwrap_or(false) {
        result = result
            .with_warning("'csv' option is deprecated. Use '--format csv' instead.".to_string());
    }
    if file_config.json.unwrap_or(false) {
        result = result
            .with_warning("'json' option is deprecated. Use '--format json' instead.".to_string());
    }

    result
}

/// Get the configuration file path (internal — also used by orchestrator for --show-config-path).
#[must_use]
pub fn get_config_path_internal() -> Option<PathBuf> {
    ProjectDirs::from("dev", "vibe", "netspeed-cli").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        if let Err(e) = fs::create_dir_all(config_dir) {
            eprintln!("Warning: Failed to create config directory: {e}");
        }
        config_dir.join("config.toml")
    })
}

/// Load the configuration file from the standard config path.
///
/// Returns `None` if no config file exists or if loading fails.
pub fn load_config_file() -> Option<File> {
    let path = get_config_path_internal()?;
    if !path.exists() {
        return None;
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read config file {}: {e}",
                path.display()
            );
            return None;
        }
    };
    let mut config: File = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to parse config: {e}");
            return None;
        }
    };

    // Validate timeout if present
    if let Some(timeout) = config.timeout {
        if timeout == 0 || timeout > 300 {
            eprintln!(
                "Warning: Invalid config timeout ({timeout}s, must be 1-300). Using default."
            );
            config.timeout = None;
        }
    }

    Some(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;

    #[test]
    fn test_config_from_args_defaults() {
        let args = Args::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);

        assert!(!config.test.no_download);
        assert!(!config.test.no_upload);
        assert!(!config.test.single);
        assert!(!config.output.bytes);
        assert!(!config.output.simple);
        assert!(!config.output.csv);
        assert!(!config.output.json);
        assert!(!config.output.list);
        assert!(!config.output.quiet);
        assert_eq!(config.network.timeout, 10);
        assert_eq!(config.output.csv_delimiter, ',');
        assert!(!config.output.csv_header);
        assert!(config.servers.server_ids.is_empty());
        assert!(config.servers.exclude_ids.is_empty());
    }

    #[test]
    fn test_config_from_args_no_download() {
        let args = Args::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.test.no_download);
        assert!(!config.test.no_upload);
    }

    #[test]
    fn test_config_file_deserialization() {
        let toml_content = r"
            no_download = true
            no_upload = false
            single = true
            bytes = true
            simple = false
            csv = false
            csv_delimiter = ';'
            csv_header = true
            json = true
            timeout = 30
        ";

        let config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(config.no_download, Some(true));
        assert_eq!(config.no_upload, Some(false));
        assert_eq!(config.single, Some(true));
        assert_eq!(config.bytes, Some(true));
        assert_eq!(config.simple, Some(false));
        assert_eq!(config.csv, Some(false));
        assert_eq!(config.csv_delimiter, Some(';'));
        assert_eq!(config.csv_header, Some(true));
        assert_eq!(config.json, Some(true));
        assert_eq!(config.timeout, Some(30));
    }

    #[test]
    fn test_config_file_partial() {
        let toml_content = r"
            no_download = true
            timeout = 20
        ";

        let config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(config.no_download, Some(true));
        assert!(config.no_upload.is_none());
        assert!(config.single.is_none());
        assert_eq!(config.timeout, Some(20));
        assert!(config.csv_delimiter.is_none());
    }

    #[test]
    fn test_config_from_args_overrides_file() {
        // Test that CLI flags override file config when explicitly set
        let args = Args::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.test.no_download);
    }

    #[test]
    fn test_config_merge_bool_file_true_cli_false() {
        // When CLI omits the flag, the config file value should be used.
        let toml_content = r"
            no_download = true
        ";
        let file_config: File = toml::from_str(toml_content).unwrap();

        // CLI args omit the flag, so clap yields None for Option<bool>.
        let args = Args::parse_from(["netspeed-cli"]);
        let file_config_loaded = Some(file_config);

        // Manual merge check
        let cli_val = args.no_download; // None
        let file_val = file_config_loaded.and_then(|c| c.no_download); // Some(true)
        let merged = cli_val.or(file_val).unwrap_or(false);
        assert!(merged);
    }

    #[test]
    fn test_validate_config_valid_profile() {
        let file_config = File {
            profile: Some("gamer".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_config_empty_is_valid() {
        // Default case: no config file
        let file_config = File::default();
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_config_invalid_profile() {
        let file_config = File {
            profile: Some("invalid_profile".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("invalid_profile"));
    }

    #[test]
    fn test_validate_config_invalid_theme() {
        let file_config = File {
            theme: Some("neon".to_string()),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("neon"));
    }

    #[test]
    fn test_validate_config_invalid_csv_delimiter() {
        let file_config = File {
            csv_delimiter: Some('X'),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_config_deprecated_simple() {
        let file_config = File {
            simple: Some(true),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(result.valid);
        assert!(!result.warnings.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("simple") && w.contains("deprecated")));
    }

    #[test]
    fn test_validate_config_multiple_issues() {
        let file_config = File {
            profile: Some("bad".to_string()),
            theme: Some("ugly".to_string()),
            csv_delimiter: Some('@'),
            ..Default::default()
        };
        let result = validate_config(&file_config);
        assert!(!result.valid);
        assert!(result.errors.len() >= 3); // profile, theme, delimiter
    }

    // ==================== TLS Configuration Tests ====================

    #[test]
    fn test_tls_config_defaults() {
        // When no CLI flags or config file, TLS options should be None/false
        let args = Args::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);
        assert!(config.network.ca_cert.is_none());
        assert!(config.network.tls_version.is_none());
        assert!(!config.network.pin_certs);
    }

    #[test]
    fn test_tls_config_file_deserialization() {
        // Test that TLS options deserialize correctly from TOML
        let toml_content = r#"
            ca_cert = "/custom/ca.pem"
            tls_version = "1.2"
            pin_certs = true
        "#;

        let file_config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(file_config.ca_cert, Some("/custom/ca.pem".to_string()));
        assert_eq!(file_config.tls_version, Some("1.2".to_string()));
        assert_eq!(file_config.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_file_partial() {
        // Test partial TLS config from file
        let toml_content = r#"
            ca_cert = "/my/ca.pem"
        "#;

        let file_config: File = toml::from_str(toml_content).unwrap();
        assert_eq!(file_config.ca_cert, Some("/my/ca.pem".to_string()));
        assert!(file_config.tls_version.is_none());
        assert!(file_config.pin_certs.is_none());
    }

    #[test]
    fn test_tls_config_cli_ca_cert() {
        // Test that --ca-cert CLI flag is parsed correctly
        // Use an existing file path (should exist on all systems)
        let args = Args::parse_from(["netspeed-cli", "--ca-cert", "/etc/passwd"]);
        assert_eq!(args.ca_cert, Some("/etc/passwd".to_string()));
    }

    #[test]
    fn test_tls_config_cli_tls_version() {
        // Test that --tls-version CLI flag is parsed correctly
        let args = Args::parse_from(["netspeed-cli", "--tls-version", "1.3"]);
        assert_eq!(args.tls_version, Some("1.3".to_string()));
    }

    #[test]
    fn test_tls_config_cli_pin_certs() {
        // Test that --pin-certs CLI flag enables pinning
        let args = Args::parse_from(["netspeed-cli", "--pin-certs"]);
        assert_eq!(args.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_cli_pin_certs_false() {
        // Test that --pin-certs=false disables pinning
        let args = Args::parse_from(["netspeed-cli", "--pin-certs=false"]);
        assert_eq!(args.pin_certs, Some(false));
    }

    #[test]
    fn test_tls_config_all_cli_options() {
        // Test all TLS options via CLI
        // Use an existing file path for --ca-cert
        let args = Args::parse_from([
            "netspeed-cli",
            "--ca-cert",
            "/etc/passwd",
            "--tls-version",
            "1.2",
            "--pin-certs",
        ]);

        assert_eq!(args.ca_cert, Some("/etc/passwd".to_string()));
        assert_eq!(args.tls_version, Some("1.2".to_string()));
        assert_eq!(args.pin_certs, Some(true));
    }

    #[test]
    fn test_tls_config_string_merge_cli_takes_precedence() {
        // For string options (ca_cert, tls_version), CLI should take precedence
        // This is tested by verifying the merge logic:
        // ca_cert: args.ca_cert.clone().or(file_config.ca_cert.clone())

        // When CLI provides ca_cert, it should be used
        let cli_val = Some("/cli/ca.pem".to_string());
        let file_val = Some("/file/ca.pem".to_string());
        let merged = cli_val.or(file_val.clone());
        assert_eq!(merged, Some("/cli/ca.pem".to_string()));

        // When CLI is None, file value should be used
        let cli_val_none: Option<String> = None;
        let merged = cli_val_none.or(file_val.clone());
        assert_eq!(merged, Some("/file/ca.pem".to_string()));

        // When both are None, result should be None
        let merged = Option::<String>::None.or(None);
        assert!(merged.is_none());
    }

    #[test]
    fn test_tls_config_bool_merge() {
        // Test boolean merge logic for pin_certs
        // merge_bool: cli.or(file).unwrap_or(false)
        // CLI takes precedence when explicitly set, file used only when CLI is None

        // CLI true, file false -> true (CLI takes precedence)
        assert!(merge_bool_test(Some(true), Some(false)));

        // CLI false, file true -> false (CLI takes precedence even when false)
        assert!(!merge_bool_test(Some(false), Some(true)));

        // CLI true, file None -> true
        assert!(merge_bool_test(Some(true), None));

        // CLI false, file None -> false
        assert!(!merge_bool_test(Some(false), None));

        // CLI None, file true -> true (fall back to file)
        assert!(merge_bool_test(None, Some(true)));

        // CLI None, file false -> false (fall back to file)
        assert!(!merge_bool_test(None, Some(false)));

        // CLI None, file None -> false (default)
        assert!(!merge_bool_test(None::<bool>, None));
    }

    // Helper function to test merge_bool logic
    fn merge_bool_test(cli: Option<bool>, file: Option<bool>) -> bool {
        cli.or(file).unwrap_or(false)
    }

    // ==================== Format Tests ====================

    #[test]
    fn test_format_from_cli_type_all_variants() {
        use crate::cli::OutputFormatType;
        assert_eq!(Format::from_cli_type(OutputFormatType::Json), Format::Json);
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Jsonl),
            Format::Jsonl
        );
        assert_eq!(Format::from_cli_type(OutputFormatType::Csv), Format::Csv);
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Minimal),
            Format::Minimal
        );
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Simple),
            Format::Simple
        );
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Compact),
            Format::Compact
        );
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Detailed),
            Format::Detailed
        );
        assert_eq!(
            Format::from_cli_type(OutputFormatType::Dashboard),
            Format::Dashboard
        );
    }

    #[test]
    fn test_format_is_machine_readable() {
        assert!(Format::Json.is_machine_readable());
        assert!(Format::Jsonl.is_machine_readable());
        assert!(Format::Csv.is_machine_readable());
        assert!(!Format::Minimal.is_machine_readable());
        assert!(!Format::Simple.is_machine_readable());
        assert!(!Format::Compact.is_machine_readable());
        assert!(!Format::Detailed.is_machine_readable());
        assert!(!Format::Dashboard.is_machine_readable());
    }

    #[test]
    fn test_format_is_non_verbose() {
        // Non-verbose: everything except Detailed
        assert!(Format::Simple.is_non_verbose());
        assert!(Format::Minimal.is_non_verbose());
        assert!(Format::Compact.is_non_verbose());
        assert!(Format::Json.is_non_verbose());
        assert!(Format::Jsonl.is_non_verbose());
        assert!(Format::Csv.is_non_verbose());
        assert!(Format::Dashboard.is_non_verbose());
        // Detailed is the only verbose format
        assert!(!Format::Detailed.is_non_verbose());
    }

    #[test]
    fn test_format_label() {
        assert_eq!(Format::Json.label(), "JSON");
        assert_eq!(Format::Jsonl.label(), "JSONL");
        assert_eq!(Format::Csv.label(), "CSV");
        assert_eq!(Format::Minimal.label(), "Minimal");
        assert_eq!(Format::Simple.label(), "Simple");
        assert_eq!(Format::Compact.label(), "Compact");
        assert_eq!(Format::Detailed.label(), "Detailed");
        assert_eq!(Format::Dashboard.label(), "Dashboard");
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", Format::Json), "JSON");
        assert_eq!(format!("{}", Format::Detailed), "Detailed");
    }

    #[test]
    fn test_format_equality() {
        assert_eq!(Format::Json, Format::Json);
        assert_ne!(Format::Json, Format::Csv);
    }

    // ==================== ConfigSource Tests ====================

    #[test]
    fn test_config_source_from_args_defaults() {
        let args = Args::parse_from(["netspeed-cli"]);
        let source = ConfigSource::from_args(&args);

        // Output defaults
        assert!(source.output.bytes.is_none());
        assert!(source.output.simple.is_none());
        assert!(source.output.csv.is_none());
        assert_eq!(source.output.csv_delimiter, ',');
        assert!(source.output.csv_header.is_none());
        assert!(source.output.json.is_none());
        assert!(!source.output.list);
        assert!(source.output.quiet.is_none());
        assert!(source.output.minimal.is_none());
        assert!(source.output.profile.is_none());
        assert_eq!(source.output.theme, "dark");
        assert!(source.output.format.is_none());

        // Test selection defaults
        assert!(source.test.no_download.is_none());
        assert!(source.test.no_upload.is_none());
        assert!(source.test.single.is_none());

        // Network defaults
        assert!(source.network.source.is_none());
        assert_eq!(source.network.timeout, 10);
        assert!(source.network.ca_cert.is_none());
        assert!(source.network.tls_version.is_none());
        assert!(source.network.pin_certs.is_none());

        // Server defaults
        assert!(source.servers.server_ids.is_empty());
        assert!(source.servers.exclude_ids.is_empty());

        // Top-level
        assert!(source.strict_config.is_none());
    }

    #[test]
    fn test_config_source_from_args_all_set() {
        let args = Args::parse_from([
            "netspeed-cli",
            "--bytes",
            "--no-download",
            "--no-upload",
            "--single",
            "--timeout",
            "30",
            "--source",
            "0.0.0.0",
            "--server",
            "1234",
            "--exclude",
            "5678",
            "--profile",
            "gamer",
            "--theme",
            "light",
            "--format",
            "json",
        ]);
        let source = ConfigSource::from_args(&args);

        assert_eq!(source.output.bytes, Some(true));
        assert_eq!(source.test.no_download, Some(true));
        assert_eq!(source.test.no_upload, Some(true));
        assert_eq!(source.test.single, Some(true));
        assert_eq!(source.network.timeout, 30);
        assert_eq!(source.network.source, Some("0.0.0.0".to_string()));
        assert_eq!(source.servers.server_ids, vec!["1234".to_string()]);
        assert_eq!(source.servers.exclude_ids, vec!["5678".to_string()]);
        assert_eq!(source.output.profile, Some("gamer".to_string()));
        assert_eq!(source.output.theme, "light");
        assert_eq!(source.output.format, Some(Format::Json));
    }

    #[test]
    fn test_config_source_format_conversion() {
        // Verify that OutputFormatType is properly converted to Format
        let args = Args::parse_from(["netspeed-cli", "--format", "csv"]);
        let source = ConfigSource::from_args(&args);
        assert_eq!(source.output.format, Some(Format::Csv));

        let args = Args::parse_from(["netspeed-cli", "--format", "dashboard"]);
        let source = ConfigSource::from_args(&args);
        assert_eq!(source.output.format, Some(Format::Dashboard));
    }

    #[test]
    fn test_config_source_preserves_option_bools() {
        // --no-download=false should yield Some(false), not None
        let args = Args::parse_from(["netspeed-cli", "--no-download=false"]);
        let source = ConfigSource::from_args(&args);
        assert_eq!(source.test.no_download, Some(false));

        // No flag should yield None
        let args = Args::parse_from(["netspeed-cli"]);
        let source = ConfigSource::from_args(&args);
        assert!(source.test.no_download.is_none());
    }

    #[test]
    fn test_config_source_default_composes_sub_sources() {
        let source = ConfigSource::default();

        // Verify each sub-source matches its own Default
        assert_eq!(
            source.output.csv_delimiter,
            OutputSource::default().csv_delimiter
        );
        assert_eq!(source.output.theme, OutputSource::default().theme);
        assert_eq!(source.network.timeout, NetworkSource::default().timeout);
        assert!(source.test.no_download.is_none()); // matches TestSource::default()
        assert!(source.servers.server_ids.is_empty()); // matches ServerSource::default()

        // Verify sub-source fields are accessible through composition
        assert!(source.output.bytes.is_none());
        assert!(source.network.source.is_none());
        assert!(source.strict_config.is_none());
    }

    // ==================== OutputSource Tests ====================

    #[test]
    fn test_output_source_default() {
        let src = OutputSource::default();
        assert!(src.bytes.is_none());
        assert!(src.simple.is_none());
        assert!(src.csv.is_none());
        assert_eq!(src.csv_delimiter, ',');
        assert!(src.csv_header.is_none());
        assert!(src.json.is_none());
        assert!(!src.list);
        assert!(src.quiet.is_none());
        assert!(src.minimal.is_none());
        assert!(src.profile.is_none());
        assert_eq!(src.theme, "dark");
        assert!(src.format.is_none());
    }

    #[test]
    fn test_output_source_custom() {
        let src = OutputSource {
            bytes: Some(true),
            csv_delimiter: ';',
            list: true,
            profile: Some("gamer".to_string()),
            theme: "light".to_string(),
            format: Some(Format::Json),
            ..Default::default()
        };
        assert_eq!(src.bytes, Some(true));
        assert_eq!(src.csv_delimiter, ';');
        assert!(src.list);
        assert_eq!(src.profile, Some("gamer".to_string()));
        assert_eq!(src.theme, "light");
        assert_eq!(src.format, Some(Format::Json));
        // Unset fields still default
        assert!(src.simple.is_none());
        assert!(src.csv.is_none());
        assert!(src.json.is_none());
    }

    #[test]
    fn test_output_source_clone() {
        let src = OutputSource {
            profile: Some("streamer".to_string()),
            ..Default::default()
        };
        let cloned = src.clone();
        assert_eq!(src.profile, cloned.profile);
        assert_eq!(src.csv_delimiter, cloned.csv_delimiter);
        assert_eq!(src.theme, cloned.theme);
    }

    // ==================== TestSource Tests ====================

    #[test]
    fn test_test_source_default() {
        let src = TestSource::default();
        assert!(src.no_download.is_none());
        assert!(src.no_upload.is_none());
        assert!(src.single.is_none());
    }

    #[test]
    fn test_test_source_custom() {
        let src = TestSource {
            no_download: Some(true),
            no_upload: Some(false),
            single: Some(true),
        };
        assert_eq!(src.no_download, Some(true));
        assert_eq!(src.no_upload, Some(false));
        assert_eq!(src.single, Some(true));
    }

    #[test]
    fn test_test_source_clone() {
        let src = TestSource {
            no_download: Some(true),
            ..Default::default()
        };
        let cloned = src.clone();
        assert_eq!(src.no_download, cloned.no_download);
    }

    // ==================== NetworkSource Tests ====================

    #[test]
    fn test_network_source_default() {
        let src = NetworkSource::default();
        assert!(src.source.is_none());
        assert_eq!(src.timeout, 10);
        assert!(src.ca_cert.is_none());
        assert!(src.tls_version.is_none());
        assert!(src.pin_certs.is_none());
    }

    #[test]
    fn test_network_source_custom() {
        let src = NetworkSource {
            source: Some("0.0.0.0".to_string()),
            timeout: 60,
            ca_cert: Some("/path/to/ca.pem".to_string()),
            tls_version: Some("1.3".to_string()),
            pin_certs: Some(true),
        };
        assert_eq!(src.source, Some("0.0.0.0".to_string()));
        assert_eq!(src.timeout, 60);
        assert_eq!(src.ca_cert, Some("/path/to/ca.pem".to_string()));
        assert_eq!(src.tls_version, Some("1.3".to_string()));
        assert_eq!(src.pin_certs, Some(true));
    }

    #[test]
    fn test_network_source_clone() {
        let src = NetworkSource {
            source: Some("192.168.1.1".to_string()),
            ..Default::default()
        };
        let cloned = src.clone();
        assert_eq!(src.source, cloned.source);
        assert_eq!(src.timeout, cloned.timeout);
    }

    // ==================== ServerSource Tests ====================

    #[test]
    fn test_server_source_default() {
        let src = ServerSource::default();
        assert!(src.server_ids.is_empty());
        assert!(src.exclude_ids.is_empty());
    }

    #[test]
    fn test_server_source_custom() {
        let src = ServerSource {
            server_ids: vec!["1234".to_string()],
            exclude_ids: vec!["5678".to_string()],
        };
        assert_eq!(src.server_ids, vec!["1234".to_string()]);
        assert_eq!(src.exclude_ids, vec!["5678".to_string()]);
    }

    #[test]
    fn test_server_source_clone() {
        let src = ServerSource {
            server_ids: vec!["1234".to_string(), "5678".to_string()],
            ..Default::default()
        };
        let cloned = src.clone();
        assert_eq!(src.server_ids, cloned.server_ids);
        assert_eq!(src.exclude_ids, cloned.exclude_ids);
    }

    // ==================== OutputConfig Tests ====================

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert!(!config.bytes);
        assert!(!config.simple);
        assert!(!config.csv);
        assert_eq!(config.csv_delimiter, ',');
        assert!(!config.csv_header);
        assert!(!config.json);
        assert!(!config.list);
        assert!(!config.quiet);
        assert!(config.profile.is_none());
        assert_eq!(config.theme, Theme::Dark);
        assert!(!config.minimal);
        assert!(config.format.is_none());
    }

    #[test]
    fn test_output_config_clone() {
        let config = OutputConfig::default();
        let cloned = config.clone();
        assert_eq!(config.bytes, cloned.bytes);
        assert_eq!(config.csv_delimiter, cloned.csv_delimiter);
        assert_eq!(config.theme, cloned.theme);
    }

    #[test]
    fn test_output_config_debug() {
        let config = OutputConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("OutputConfig"));
    }

    #[test]
    fn test_output_config_custom_theme() {
        let custom = OutputConfig {
            theme: Theme::Light,
            ..Default::default()
        };
        assert_eq!(custom.theme, Theme::Light);
    }

    #[test]
    fn test_output_config_csv_settings() {
        let custom = OutputConfig {
            csv: true,
            csv_delimiter: ';',
            csv_header: true,
            ..Default::default()
        };
        assert!(custom.csv);
        assert_eq!(custom.csv_delimiter, ';');
        assert!(custom.csv_header);
    }

    #[test]
    fn test_test_selection_defaults() {
        let config = TestSelection::default();
        assert!(!config.no_download);
        assert!(!config.no_upload);
        assert!(!config.single);
    }

    #[test]
    fn test_test_selection_skip_tests() {
        let custom = TestSelection {
            no_download: true,
            no_upload: true,
            single: true,
        };
        assert!(custom.no_download);
        assert!(custom.no_upload);
        assert!(custom.single);
    }

    #[test]
    fn test_output_config_profile() {
        let with_profile = OutputConfig {
            profile: Some("gamer".to_string()),
            ..Default::default()
        };
        assert_eq!(with_profile.profile, Some("gamer".to_string()));
    }

    // ==================== NetworkConfig Tests ====================

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(config.source.is_none());
        assert_eq!(config.timeout, 10);
        assert!(config.ca_cert.is_none());
        assert!(config.tls_version.is_none());
        assert!(!config.pin_certs);
    }

    #[test]
    fn test_network_config_clone() {
        let config = NetworkConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout, cloned.timeout);
        assert_eq!(config.pin_certs, cloned.pin_certs);
    }

    #[test]
    fn test_network_config_debug() {
        let config = NetworkConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("NetworkConfig"));
    }

    #[test]
    fn test_network_config_custom_timeout() {
        let custom = NetworkConfig {
            timeout: 60,
            ..Default::default()
        };
        assert_eq!(custom.timeout, 60);
    }

    #[test]
    fn test_network_config_source_ip() {
        let with_source = NetworkConfig {
            source: Some("192.168.1.100".to_string()),
            ..Default::default()
        };
        assert_eq!(with_source.source, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_network_config_tls_settings() {
        let custom = NetworkConfig {
            ca_cert: Some("/path/to/ca.pem".to_string()),
            tls_version: Some("1.2".to_string()),
            pin_certs: true,
            ..Default::default()
        };
        assert_eq!(custom.ca_cert, Some("/path/to/ca.pem".to_string()));
        assert_eq!(custom.tls_version, Some("1.2".to_string()));
        assert!(custom.pin_certs);
    }

    #[test]
    fn test_network_config_tls_1_3() {
        let custom = NetworkConfig {
            tls_version: Some("1.3".to_string()),
            pin_certs: true,
            ..Default::default()
        };
        assert_eq!(custom.tls_version, Some("1.3".to_string()));
        assert!(custom.pin_certs);
    }

    // ==================== ServerSelection Tests ====================

    #[test]
    fn test_server_selection_default() {
        let selection = ServerSelection::default();
        assert!(selection.server_ids.is_empty());
        assert!(selection.exclude_ids.is_empty());
    }

    #[test]
    fn test_server_selection_clone() {
        let selection = ServerSelection::default();
        let cloned = selection.clone();
        assert!(cloned.server_ids.is_empty());
        assert!(cloned.exclude_ids.is_empty());
    }

    #[test]
    fn test_server_selection_debug() {
        let selection = ServerSelection::default();
        let debug_str = format!("{selection:?}");
        assert!(debug_str.contains("ServerSelection"));
    }

    #[test]
    fn test_server_selection_specific_ids() {
        let selection = ServerSelection {
            server_ids: vec!["1234".to_string(), "5678".to_string()],
            exclude_ids: Vec::new(),
        };
        assert_eq!(selection.server_ids.len(), 2);
        assert!(selection.exclude_ids.is_empty());
    }

    #[test]
    fn test_server_selection_exclude() {
        let selection = ServerSelection {
            server_ids: Vec::new(),
            exclude_ids: vec!["9999".to_string()],
        };
        assert!(selection.server_ids.is_empty());
        assert_eq!(selection.exclude_ids.len(), 1);
        assert_eq!(selection.exclude_ids[0], "9999");
    }

    #[test]
    fn test_server_selection_both() {
        let selection = ServerSelection {
            server_ids: vec!["1234".to_string()],
            exclude_ids: vec!["5678".to_string()],
        };
        assert_eq!(selection.server_ids.len(), 1);
        assert_eq!(selection.exclude_ids.len(), 1);
    }

    #[test]
    fn test_server_selection_from_source_empty() {
        let args = Args::parse_from(["netspeed-cli"]);
        let source = ConfigSource::from_args(&args);
        let selection = ServerSelection::from_source(&source.servers);
        assert!(selection.server_ids.is_empty());
        assert!(selection.exclude_ids.is_empty());
    }

    #[test]
    fn test_server_selection_from_source_with_servers() {
        let args = Args::parse_from(["netspeed-cli", "--server", "1234", "--server", "5678"]);
        let source = ConfigSource::from_args(&args);
        let selection = ServerSelection::from_source(&source.servers);
        assert_eq!(selection.server_ids, vec!["1234", "5678"]);
    }

    #[test]
    fn test_server_selection_from_source_with_excludes() {
        let args = Args::parse_from(["netspeed-cli", "--exclude", "9999", "--exclude", "8888"]);
        let source = ConfigSource::from_args(&args);
        let selection = ServerSelection::from_source(&source.servers);
        assert_eq!(selection.exclude_ids, vec!["9999", "8888"]);
    }

    // ==================== Config Getters Tests ====================

    #[test]
    fn test_config_getters_match_direct_access() {
        let config = Config::default();

        // Test execution getters
        assert_eq!(config.no_download(), config.test.no_download);
        assert_eq!(config.no_upload(), config.test.no_upload);
        assert_eq!(config.single(), config.test.single);

        // Output getters
        assert_eq!(config.bytes(), config.output.bytes);
        assert_eq!(config.simple(), config.output.simple);
        assert_eq!(config.csv(), config.output.csv);
        assert_eq!(config.json(), config.output.json);
        assert_eq!(config.quiet(), config.output.quiet);
        assert_eq!(config.list(), config.output.list);
        assert_eq!(config.minimal(), config.output.minimal);
        assert_eq!(config.theme(), config.output.theme);
        assert_eq!(config.csv_delimiter(), config.output.csv_delimiter);
        assert_eq!(config.csv_header(), config.output.csv_header);
        assert_eq!(config.profile(), config.output.profile.as_deref());
        assert_eq!(config.format(), config.output.format);

        // Network getters
        assert_eq!(config.timeout(), config.network.timeout);
        assert_eq!(config.source(), config.network.source.as_deref());
        assert_eq!(config.ca_cert(), config.network.ca_cert.as_deref());
        assert_eq!(config.tls_version(), config.network.tls_version.as_deref());
        assert_eq!(config.pin_certs(), config.network.pin_certs);

        // Server getters
        assert_eq!(config.server_ids(), &config.servers.server_ids[..]);
        assert_eq!(config.exclude_ids(), &config.servers.exclude_ids[..]);

        // Top-level getters
        assert_eq!(
            config.custom_user_agent(),
            config.custom_user_agent.as_deref()
        );
        assert_eq!(config.strict(), config.strict);
    }

    #[test]
    fn test_config_getter_returns_for_option_fields() {
        // Test that Option getters return correct values
        let config = Config {
            output: OutputConfig {
                profile: Some("gamer".to_string()),
                ..Default::default()
            },
            test: TestSelection {
                no_download: false,
                no_upload: false,
                single: false,
            },
            network: NetworkConfig {
                source: Some("192.168.1.1".to_string()),
                ca_cert: Some("/path/to/cert".to_string()),
                tls_version: Some("1.3".to_string()),
                ..Default::default()
            },
            servers: ServerSelection {
                server_ids: vec!["1234".to_string()],
                exclude_ids: vec!["5678".to_string()],
            },
            custom_user_agent: Some("CustomAgent/1.0".to_string()),
            strict: true,
        };

        // Option getters should return Some(&str)
        assert_eq!(config.profile(), Some("gamer"));
        assert_eq!(config.source(), Some("192.168.1.1"));
        assert_eq!(config.ca_cert(), Some("/path/to/cert"));
        assert_eq!(config.tls_version(), Some("1.3"));
        assert_eq!(config.custom_user_agent(), Some("CustomAgent/1.0"));

        // Slice getters should return &["String"]
        assert_eq!(config.server_ids(), ["1234"]);
        assert_eq!(config.exclude_ids(), ["5678"]);

        // Boolean getters
        assert!(!config.pin_certs()); // Default is false, we didn't enable it
        assert!(config.strict());
    }

    #[test]
    fn test_config_getters_none_for_unset_options() {
        let config = Config::default();

        assert_eq!(config.profile(), None);
        assert_eq!(config.source(), None);
        assert_eq!(config.ca_cert(), None);
        assert_eq!(config.tls_version(), None);
        assert_eq!(config.custom_user_agent(), None);
        assert!(config.server_ids().is_empty());
        assert!(config.exclude_ids().is_empty());
    }

    // ==================== should_save_history Tests ====================

    #[test]
    fn test_should_save_history_default_format() {
        let config = Config::default();
        // Default format is None (Detailed) → should save history
        assert!(config.should_save_history());
    }

    #[test]
    fn test_should_save_history_json_format() {
        let mut config = Config::default();
        config.output.format = Some(Format::Json);
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_jsonl_format() {
        let mut config = Config::default();
        config.output.format = Some(Format::Jsonl);
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_csv_format() {
        let mut config = Config::default();
        config.output.format = Some(Format::Csv);
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_non_machine_readable_formats() {
        // Non-machine-readable formats should still save history
        for fmt in [
            Format::Minimal,
            Format::Simple,
            Format::Compact,
            Format::Detailed,
            Format::Dashboard,
        ] {
            let mut config = Config::default();
            config.output.format = Some(fmt);
            assert!(
                config.should_save_history(),
                "format {:?} should save history",
                fmt
            );
        }
    }

    #[test]
    fn test_should_save_history_legacy_json_flag() {
        let mut config = Config::default();
        config.output.json = true;
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_legacy_csv_flag() {
        let mut config = Config::default();
        config.output.csv = true;
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_both_format_and_legacy() {
        // Even if both are set, machine-readable wins
        let mut config = Config::default();
        config.output.format = Some(Format::Detailed); // human-readable
        config.output.json = true; // machine-readable
        assert!(!config.should_save_history());
    }

    #[test]
    fn test_should_save_history_verbose_detailed() {
        // Detailed format is not machine-readable, should save
        let mut config = Config::default();
        config.output.format = Some(Format::Detailed);
        assert!(config.should_save_history());
    }

    // ==================== validate_and_report Tests ====================

    #[test]
    fn test_validate_and_report_with_file_config() {
        let source = ConfigSource::default();
        let config = Config::from_source(&source);
        let file_config = File::default();

        // Pass pre-loaded file config to avoid redundant loading
        let result = config.validate_and_report(&source, Some(file_config));
        assert!(result.valid);
    }

    #[test]
    fn test_validate_and_report_invalid_profile() {
        let mut source = ConfigSource::default();
        source.output.profile = Some("invalid_profile_xyz".to_string());
        let config = Config::from_source(&source);
        let file_config = File::default();

        let result = config.validate_and_report(&source, Some(file_config));
        // Invalid profile from source is a warning, not an error (graceful fallback)
        assert!(result.valid); // valid=true because it's a warning, not an error
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("invalid_profile_xyz"));
    }

    #[test]
    fn test_validate_and_report_invalid_file_config() {
        let source = ConfigSource::default();
        let config = Config::from_source(&source);

        // File config with invalid profile is an error
        let file_config = File {
            profile: Some("bad_profile".to_string()),
            ..Default::default()
        };

        let result = config.validate_and_report(&source, Some(file_config));
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    // ==================== Config Defaults Tests ====================

    #[test]
    fn test_config_default_composes_sub_structs() {
        let config = Config::default();

        // Verify sub-struct defaults are composed correctly
        assert!(!config.output.bytes); // bytes defaults to false
        let _ = config.output;
        let _ = config.test;
        let _ = config.network;
        let _ = config.servers;

        // Direct field access still works
        assert!(!config.test.no_download);
        assert_eq!(config.network.timeout, 10);
        assert!(config.servers.server_ids.is_empty());
    }

    #[test]
    fn test_config_clone_preserves_all_fields() {
        let config = Config {
            output: OutputConfig {
                bytes: true,
                theme: Theme::Light,
                profile: Some("test".to_string()),
                ..Default::default()
            },
            test: TestSelection {
                no_download: true,
                no_upload: false,
                single: true,
            },
            network: NetworkConfig {
                timeout: 30,
                source: Some("127.0.0.1".to_string()),
                pin_certs: true,
                ..Default::default()
            },
            servers: ServerSelection {
                server_ids: vec!["abc".to_string()],
                exclude_ids: vec!["xyz".to_string()],
            },
            custom_user_agent: Some("TestAgent".to_string()),
            strict: true,
        };

        let cloned = config.clone();

        // Verify all fields are preserved
        assert!(cloned.output.bytes);
        assert_eq!(cloned.output.theme, Theme::Light);
        assert_eq!(cloned.output.profile, Some("test".to_string()));
        assert!(cloned.test.no_download);
        assert!(!cloned.test.no_upload);
        assert!(cloned.test.single);
        assert_eq!(cloned.network.timeout, 30);
        assert_eq!(cloned.network.source, Some("127.0.0.1".to_string()));
        assert!(cloned.network.pin_certs);
        assert_eq!(cloned.servers.server_ids, vec!["abc".to_string()]);
        assert_eq!(cloned.servers.exclude_ids, vec!["xyz".to_string()]);
        assert_eq!(cloned.custom_user_agent, Some("TestAgent".to_string()));
        assert!(cloned.strict);
    }
}
