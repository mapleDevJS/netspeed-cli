//! Raw CLI input types that bridge the CLI layer to the config layer.
//!
//! These structs carry unmerged, unvalidated values from [`crate::cli::Args`].
//! The only place that touches `Args` directly is `ConfigSource::from_args`;
//! everything downstream depends on these source types, not on the CLI crate.

use super::Format;

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

/// Raw CLI input values for test execution settings.
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
    /// Restrict TLS connections to speedtest.net and ookla.com domains.
    pub pin_certs: Option<bool>,
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

/// Raw CLI input values for server selection settings.
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
/// The sole bridge between [`crate::cli::Args`] and the config layer.
/// All downstream config code depends on these source types, not on `Args`.
///
/// # Example
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

impl ConfigSource {
    /// Extract config-relevant values from parsed CLI arguments.
    ///
    /// This is the **only** method in the config layer that touches
    /// [`crate::cli::Args`]. All downstream code uses [`ConfigSource`].
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
