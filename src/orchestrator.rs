//! Orchestrates the full speed test lifecycle.
//!
//! Extracted from `main.rs` to follow single-responsibility and enable
//! unit testing of the test flow independent of the binary entry point.

use crate::cli::{CliArgs, ShellType};
use crate::config::Config;
use crate::error::SpeedtestError;
use crate::formatter::formatting::format_distance;
use crate::formatter::{OutputFormat, format_list};
use crate::history;
use crate::http;
use crate::progress::{create_spinner, finish_ok, no_color};
use crate::servers::{create_latency_monitor, fetch_servers, ping_test, select_best_server};
use crate::test_runner::{self, TestRunResult};
use crate::types::{BandwidthMetrics, Server, ServerInfo, TestResult};
use crate::{download, upload};

use owo_colors::OwoColorize;

/// Orchestrates the full speed test lifecycle.
pub struct SpeedTestOrchestrator {
    args: CliArgs,
    config: Config,
    client: reqwest::Client,
}

impl SpeedTestOrchestrator {
    /// Create a new orchestrator from CLI arguments.
    pub fn new(args: CliArgs) -> Result<Self, SpeedtestError> {
        let config = Config::from_args(&args);
        let client = http::create_client(&config)?;
        Ok(Self {
            args,
            config,
            client,
        })
    }

    /// Run the full speed test workflow.
    pub async fn run(&self) -> Result<(), SpeedtestError> {
        // Shell completion early-exit
        if let Some(shell) = self.args.generate_completion {
            Self::generate_shell_completion(shell);
            return Ok(());
        }

        // History display early-exit
        if self.args.history {
            history::print_history()?;
            return Ok(());
        }

        // Dry-run: validate configuration and exit
        if self.args.dry_run {
            return self.run_dry_run();
        }

        let is_verbose = self.is_verbose();

        // Print header
        if is_verbose {
            Self::print_header();
        }

        // Fetch and filter servers
        let servers = self.fetch_and_filter_servers(is_verbose).await?;

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

        // Discover client IP
        let client_ip = http::discover_client_ip(&self.client).await.ok();

        // Run ping test
        let (ping, jitter, packet_loss, ping_samples) =
            self.run_ping_test(&server, is_verbose).await?;

        // Run download test
        let dl_result = self.run_download_test(&server, is_verbose).await?;

        // Run upload test
        let ul_result = self.run_upload_test(&server, is_verbose).await?;

        // Build result
        let dl_metrics = BandwidthMetrics::from(&dl_result);
        let ul_metrics = BandwidthMetrics::from(&ul_result);
        let result = TestResult::from_test_runs(
            ServerInfo::from(&server),
            ping,
            jitter,
            packet_loss,
            ping_samples,
            &dl_metrics,
            &ul_metrics,
            client_ip,
        );

        // Save to history (unless --json or --csv)
        if !self.config.json && !self.config.csv {
            history::save_result(&result).ok();
        }

        // Output — Strategy pattern dispatch
        self.output_results(&result, &dl_result, &ul_result)?;

        Ok(())
    }

    /// Whether verbose output should be shown.
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
                    | OutputFormatType::Json
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

    fn print_header() {
        eprintln!(
            "{}",
            format!("  ═══  NetSpeed CLI v{}  ═══", env!("CARGO_PKG_VERSION"))
                .dimmed()
                .bold()
        );
        eprintln!("{}", "  Bandwidth test · speedtest.net".dimmed());
        eprintln!();
    }

    fn print_server_info(server: &Server) {
        let dist = format_distance(server.distance);
        eprintln!();
        if no_color() {
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

    async fn fetch_and_filter_servers(
        &self,
        is_verbose: bool,
    ) -> Result<Vec<Server>, SpeedtestError> {
        let fetch_spinner = if is_verbose {
            Some(create_spinner("Finding servers..."))
        } else {
            None
        };
        let mut servers = fetch_servers(&self.client).await?;
        if let Some(ref pb) = fetch_spinner {
            finish_ok(pb, &format!("Found {} servers", servers.len()));
            eprintln!();
        }

        // Handle --list option
        if self.config.list {
            format_list(&servers)?;
            return Ok(Vec::new()); // caller checks config.list
        }

        // Filter servers
        if !self.config.server_ids.is_empty() {
            servers.retain(|s| self.config.server_ids.contains(&s.id));
        }
        if !self.config.exclude_ids.is_empty() {
            servers.retain(|s| !self.config.exclude_ids.contains(&s.id));
        }

        if servers.is_empty() {
            return Err(SpeedtestError::ServerNotFound(
                "No servers match your criteria. Try running without --server/--exclude filters, or use --list to see available servers.".to_string(),
            ));
        }

        Ok(servers)
    }

    async fn run_ping_test(
        &self,
        server: &Server,
        is_verbose: bool,
    ) -> Result<(Option<f64>, Option<f64>, Option<f64>, Vec<f64>), SpeedtestError> {
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
            let msg = if no_color() {
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
    ) -> Result<TestRunResult, SpeedtestError> {
        if self.config.no_download {
            return Ok(TestRunResult::default());
        }

        test_runner::run_bandwidth_test(
            &self.client,
            &server.url,
            "Download",
            is_verbose,
            |progress| async {
                let result =
                    download::download_test(&self.client, server, self.config.single, progress)
                        .await?;
                Ok((
                    result.avg_bps,
                    result.peak_bps,
                    result.total_bytes,
                    result.speed_samples,
                ))
            },
            Some(|client: &reqwest::Client, server_url: &str| {
                create_latency_monitor(client, server_url)
            }),
        )
        .await
    }

    async fn run_upload_test(
        &self,
        server: &Server,
        is_verbose: bool,
    ) -> Result<TestRunResult, SpeedtestError> {
        if self.config.no_upload {
            return Ok(TestRunResult::default());
        }

        test_runner::run_bandwidth_test(
            &self.client,
            &server.url,
            "Upload",
            is_verbose,
            |progress| async {
                let result =
                    upload::upload_test(&self.client, server, self.config.single, progress).await?;
                Ok((
                    result.avg_bps,
                    result.peak_bps,
                    result.total_bytes,
                    result.speed_samples,
                ))
            },
            Some(|client: &reqwest::Client, server_url: &str| {
                create_latency_monitor(client, server_url)
            }),
        )
        .await
    }

    fn output_results(
        &self,
        result: &TestResult,
        dl_result: &TestRunResult,
        ul_result: &TestRunResult,
    ) -> Result<(), SpeedtestError> {
        use crate::cli::OutputFormatType;

        // --format flag takes precedence over legacy --json/--csv/--simple booleans
        let output_format = match self.args.format {
            Some(OutputFormatType::Json) => OutputFormat::Json,
            Some(OutputFormatType::Csv) => OutputFormat::Csv {
                delimiter: self.config.csv_delimiter,
                header: self.config.csv_header,
            },
            Some(OutputFormatType::Simple) => OutputFormat::Simple,
            Some(OutputFormatType::Dashboard) => OutputFormat::Dashboard {
                dl: dl_result.clone(),
                ul: ul_result.clone(),
            },
            Some(OutputFormatType::Detailed) => OutputFormat::Detailed {
                dl: dl_result.clone(),
                ul: ul_result.clone(),
            },
            None => {
                // Legacy boolean flag fallback
                if self.config.json {
                    OutputFormat::Json
                } else if self.config.csv {
                    OutputFormat::Csv {
                        delimiter: self.config.csv_delimiter,
                        header: self.config.csv_header,
                    }
                } else if self.config.simple {
                    OutputFormat::Simple
                } else {
                    OutputFormat::Detailed {
                        dl: dl_result.clone(),
                        ul: ul_result.clone(),
                    }
                }
            }
        };
        output_format.format(result, self.config.bytes)?;

        Ok(())
    }

    fn generate_shell_completion(shell: ShellType) {
        use clap::CommandFactory;
        use clap_complete::{Shell as CompleteShell, generate};
        use std::io;

        let shell_type = match shell {
            ShellType::Bash => CompleteShell::Bash,
            ShellType::Zsh => CompleteShell::Zsh,
            ShellType::Fish => CompleteShell::Fish,
            ShellType::PowerShell => CompleteShell::PowerShell,
            ShellType::Elvish => CompleteShell::Elvish,
        };

        let mut cmd = CliArgs::command();
        let bin_name = "netspeed-cli";
        generate(shell_type, &mut cmd, bin_name, &mut io::stdout());
    }

    /// Validate configuration and print confirmation without running tests.
    fn run_dry_run(&self) -> Result<(), SpeedtestError> {
        let nc = no_color();

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
            eprintln!(
                "\n{}",
                "Dry run complete. Run without --dry-run to perform speed test.".bright_black()
            );
        }

        Ok(())
    }

    /// Return a human-readable description of the output format.
    fn format_description(&self) -> &'static str {
        use crate::cli::OutputFormatType;
        match self.args.format {
            Some(OutputFormatType::Json) => "JSON",
            Some(OutputFormatType::Csv) => "CSV",
            Some(OutputFormatType::Simple) => "Simple",
            Some(OutputFormatType::Detailed) => "Detailed",
            Some(OutputFormatType::Dashboard) => "Dashboard",
            None => "Detailed (default)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliArgs;
    use clap::Parser;

    #[test]
    fn test_is_verbose_default() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        assert!(orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_simple() {
        let args = CliArgs::parse_from(["netspeed-cli", "--simple"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_json() {
        let args = CliArgs::parse_from(["netspeed-cli", "--json"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_csv() {
        let args = CliArgs::parse_from(["netspeed-cli", "--csv"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_list() {
        let args = CliArgs::parse_from(["netspeed-cli", "--list"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_orchestrator_creation() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let orch = SpeedTestOrchestrator::new(args);
        assert!(orch.is_ok());
    }

    #[test]
    fn test_orchestrator_creation_default() {
        // Default args (no source IP) should always create successfully
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let orch = SpeedTestOrchestrator::new(args);
        assert!(orch.is_ok());
    }

    #[test]
    fn test_dry_run_succeeds() {
        let args = CliArgs::parse_from(["netspeed-cli", "--dry-run"]);
        let orch = SpeedTestOrchestrator::new(args).unwrap();
        // run_dry_run should not panic and should return Ok
        let result = orch.run_dry_run();
        assert!(result.is_ok());
    }
}
