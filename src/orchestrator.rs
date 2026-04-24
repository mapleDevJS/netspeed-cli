//! Orchestrates the full speed test lifecycle.
//!
//! Extracted from `main.rs` to follow single-responsibility and enable
//! unit testing of the test flow independent of the binary entry point.

use crate::cli::{Args, ShellType};
use crate::common;
use crate::config::Config;
use crate::error::Error;
use crate::formatter::format_list;
use crate::history;
use crate::http;
use crate::progress::{create_spinner, finish_ok, reveal_pause, reveal_scan_complete};
use crate::servers::{fetch, ping_test, select_best_server};
use crate::task_runner::{self, TestRunResult};
use crate::terminal;
use crate::types::Server;
use crate::types::{self, PhaseResult, TestPhases, TestResult};
use crate::{download, upload};

use owo_colors::OwoColorize;

/// Orchestrates the full speed test lifecycle.
pub struct Orchestrator {
    args: Args,
    config: Config,
    client: reqwest::Client,
}

impl Orchestrator {
    /// Create a new orchestrator from CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NetworkError`] if the HTTP client cannot be created
    /// (e.g., invalid source IP, TLS backend failure).
    pub fn new(args: Args) -> Result<Self, Error> {
        let config = Config::from_args(&args);
        let http_settings = http::Settings {
            timeout_secs: config.timeout,
            source_ip: config.source.clone(),
            user_agent: http::Settings::default().user_agent,
            retry_enabled: true,
            tls: http::TlsConfig {
                ca_cert_path: config.ca_cert.as_ref().map(std::path::PathBuf::from),
                min_tls_version: config.tls_version.clone(),
                pin_speedtest_certs: config.pin_certs,
            },
        };
        let client = http::create_client(&http_settings)?;
        Ok(Self {
            args,
            config,
            client,
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

        // Show config path early-exit
        if self.args.show_config_path {
            Self::show_config_path();
            return Ok(());
        }

        // Shell completion early-exit
        if let Some(shell) = self.args.generate_completion {
            Self::generate_shell_completion(shell);
            return Ok(());
        }

        // History display early-exit
        if self.args.history {
            history::show()?;
            return Ok(());
        }

        // Dry-run: validate configuration and exit
        if self.args.dry_run {
            self.run_dry_run();
            return Ok(());
        }

        let is_verbose = self.is_verbose();

        // Print header
        if is_verbose {
            Self::print_header();
        }

        // Start test timer
        let test_start = std::time::Instant::now();

        // Fetch client location and servers
        let (client_location, servers) = self.fetch_client_location_and_servers(is_verbose).await?;

        // Handle --list: format_list already printed, signal completion
        if self.config.list {
            return Ok(());
        }

        // Select best server
        let server = select_best_server(&servers)?;

        // Server info
        if is_verbose {
            Self::print_server_info(&server);
        }

        // Discover client IP (non-fatal — continue even if discovery fails)
        let client_ip = match http::discover_client_ip(&self.client).await {
            Ok(ip) => Some(ip),
            Err(e) => {
                if is_verbose {
                    eprintln!("Warning: Could not discover client IP: {e}");
                }
                None
            }
        };

        // Run ping test
        let (ping, jitter, packet_loss, ping_samples) =
            self.run_ping_test(&server, is_verbose).await?;

        // Run download test
        let dl_result = self.run_download_test(&server, is_verbose).await?;

        // Run upload test
        let ul_result = self.run_upload_test(&server, is_verbose).await?;

        // Build result
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
            client_ip,
            client_location,
        );
        result.phases = TestPhases {
            ping: if self.config.no_download && self.config.no_upload {
                PhaseResult::skipped("both bandwidth phases disabled")
            } else {
                PhaseResult::completed()
            },
            download: if self.config.no_download {
                PhaseResult::skipped("disabled by user")
            } else {
                PhaseResult::completed()
            },
            upload: if self.config.no_upload {
                PhaseResult::skipped("disabled by user")
            } else {
                PhaseResult::completed()
            },
        };

        // Save to history (unless --json or --csv)
        if !self.config.json && !self.config.csv {
            if let Err(e) = history::save_result(&result) {
                eprintln!("Warning: Failed to save test result to history: {e}");
            }
        }

        // Calculate total elapsed time
        let elapsed = test_start.elapsed();

        // Output — Strategy pattern dispatch
        self.output_results(&mut result, &dl_result, &ul_result, elapsed)?;

        Ok(())
    }

    /// Whether verbose output should be shown.
    #[must_use]
    pub fn is_verbose(&self) -> bool {
        use crate::cli::OutputFormatType;
        // Quiet mode suppresses all stderr output
        if self.config.quiet {
            return false;
        }
        let format_non_verbose = matches!(
            self.args.format,
            Some(
                OutputFormatType::Simple
                    | OutputFormatType::Minimal
                    | OutputFormatType::Compact
                    | OutputFormatType::Json
                    | OutputFormatType::Jsonl
                    | OutputFormatType::Csv
                    | OutputFormatType::Dashboard
            )
        );
        !self.config.simple
            && !self.config.json
            && !self.config.csv
            && !self.config.list
            && !format_non_verbose
    }

    /// Check if this is a simple/quiet mode — show minimal one-line progress.
    #[must_use]
    pub fn is_simple_mode(&self) -> bool {
        #[allow(deprecated)]
        let simple = self.args.simple;
        simple.unwrap_or(false)
            || self.args.quiet.unwrap_or(false)
            || matches!(self.args.format, Some(crate::cli::OutputFormatType::Simple))
    }

    fn print_header() {
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

    fn print_server_info(server: &Server) {
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

    /// Fetch client location and server list in a single pass.
    ///
    /// Returns the client location (for including in test results) and the
    /// filtered server list. Makes a single API call to speedtest.net.
    async fn fetch_client_location_and_servers(
        &self,
        is_verbose: bool,
    ) -> Result<(Option<types::ClientLocation>, Vec<Server>), Error> {
        let fetch_spinner = if is_verbose {
            Some(create_spinner("Finding servers..."))
        } else {
            None
        };

        // Fetch servers - this also returns client location to avoid duplicate API call
        let (mut servers, client_location) = fetch(&self.client).await?;

        if let Some(ref pb) = fetch_spinner {
            finish_ok(pb, &format!("Found {} servers", servers.len()));
            eprintln!();
        }

        // Handle --list option
        if self.config.list {
            format_list(&servers)?;
            return Ok((client_location, Vec::new())); // caller checks config.list
        }

        // Filter servers
        if !self.config.server_ids.is_empty() {
            servers.retain(|s| self.config.server_ids.contains(&s.id));
        }
        if !self.config.exclude_ids.is_empty() {
            servers.retain(|s| !self.config.exclude_ids.contains(&s.id));
        }

        if servers.is_empty() {
            return Err(Error::ServerNotFound(
                "No servers match your criteria. Try running without --server/--exclude filters, or use --list to see available servers.".to_string(),
            ));
        }

        Ok((client_location, servers))
    }

    async fn run_ping_test(
        &self,
        server: &Server,
        is_verbose: bool,
    ) -> Result<(Option<f64>, Option<f64>, Option<f64>, Vec<f64>), Error> {
        if self.config.no_download && self.config.no_upload {
            return Ok((None, None, None, Vec::new()));
        }

        let ping_spinner = if is_verbose {
            Some(create_spinner("Testing latency..."))
        } else {
            None
        };
        let ping_result = ping_test(&self.client, server).await?;
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
        Ok((
            Some(ping_result.0),
            Some(ping_result.1),
            Some(ping_result.2),
            ping_result.3,
        ))
    }

    async fn run_download_test(
        &self,
        server: &Server,
        is_verbose: bool,
    ) -> Result<TestRunResult, Error> {
        if self.config.no_download {
            return Ok(TestRunResult::default());
        }

        task_runner::run_bandwidth_test(
            self.client.clone(),
            server,
            "Download",
            is_verbose,
            |progress| async {
                download::run(&self.client, server, self.config.single, progress).await
            },
        )
        .await
    }

    async fn run_upload_test(
        &self,
        server: &Server,
        is_verbose: bool,
    ) -> Result<TestRunResult, Error> {
        if self.config.no_upload {
            return Ok(TestRunResult::default());
        }

        task_runner::run_bandwidth_test(
            self.client.clone(),
            server,
            "Upload",
            is_verbose,
            |progress| async {
                upload::run(&self.client, server, self.config.single, progress).await
            },
        )
        .await
    }

    fn output_results(
        &self,
        result: &mut TestResult,
        dl_result: &TestRunResult,
        ul_result: &TestRunResult,
        elapsed: std::time::Duration,
    ) -> Result<(), Error> {
        // Compute grades once and attach to result for JSON/machine-readable output
        let profile = crate::profiles::UserProfile::from_name(
            self.args.profile.as_deref().unwrap_or("power-user"),
        )
        .unwrap_or(crate::profiles::UserProfile::PowerUser);
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
            &self.args,
            &self.config,
            dl_result,
            ul_result,
            elapsed,
        );

        // For verbose output formats (detailed), show scan complete reveal first
        if self.is_verbose() {
            Self::reveal_results(result, self.config.theme);
        }

        output_format.format(result, self.config.bytes)?;
        Ok(())
    }

    /// Show the scan completion reveal before outputting detailed results.
    /// Creates intentional friction — user sees the grade "computed" from samples.
    fn reveal_results(result: &TestResult, theme: crate::theme::Theme) {
        let nc = terminal::no_color();

        // Count total samples
        let sample_count = result.download_samples.as_ref().map_or(0, Vec::len)
            + result.upload_samples.as_ref().map_or(0, Vec::len)
            + result.ping_samples.as_ref().map_or(0, Vec::len);

        // Compute overall grade for reveal
        let profile = crate::profiles::UserProfile::from_name("power-user")
            .unwrap_or(crate::profiles::UserProfile::PowerUser);
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

    fn generate_shell_completion(shell: ShellType) {
        // Shell completions are generated at build time by build.rs and placed in
        // the completions/ directory. This runtime flag prints a helpful message
        // pointing users to the pre-generated files.
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
    }

    /// Validate configuration and print confirmation without running tests.
    fn run_dry_run(&self) {
        let nc = terminal::no_color();

        if nc {
            eprintln!("Configuration valid:");
            eprintln!("  Timeout: {}s", self.config.timeout);
            eprintln!("  Format: {}", self.format_description());
            if self.config.quiet {
                eprintln!("  Quiet: enabled");
            }
            if let Some(ref source) = self.config.source {
                eprintln!("  Source IP: {source}");
            }
            if self.config.no_download {
                eprintln!("  Download test: disabled");
            }
            if self.config.no_upload {
                eprintln!("  Upload test: disabled");
            }
            if self.config.single {
                eprintln!("  Streams: single");
            }
            // TLS configuration
            if let Some(ref ca_cert) = self.config.ca_cert {
                eprintln!("  CA cert: {ca_cert}");
            }
            if let Some(ref tls_version) = self.config.tls_version {
                eprintln!("  TLS version: {tls_version}");
            }
            if self.config.pin_certs {
                eprintln!("  Cert pinning: enabled");
            }
            eprintln!("\nDry run complete. Run without --dry-run to perform speed test.");
        } else {
            eprintln!("{}", "Configuration valid:".green().bold());
            eprintln!(
                "  {}: {}s",
                "Timeout".dimmed(),
                self.config.timeout.to_string().cyan()
            );
            eprintln!(
                "  {}: {}",
                "Format".dimmed(),
                self.format_description().white()
            );
            if self.config.quiet {
                eprintln!("  {}: {}", "Quiet".dimmed(), "enabled".green());
            }
            if let Some(ref source) = self.config.source {
                eprintln!("  {}: {source}", "Source IP".dimmed());
            }
            if self.config.no_download {
                eprintln!("  {}: {}", "Download test".dimmed(), "disabled".yellow());
            }
            if self.config.no_upload {
                eprintln!("  {}: {}", "Upload test".dimmed(), "disabled".yellow());
            }
            if self.config.single {
                eprintln!("  {}: {}", "Streams".dimmed(), "single".yellow());
            }
            // TLS configuration
            if let Some(ref ca_cert) = self.config.ca_cert {
                eprintln!("  {}: {ca_cert}", "CA cert".dimmed());
            }
            if let Some(ref tls_version) = self.config.tls_version {
                eprintln!("  {}: {tls_version}", "TLS version".dimmed());
            }
            if self.config.pin_certs {
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
        use crate::cli::OutputFormatType;
        match self.args.format {
            Some(OutputFormatType::Json) => "JSON",
            Some(OutputFormatType::Jsonl) => "JSONL",
            Some(OutputFormatType::Csv) => "CSV",
            Some(OutputFormatType::Minimal) => "Minimal",
            Some(OutputFormatType::Simple) => "Simple",
            Some(OutputFormatType::Compact) => "Compact",
            Some(OutputFormatType::Detailed) => "Detailed",
            Some(OutputFormatType::Dashboard) => "Dashboard",
            None => "Detailed (default)",
        }
    }

    /// Show the configuration file path and exit.
    fn show_config_path() {
        match crate::config::get_config_path_internal() {
            Some(path) => eprintln!("Configuration file: {}", path.display()),
            None => eprintln!("No configuration path available (directories crate returned None)."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use clap::Parser;

    #[test]
    fn test_is_verbose_default() {
        let args = Args::parse_from(["netspeed-cli"]);
        let orch = Orchestrator::new(args).unwrap();
        assert!(orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_simple() {
        let args = Args::parse_from(["netspeed-cli", "--simple"]);
        let orch = Orchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_json() {
        let args = Args::parse_from(["netspeed-cli", "--json"]);
        let orch = Orchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_csv() {
        let args = Args::parse_from(["netspeed-cli", "--csv"]);
        let orch = Orchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_list() {
        let args = Args::parse_from(["netspeed-cli", "--list"]);
        let orch = Orchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_orchestrator_creation() {
        let args = Args::parse_from(["netspeed-cli"]);
        let orch = Orchestrator::new(args);
        assert!(orch.is_ok());
    }

    #[test]
    fn test_orchestrator_creation_default() {
        // Default args (no source IP) should always create successfully
        let args = Args::parse_from(["netspeed-cli"]);
        let orch = Orchestrator::new(args);
        assert!(orch.is_ok());
    }

    #[test]
    fn test_dry_run_succeeds() {
        let args = Args::parse_from(["netspeed-cli", "--dry-run"]);
        let orch = Orchestrator::new(args).unwrap();
        // run_dry_run should not panic
        orch.run_dry_run();
    }
}
