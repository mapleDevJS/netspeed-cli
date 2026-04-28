//! Output format and display configuration types.

use crate::theme::Theme;

use super::{File, OutputSource};

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
pub struct OutputConfig {
    /// Display values in bytes instead of bits
    pub bytes: bool,
    /// Suppress verbose output (deprecated, use format)
    pub simple: bool,
    /// Output in CSV format (deprecated, use format)
    pub csv: bool,
    /// CSV field delimiter
    pub csv_delimiter: char,
    /// Include CSV headers
    pub csv_header: bool,
    /// Output in JSON format (deprecated, use format)
    pub json: bool,
    /// Display server list and exit
    pub list: bool,
    /// Suppress all progress output
    pub quiet: bool,
    /// User profile for customized output
    pub profile: Option<String>,
    /// Color theme for terminal output
    pub theme: Theme,
    /// Minimal ASCII-only output (no Unicode box-drawing)
    pub minimal: bool,
    /// Output format (supersedes legacy --json/--csv/--simple)
    pub format: Option<Format>,
}

// OutputConfig has a manual Default impl to preserve the business-logic default
// for csv_delimiter (',') rather than char's default ('\0').
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
