//! Configuration source types (CLI input → config bridge).
//!
//! These structs represent raw input from CLI arguments and config files
//! before being processed into the final Config type.

use serde::Deserialize;

/// Raw CLI input values for output settings.
#[derive(Debug, Clone, Default)]
pub struct OutputSource {
    pub bytes: Option<bool>,
    pub simple: Option<bool>,
    pub csv: Option<bool>,
    pub csv_delimiter: char,
    pub csv_header: Option<bool>,
    pub json: Option<bool>,
    pub list: bool,
    pub quiet: Option<bool>,
    pub minimal: Option<bool>,
    pub profile: Option<String>,
    pub theme: String,
    pub format: Option<Format>,
}

/// Raw CLI input values for test execution settings.
#[derive(Debug, Clone, Default)]
pub struct TestSource {
    pub no_download: Option<bool>,
    pub no_upload: Option<bool>,
    pub single: Option<bool>,
    pub timeout: Option<u64>,
}

/// Raw CLI input values for network settings.
#[derive(Debug, Clone, Default)]
pub struct NetworkSource {
    pub source: Option<String>,
    pub timeout: u64,
    pub ca_cert: Option<String>,
    pub tls_version: Option<String>,
    pub pin_certs: Option<bool>,
    pub insecure: Option<bool>,
}

/// Raw CLI input values for server selection.
#[derive(Debug, Clone, Default)]
pub struct ServerSource {
    pub server_ids: Vec<u64>,
    pub exclude_ids: Vec<u64>,
    pub server_id: Option<u64>,
}

/// Combined raw input from all sources.
#[derive(Debug, Clone, Default)]
pub struct ConfigSource {
    pub output: OutputSource,
    pub test: TestSource,
    pub network: NetworkSource,
    pub server: ServerSource,
}

/// Output format selection — config-internal domain type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Jsonl,
    Csv,
    Minimal,
    Simple,
    Compact,
    Detailed,
    Dashboard,
}

impl Format {
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

    #[must_use]
    pub fn is_machine_readable(self) -> bool {
        matches!(self, Self::Json | Self::Jsonl | Self::Csv)
    }

    #[must_use]
    pub fn is_non_verbose(self) -> bool {
        !matches!(self, Self::Detailed)
    }
}