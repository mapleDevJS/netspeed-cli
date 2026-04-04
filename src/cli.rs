use clap::{Parser, ValueEnum};

/// Test internet bandwidth using speedtest.net servers.
///
/// Supports both standard speedtest.net servers and Speedtest Mini installations.
/// Results can be displayed in simple text, JSON, or CSV format.
#[derive(Parser, Debug, Default)]
#[command(name = "netspeed-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Command line interface for testing internet bandwidth using speedtest.net")]
pub struct CliArgs {
    /// Do not perform download test
    #[arg(long)]
    pub no_download: bool,

    /// Do not perform upload test
    #[arg(long)]
    pub no_upload: bool,

    /// Only use a single connection instead of multiple
    #[arg(long)]
    pub single: bool,

    /// Display values in bytes instead of bits
    #[arg(long)]
    pub bytes: bool,

    /// Generate and provide a URL to the speedtest.net share results image
    #[arg(long)]
    pub share: bool,

    /// Suppress verbose output, only show basic information
    #[arg(long)]
    pub simple: bool,

    /// Enable verbose/debug logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Output in CSV format
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,

    /// Single character delimiter for CSV output (default: ",")
    #[arg(long, default_value = ",", requires = "csv")]
    pub csv_delimiter: char,

    /// Print CSV headers
    #[arg(long, requires = "csv")]
    pub csv_header: bool,

    /// Output in JSON format
    #[arg(long, conflicts_with = "csv")]
    pub json: bool,

    /// Display a list of speedtest.net servers sorted by distance
    #[arg(long)]
    pub list: bool,

    /// Specify a server ID to test against (can be supplied multiple times)
    #[arg(long)]
    pub server: Vec<String>,

    /// Exclude a server from selection (can be supplied multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// URL of the Speedtest Mini server
    #[arg(long)]
    pub mini: Option<String>,

    /// Source IP address to bind to
    #[arg(long)]
    pub source: Option<String>,

    /// HTTP timeout in seconds (default: 10)
    #[arg(long, default_value = "10")]
    pub timeout: u64,

    /// Use HTTPS instead of HTTP
    #[arg(long)]
    pub secure: bool,

    /// Do not pre-allocate upload data
    #[arg(long)]
    pub no_pre_allocate: bool,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    pub generate_completion: Option<ShellType>,
}

/// Supported shell types for completion generation.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}
