//! Orchestrates the full speed test lifecycle.
//!
//! Extracted from `main.rs` to follow single-responsibility and enable
//! unit testing of the test flow independent of the binary entry point.

use crate::cli::{Args, ShellType};
use crate::common;
use crate::config::{Config, ConfigSource, Format};
use crate::error::Error;
use crate::formatter::format_list;
use crate::history;
use crate::http;
use crate::profiles::UserProfile;
use crate::progress::{create_spinner, finish_ok, reveal_pause, reveal_scan_complete};
use crate::servers::{fetch, ping_test, select_best_server};
use crate::task_runner::{self, TestRunResult};
use crate::terminal;
use crate::types::Server;
use crate::types::{self, PhaseResult, TestPhases, TestResult};
use crate::{download, upload};

use owo_colors::OwoColorize;

// ============================================================================
// Phase Execution Framework (Command Pattern)
// ============================================================================

/// Outcome of a phase execution.
#[derive(Debug)]
pub enum PhaseOutcome {
    /// Phase completed successfully, continue to next phase
    Completed,
    /// Phase triggered an early exit (e.g., --list, --dry-run)
    EarlyExit,
    /// Phase failed with an error
    Error(Error),
}

/// Context passed between phases — holds all data accumulated during execution.
#[derive(Debug, Default)]
pub struct PhaseContext {
    /// Client location from speedtest.net API
    pub client_location: Option<types::ClientLocation>,
    /// Discovered client IP address
    pub client_ip: Option<String>,
    /// Selected server for testing
    pub server: Option<Server>,
    /// Ping test results
    pub ping_result: Option<(f64, f64, f64, Vec<f64>)>,
    /// Download test results
    pub download_result: Option<TestRunResult>,
    /// Upload test results
    pub upload_result: Option<TestRunResult>,
    /// Whether --list was triggered (server list was printed)
    pub list_printed: bool,
    /// Total elapsed time for the test
    pub elapsed: Option<std::time::Duration>,
}

/// Phase variants — enum instead of trait object for async compatibility.
///
/// Using an enum instead of `Box<dyn Phase>` because async methods make
/// trait objects incompatible with `dyn` in stable Rust.
enum PhaseKind {
    EarlyExit,
    Header,
    ServerDiscovery,
    IpDiscovery,
    Ping,
    Bandwidth,
    Result {
        is_verbose: bool,
        test_start: std::time::Instant,
    },
}

impl PhaseKind {
    #[allow(dead_code)]
    fn name(&self) -> &'static str {
        match self {
            PhaseKind::EarlyExit => "EarlyExit",
            PhaseKind::Header => "Header",
            PhaseKind::ServerDiscovery => "ServerDiscovery",
            PhaseKind::IpDiscovery => "IpDiscovery",
            PhaseKind::Ping => "Ping",
            PhaseKind::Bandwidth => "Bandwidth",
            PhaseKind::Result { .. } => "Result",
        }
    }

    /// Execute this phase.
    async fn execute(&self, orch: &Orchestrator, ctx: &mut PhaseContext) -> PhaseOutcome {
        match self {
            PhaseKind::EarlyExit => Self::execute_early_exit(orch, ctx).await,
            PhaseKind::Header => Self::execute_header(orch, ctx).await,
            PhaseKind::ServerDiscovery => Self::execute_server_discovery(orch, ctx).await,
            PhaseKind::IpDiscovery => Self::execute_ip_discovery(orch, ctx).await,
            PhaseKind::Ping => Self::execute_ping(orch, ctx).await,
            PhaseKind::Bandwidth => Self::execute_bandwidth(orch, ctx).await,
            PhaseKind::Result {
                is_verbose,
                test_start,
            } => Self::execute_result(orch, ctx, *is_verbose, *test_start).await,
        }
    }

    // Phase implementation methods

    async fn execute_early_exit(orch: &Orchestrator, _ctx: &mut PhaseContext) -> PhaseOutcome {
        // Show config path early-exit
        if orch.early_exit.show_config_path {
            match crate::config::get_config_path_internal() {
                Some(path) => eprintln!("Configuration file: {}", path.display()),
                None => {
                    eprintln!("No configuration path available (directories crate returned None).")
                }
            }
            return PhaseOutcome::EarlyExit;
        }

        // Shell completion early-exit
        if let Some(shell) = orch.early_exit.generate_completion {
            let shell_name = match shell {
                ShellType::Bash => "netspeed-cli.bash",
                ShellType::Zsh => "_netspeed-cli",
                ShellType::Fish => "netspeed-cli.fish",
                ShellType::PowerShell => "_netspeed-cli.ps1",
                ShellType::Elvish => "netspeed-cli.elv",
            };
            eprintln!("Shell completion for {shell:?} is available in the completions/ directory.");
            eprintln!("  File: {shell_name}");
            eprintln!("  Install: copy it to your shell's completion directory and reload.");
            return PhaseOutcome::EarlyExit;
        }

        // History display early-exit
        if orch.early_exit.history {
            match history::show() {
                Ok(()) => return PhaseOutcome::EarlyExit,
                Err(e) => return PhaseOutcome::Error(e),
            }
        }

        // Dry-run: validate configuration and exit
        if orch.early_exit.dry_run {
            orch.run_dry_run();
            return PhaseOutcome::EarlyExit;
        }

        PhaseOutcome::Completed
    }

    async fn execute_header(orch: &Orchestrator, _ctx: &mut PhaseContext) -> PhaseOutcome {
        if orch.is_verbose() {
            let version = env!("CARGO_PKG_VERSION");
            let nc = terminal::no_color();

            if nc {
                eprintln!();
                eprintln!("  NetSpeed CLI v{version}  ·  speedtest.net");
                eprintln!();
            } else {
                eprintln!();
                eprintln!(
                    "  {} v{}  {}  {}",
                    "NetSpeed CLI".cyan().bold(),
                    version.white(),
                    "·".dimmed(),
                    "speedtest.net".bright_black()
                );
                eprintln!();
            }
        }
        PhaseOutcome::Completed
    }

    async fn execute_server_discovery(orch: &Orchestrator, ctx: &mut PhaseContext) -> PhaseOutcome {
        let is_verbose = orch.is_verbose();

        // Fetch servers - this also returns client location to avoid duplicate API call
        let fetch_spinner = if is_verbose {
            Some(create_spinner("Finding servers..."))
        } else {
            None
        };

        let (mut servers, client_location) = match fetch(&orch.client).await {
            Ok((servers, location)) => (servers, location),
            Err(e) => return PhaseOutcome::Error(e),
        };

        ctx.client_location = client_location;

        if let Some(ref pb) = fetch_spinner {
            finish_ok(pb, &format!("Found {} servers", servers.len()));
            eprintln!();
        }

        // Handle --list option
        if orch.config.list() {
            if let Err(e) = format_list(&servers) {
                return PhaseOutcome::Error(e.into());
            }
            ctx.list_printed = true;
            return PhaseOutcome::EarlyExit;
        }

        // Filter servers
        if !orch.config.server_ids().is_empty() {
            servers.retain(|s| orch.config.server_ids().contains(&s.id));
        }
        if !orch.config.exclude_ids().is_empty() {
            servers.retain(|s| !orch.config.exclude_ids().contains(&s.id));
        }

        if servers.is_empty() {
            return PhaseOutcome::Error(crate::error::Error::ServerNotFound(
                "No servers match your criteria. Try running without --server/--exclude filters, or use --list to see available servers.".to_string(),
            ));
        }

        // Select best server
        let server = match select_best_server(&servers) {
            Ok(s) => s,
            Err(e) => return PhaseOutcome::Error(e),
        };

        // Print server info if verbose
        if is_verbose {
            let dist = common::format_distance(server.distance);
            eprintln!();
            if terminal::no_color() {
                eprintln!("  Server:   {} ({})", server.sponsor, server.name);
                eprintln!("  Location: {} ({dist})", server.country);
            } else {
                eprintln!(
                    "  {}   {} ({})",
                    "Server:".dimmed(),
                    server.sponsor.white().bold(),
                    server.name
                );
                eprintln!("  {} {} ({dist})", "Location:".dimmed(), server.country);
            }
            eprintln!();
        }

        ctx.server = Some(server);
        PhaseOutcome::Completed
    }

    async fn execute_ip_discovery(orch: &Orchestrator, ctx: &mut PhaseContext) -> PhaseOutcome {
        let is_verbose = orch.is_verbose();

        match http::discover_client_ip(&orch.client).await {
            Ok(ip) => {
                ctx.client_ip = Some(ip);
            }
            Err(e) => {
                if is_verbose {
                    eprintln!("Warning: Could not discover client IP: {e}");
                }
                // Non-fatal: continue even if IP discovery fails
            }
        }

        PhaseOutcome::Completed
    }

    async fn execute_ping(orch: &Orchestrator, ctx: &mut PhaseContext) -> PhaseOutcome {
        // Skip if both bandwidth tests are disabled
        if orch.config.no_download() && orch.config.no_upload() {
            return PhaseOutcome::Completed;
        }

        let server = match &ctx.server {
            Some(s) => s,
            None => return PhaseOutcome::Completed,
        };

        let is_verbose = orch.is_verbose();

        let ping_spinner = if is_verbose {
            Some(create_spinner("Testing latency..."))
        } else {
            None
        };

        let ping_result = match ping_test(&orch.client, server).await {
            Ok(result) => result,
            Err(e) => return PhaseOutcome::Error(e),
        };

        if let Some(ref pb) = ping_spinner {
            let msg = if terminal::no_color() {
                format!("Latency: {:.2} ms", ping_result.0)
            } else {
                format!(
                    "Latency: {}",
                    format!("{:.2} ms", ping_result.0).cyan().bold()
                )
            };
            finish_ok(pb, &msg);
        }

        ctx.ping_result = Some((ping_result.0, ping_result.1, ping_result.2, ping_result.3));
        PhaseOutcome::Completed
    }

    async fn execute_bandwidth(orch: &Orchestrator, ctx: &mut PhaseContext) -> PhaseOutcome {
        let server = match &ctx.server {
            Some(s) => s,
            None => return PhaseOutcome::Completed,
        };

        let is_verbose = orch.is_verbose();

        // Download test
        if orch.config.no_download() {
            ctx.download_result = Some(TestRunResult::default());
        } else {
            match task_runner::run_bandwidth_test(
                orch.client.clone(),
                server,
                "Download",
                is_verbose,
                |progress| async {
                    download::run(&orch.client, server, orch.config.single(), progress).await
                },
            )
            .await
            {
                Ok(result) => ctx.download_result = Some(result),
                Err(e) => return PhaseOutcome::Error(e),
            }
        }

        // Upload test
        if orch.config.no_upload() {
            ctx.upload_result = Some(TestRunResult::default());
        } else {
            match task_runner::run_bandwidth_test(
                orch.client.clone(),
                server,
                "Upload",
                is_verbose,
                |progress| async {
                    upload::run(&orch.client, server, orch.config.single(), progress).await
                },
            )
            .await
            {
                Ok(result) => ctx.upload_result = Some(result),
                Err(e) => return PhaseOutcome::Error(e),
            }
        }

        PhaseOutcome::Completed
    }

    async fn execute_result(
        orch: &Orchestrator,
        ctx: &mut PhaseContext,
        _is_verbose: bool,
        test_start: std::time::Instant,
    ) -> PhaseOutcome {
        let server = match &ctx.server {
            Some(s) => s,
            None => return PhaseOutcome::Completed,
        };

        let ping_result = ctx.ping_result.take();
        let download_result = ctx.download_result.take();
        let upload_result = ctx.upload_result.take();

        let (ping, jitter, packet_loss, ping_samples) = match ping_result {
            Some((p, j, pl, s)) => (Some(p), Some(j), Some(pl), s),
            None => (None, None, None, Vec::new()),
        };

        let dl_result = download_result.unwrap_or_default();
        let ul_result = upload_result.unwrap_or_default();

        let mut result = TestResult::from_test_runs(
            types::ServerInfo {
                id: server.id.clone(),
                name: server.name.clone(),
                sponsor: server.sponsor.clone(),
                country: server.country.clone(),
                distance: server.distance,
            },
            ping,
            jitter,
            packet_loss,
            &ping_samples,
            &dl_result,
            &ul_result,
            ctx.client_ip.clone(),
            ctx.client_location.clone(),
        );

        result.phases = TestPhases {
            ping: if orch.config.no_download() && orch.config.no_upload() {
                PhaseResult::skipped("both bandwidth phases disabled")
            } else {
                PhaseResult::completed()
            },
            download: if orch.config.no_download() {
                PhaseResult::skipped("disabled by user")
            } else {
                PhaseResult::completed()
            },
            upload: if orch.config.no_upload() {
                PhaseResult::skipped("disabled by user")
            } else {
                PhaseResult::completed()
            },
        };

        // Save to history (unless machine-readable format — would corrupt stdout)
        if orch.config.should_save_history() {
            if let Err(e) = history::save_result(&result) {
                eprintln!("Warning: Failed to save test result to history: {e}");
            }
        }

        // Calculate elapsed time
        let elapsed = test_start.elapsed();
        ctx.elapsed = Some(elapsed);

        // Output results
        if let Err(e) = orch.output_results(&mut result, &dl_result, &ul_result, elapsed) {
            return PhaseOutcome::Error(e);
        }

        PhaseOutcome::Completed
    }
}

/// Orchestrates the full speed test lifecycle.
pub struct Orchestrator {
    config: Config,
    client: reqwest::Client,
    /// Early-exit flags that don't belong in Config (structural CLI controls)
    early_exit: EarlyExitFlags,
}

/// Early-exit flags extracted from Args — these control flow, not configuration.
pub(crate) struct EarlyExitFlags {
    pub(crate) show_config_path: bool,
    pub(crate) generate_completion: Option<ShellType>,
    pub(crate) history: bool,
    pub(crate) dry_run: bool,
}

impl Orchestrator {
    /// Create a new orchestrator from CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NetworkError`] if the HTTP client cannot be created
    /// (e.g., invalid source IP, TLS backend failure).
    pub fn new(args: Args, file_config: Option<crate::config::File>) -> Result<Self, Error> {
        // Create source once and reuse to avoid redundant ConfigSource construction
        let source = ConfigSource::from_args(&args);

        // Use from_args_with_file to avoid double-loading config file
        let (config, profile_validation) =
            Config::from_args_with_file(&source, file_config.clone());

        // Emit profile validation warnings (from source/CLI)
        for warning in &profile_validation.warnings {
            eprintln!("Warning: {warning}");
        }

        // File config validation (uses pre-loaded file_config to avoid double-loading)
        let file_validation = config.validate_and_report(&source, file_config);
        for error in &file_validation.errors {
            eprintln!("Error: {error}");
        }

        // Combine both validation results for strict mode check
        let combined_valid = profile_validation.valid && file_validation.valid;
        if config.strict() && !combined_valid {
            return Err(crate::error::Error::Context {
                msg: "Configuration validation failed".to_string(),
                source: None,
            });
        }

        let early_exit = EarlyExitFlags {
            show_config_path: args.show_config_path,
            generate_completion: args.generate_completion,
            history: args.history,
            dry_run: args.dry_run,
        };
        Self::from_config(config, early_exit)
    }

    /// Create an orchestrator from pre-built config and early-exit flags.
    ///
    /// This constructor is the testability payoff of the [`ConfigSource`] refactoring:
    /// tests can construct a [`crate::config::Config`] from a hand-built [`ConfigSource`] instead
    /// of going through CLI parsing, then create an [`Orchestrator`] directly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NetworkError`] if the HTTP client cannot be created.
    pub(crate) fn from_config(config: Config, early_exit: EarlyExitFlags) -> Result<Self, Error> {
        let http_settings = http::Settings::from(&config);
        let client = http::create_client(&http_settings)?;
        Ok(Self {
            config,
            client,
            early_exit,
        })
    }

    /// Run the full speed test workflow.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] for network failures, server selection errors,
    /// or output formatting failures.
    pub async fn run(&self) -> Result<(), Error> {
        // NOTE: NO_EMOJI env var is now set in main.rs before the async
        // runtime starts, eliminating the need for unsafe set_var here.

        let mut ctx = PhaseContext::default();
        let is_verbose = self.is_verbose();

        // Execute phases in order
        let phases = [
            PhaseKind::EarlyExit,
            PhaseKind::Header,
            PhaseKind::ServerDiscovery,
            PhaseKind::IpDiscovery,
            PhaseKind::Ping,
            PhaseKind::Bandwidth,
            PhaseKind::Result {
                is_verbose,
                test_start: std::time::Instant::now(),
            },
        ];

        for phase in phases {
            match phase.execute(self, &mut ctx).await {
                PhaseOutcome::Completed => {}
                PhaseOutcome::EarlyExit => return Ok(()),
                PhaseOutcome::Error(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Whether verbose output should be shown.
    #[must_use]
    pub fn is_verbose(&self) -> bool {
        // Quiet mode suppresses all stderr output
        if self.config.quiet() {
            return false;
        }
        let format_non_verbose = self.config.format().is_some_and(|f| f.is_non_verbose());
        !self.config.simple()
            && !self.config.json()
            && !self.config.csv()
            && !self.config.list()
            && !format_non_verbose
    }

    /// Check if this is a simple/quiet mode — show minimal one-line progress.
    #[must_use]
    pub fn is_simple_mode(&self) -> bool {
        self.config.simple() || self.config.quiet() || self.config.format() == Some(Format::Simple)
    }

    #[allow(dead_code)]
    fn output_results(
        &self,
        result: &mut TestResult,
        dl_result: &TestRunResult,
        ul_result: &TestRunResult,
        elapsed: std::time::Duration,
    ) -> Result<(), Error> {
        // Compute grades once and attach to result for JSON/machine-readable output
        let profile = UserProfile::from_name(self.config.profile().unwrap_or("power-user"))
            .unwrap_or(UserProfile::PowerUser);
        let overall = crate::grades::grade_overall(
            result.ping,
            result.jitter,
            result.download,
            result.upload,
            profile,
        );
        result.overall_grade = Some(overall.as_str().to_string());
        result.download_grade = result.download.map(|d| {
            crate::grades::grade_download(d / 1_000_000.0, profile)
                .as_str()
                .to_string()
        });
        result.upload_grade = result.upload.map(|u| {
            crate::grades::grade_upload(u / 1_000_000.0, profile)
                .as_str()
                .to_string()
        });
        result.connection_rating =
            Some(crate::formatter::ratings::connection_rating(result).to_string());

        // Strategy pattern: resolve format → dispatch via OutputFormat::format (OCP)
        let output_format = crate::output_strategy::resolve_output_format(
            &self.config,
            dl_result,
            ul_result,
            elapsed,
        );

        // For verbose output formats (detailed), show scan complete reveal first
        if self.is_verbose() {
            Self::reveal_results(result, self.config.theme(), profile);
        }

        output_format.format(result, self.config.bytes())?;
        Ok(())
    }

    /// Show the scan completion reveal before outputting detailed results.
    /// Creates intentional friction — user sees the grade "computed" from samples.
    fn reveal_results(result: &TestResult, theme: crate::theme::Theme, profile: UserProfile) {
        let nc = terminal::no_color();

        // Count total samples
        let sample_count = result.download_samples.as_ref().map_or(0, Vec::len)
            + result.upload_samples.as_ref().map_or(0, Vec::len)
            + result.ping_samples.as_ref().map_or(0, Vec::len);

        // Compute overall grade for reveal — uses the user's chosen profile
        let overall_grade = crate::grades::grade_overall(
            result.ping,
            result.jitter,
            result.download,
            result.upload,
            profile,
        );

        // Reveal scan completion with grade
        let grade_badge = overall_grade.color_str(nc, theme);
        let grade_plain = overall_grade.as_str().to_string();
        reveal_scan_complete(sample_count, &grade_badge, &grade_plain, nc);

        // Brief pause before sections start
        reveal_pause();
    }

    /// Validate configuration and print confirmation without running tests.
    fn run_dry_run(&self) {
        let nc = terminal::no_color();

        if nc {
            eprintln!("Configuration valid:");
            eprintln!("  Timeout: {}s", self.config.timeout());
            eprintln!("  Format: {}", self.format_description());
            if self.config.quiet() {
                eprintln!("  Quiet: enabled");
            }
            if let Some(source) = self.config.source() {
                eprintln!("  Source IP: {source}");
            }
            if self.config.no_download() {
                eprintln!("  Download test: disabled");
            }
            if self.config.no_upload() {
                eprintln!("  Upload test: disabled");
            }
            if self.config.single() {
                eprintln!("  Streams: single");
            }
            if let Some(ca_cert) = self.config.ca_cert() {
                eprintln!("  CA cert: {ca_cert}");
            }
            if let Some(tls_version) = self.config.tls_version() {
                eprintln!("  TLS version: {tls_version}");
            }
            if self.config.pin_certs() {
                eprintln!("  Cert pinning: enabled");
            }
            eprintln!("\nDry run complete. Run without --dry-run to perform speed test.");
        } else {
            eprintln!("{}", "Configuration valid:".green().bold());
            eprintln!(
                "  {}: {}s",
                "Timeout".dimmed(),
                self.config.timeout().to_string().cyan()
            );
            eprintln!(
                "  {}: {}",
                "Format".dimmed(),
                self.format_description().white()
            );
            if self.config.quiet() {
                eprintln!("  {}: {}", "Quiet".dimmed(), "enabled".green());
            }
            if let Some(source) = self.config.source() {
                eprintln!("  {}: {source}", "Source IP".dimmed());
            }
            if self.config.no_download() {
                eprintln!("  {}: {}", "Download test".dimmed(), "disabled".yellow());
            }
            if self.config.no_upload() {
                eprintln!("  {}: {}", "Upload test".dimmed(), "disabled".yellow());
            }
            if self.config.single() {
                eprintln!("  {}: {}", "Streams".dimmed(), "single".yellow());
            }
            if let Some(ca_cert) = self.config.ca_cert() {
                eprintln!("  {}: {ca_cert}", "CA cert".dimmed());
            }
            if let Some(tls_version) = self.config.tls_version() {
                eprintln!("  {}: {tls_version}", "TLS version".dimmed());
            }
            if self.config.pin_certs() {
                eprintln!("  {}: {}", "Cert pinning".dimmed(), "enabled".yellow());
            }
            eprintln!(
                "\n{}",
                "Dry run complete. Run without --dry-run to perform speed test.".bright_black()
            );
        }
    }

    /// Return a human-readable description of the output format.
    fn format_description(&self) -> &'static str {
        match self.config.format() {
            Some(f) => f.label(),
            None => "Detailed (default)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigSource, OutputSource};

    /// Helper: build an Orchestrator from a ConfigSource + early-exit flags.
    fn orch_from_source(source: &ConfigSource, early_exit: EarlyExitFlags) -> Orchestrator {
        let config = Config::from_source(source);
        Orchestrator::from_config(config, early_exit).unwrap()
    }

    /// Default early-exit flags (all off).
    fn default_early_exit() -> EarlyExitFlags {
        EarlyExitFlags {
            show_config_path: false,
            generate_completion: None,
            history: false,
            dry_run: false,
        }
    }

    #[test]
    fn test_is_verbose_default() {
        let source = ConfigSource::default();
        let orch = orch_from_source(&source, default_early_exit());
        assert!(orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_simple() {
        let source = ConfigSource {
            output: OutputSource {
                simple: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_json() {
        let source = ConfigSource {
            output: OutputSource {
                json: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_csv() {
        let source = ConfigSource {
            output: OutputSource {
                csv: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_list() {
        let source = ConfigSource {
            output: OutputSource {
                list: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_orchestrator_creation() {
        let source = ConfigSource::default();
        let config = Config::from_source(&source);
        let orch = Orchestrator::from_config(config, default_early_exit());
        assert!(orch.is_ok());
    }

    #[test]
    fn test_orchestrator_creation_default() {
        // Default source (no source IP) should always create successfully
        let source = ConfigSource::default();
        let config = Config::from_source(&source);
        let orch = Orchestrator::from_config(config, default_early_exit());
        assert!(orch.is_ok());
    }

    #[test]
    fn test_dry_run_succeeds() {
        let source = ConfigSource::default();
        let early_exit = EarlyExitFlags {
            dry_run: true,
            ..default_early_exit()
        };
        let orch = orch_from_source(&source, early_exit);
        // run_dry_run should not panic
        orch.run_dry_run();
    }
}
