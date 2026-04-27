//! Phase definitions for the speed test lifecycle.
//!
//! ## Design
//!
//! - [`PhaseContext`] — shared state with private fields (ISP: clients use accessors)
//! - [`PhaseOutcome`] — result of phase execution  
//! - Each phase is an async function that takes (orch, ctx)
//! - [`PhaseExecutor`] — runs phases in sequence

use crate::error::Error;
use crate::services::Services;
use futures::future::BoxFuture;
use std::sync::Arc;

use crate::orchestrator::Orchestrator;
use crate::task_runner::TestRunResult;
use crate::types::Server;

/// Context passed between phases — holds all data accumulated during execution.
pub struct PhaseContext {
    client_location: Option<crate::types::ClientLocation>,
    client_ip: Option<String>,
    server: Option<Server>,
    ping_result: Option<(f64, f64, f64, Vec<f64>)>,
    download_result: Option<TestRunResult>,
    upload_result: Option<TestRunResult>,
    list_printed: bool,
    elapsed: Option<std::time::Duration>,
    services: std::sync::Arc<dyn Services>,
}

impl PhaseContext {
    /// Create a new context with the given services.
    pub fn new(services: std::sync::Arc<dyn Services>) -> Self {
        Self {
            client_location: None,
            client_ip: None,
            server: None,
            ping_result: None,
            download_result: None,
            upload_result: None,
            list_printed: false,
            elapsed: None,
            services,
        }
    }

    // === New setter/taker methods for encapsulation ===

    /// Take the server (removes from context).
    pub fn take_server(&mut self) -> Option<Server> {
        self.server.take()
    }

    /// Set the server.
    pub fn set_server(&mut self, server: Server) {
        self.server = Some(server);
    }

    /// Set client IP.
    pub fn set_client_ip(&mut self, ip: String) {
        self.client_ip = Some(ip);
    }

    /// Set client location.
    pub fn set_client_location(&mut self, location: Option<crate::types::ClientLocation>) {
        self.client_location = location;
    }

    /// Set ping result.
    pub fn set_ping_result(&mut self, result: (f64, f64, f64, Vec<f64>)) {
        self.ping_result = Some(result);
    }

    /// Take ping result.
    pub fn take_ping_result(&mut self) -> Option<(f64, f64, f64, Vec<f64>)> {
        self.ping_result.take()
    }

    /// Set download result.
    pub fn set_download_result(&mut self, result: TestRunResult) {
        self.download_result = Some(result);
    }

    /// Take download result.
    pub fn take_download_result(&mut self) -> Option<TestRunResult> {
        self.download_result.take()
    }

    /// Set upload result.
    pub fn set_upload_result(&mut self, result: TestRunResult) {
        self.upload_result = Some(result);
    }

    /// Take upload result.
    pub fn take_upload_result(&mut self) -> Option<TestRunResult> {
        self.upload_result.take()
    }

    /// Mark list as printed.
    pub fn set_list_printed(&mut self) {
        self.list_printed = true;
    }
}

impl std::fmt::Debug for PhaseContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhaseContext")
            .field("client_location", &self.client_location)
            .field("client_ip", &self.client_ip)
            .field("server", &self.server)
            .field("ping_result", &self.ping_result)
            .field("download_result", &self.download_result)
            .field("upload_result", &self.upload_result)
            .field("list_printed", &self.list_printed)
            .field("elapsed", &self.elapsed)
            .field("services", &"dyn Services")
            .finish()
    }
}

/// Phase outcome.
#[derive(Debug)]
pub enum PhaseOutcome {
    PhaseCompleted,
    PhaseEarlyExit,
    PhaseError(Error),
}

/// Async phase function signature.
pub type PhaseFn =
    for<'a> fn(&'a Orchestrator, &'a mut PhaseContext) -> BoxFuture<'a, PhaseOutcome>;

impl Default for PhaseExecutor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PhaseExecutor {
    phases: Vec<PhaseFn>,
}

impl PhaseExecutor {
    pub fn new() -> Self {
        Self { phases: Vec::new() }
    }

    pub fn register(mut self, phase: PhaseFn) -> Self {
        self.phases.push(phase);
        self
    }

    pub async fn execute_all(&self, orch: &Orchestrator) -> Result<(), Error> {
        let mut ctx = PhaseContext::new(orch.services_arc());
        for phase in &self.phases {
            let outcome = phase(orch, &mut ctx).await;
            match outcome {
                PhaseOutcome::PhaseCompleted => {}
                PhaseOutcome::PhaseEarlyExit => return Ok(()),
                PhaseOutcome::PhaseError(e) => return Err(e),
            }
        }
        Ok(())
    }
}

pub type PhaseResults = (
    Option<(f64, f64, f64, Vec<f64>)>,
    Option<TestRunResult>,
    Option<TestRunResult>,
);

/// PhaseContext accessor methods.
impl PhaseContext {
    pub fn client_location(&self) -> Option<&crate::types::ClientLocation> {
        self.client_location.as_ref()
    }

    pub fn client_ip(&self) -> Option<&str> {
        self.client_ip.as_deref()
    }

    pub fn server(&self) -> Option<&Server> {
        self.server.as_ref()
    }

    pub fn ping_result(&self) -> Option<&(f64, f64, f64, Vec<f64>)> {
        self.ping_result.as_ref()
    }

    pub fn download_result(&self) -> Option<&TestRunResult> {
        self.download_result.as_ref()
    }

    pub fn upload_result(&self) -> Option<&TestRunResult> {
        self.upload_result.as_ref()
    }

    pub fn is_list_printed(&self) -> bool {
        self.list_printed
    }

    pub fn elapsed(&self) -> Option<std::time::Duration> {
        self.elapsed
    }

    pub fn services(&self) -> &dyn Services {
        self.services.as_ref()
    }

    pub fn services_arc(&self) -> std::sync::Arc<dyn Services> {
        self.services.clone()
    }

    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    pub fn with_client_location(mut self, location: crate::types::ClientLocation) -> Self {
        self.client_location = Some(location);
        self
    }

    pub fn with_server(mut self, server: Server) -> Self {
        self.server = Some(server);
        self
    }

    pub fn with_ping_result(mut self, ping: (f64, f64, f64, Vec<f64>)) -> Self {
        self.ping_result = Some(ping);
        self
    }

    pub fn with_download_result(mut self, result: TestRunResult) -> Self {
        self.download_result = Some(result);
        self
    }

    pub fn with_upload_result(mut self, result: TestRunResult) -> Self {
        self.upload_result = Some(result);
        self
    }

    pub fn mark_list_printed(&mut self) {
        self.list_printed = true;
    }

    pub fn set_elapsed(&mut self, elapsed: std::time::Duration) {
        self.elapsed = Some(elapsed);
    }

    pub fn take_results(&mut self) -> PhaseResults {
        let ping = self.ping_result.take();
        let download = self.download_result.take();
        let upload = self.upload_result.take();
        (ping, download, upload)
    }

    pub fn with_services(mut self, services: std::sync::Arc<dyn Services>) -> Self {
        self.services = services;
        self
    }
}

// ============================================================================
// Phase Implementations (use task_runner for async operations)
// ============================================================================

pub(crate) fn run_early_exit<'a>(
    orch: &'a Orchestrator,
    _ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    let early_exit = orch.early_exit().clone();
    Box::pin(async move {
        // early_exit already cloned above

        if early_exit.show_config_path {
            match crate::config::get_config_path_internal() {
                Some(path) => eprintln!("Configuration file: {}", path.display()),
                None => eprintln!("No configuration path available."),
            }
            return PhaseOutcome::PhaseEarlyExit;
        }

        if let Some(shell) = early_exit.generate_completion {
            let shell_name = match shell {
                crate::cli::ShellType::Bash => "netspeed-cli.bash",
                crate::cli::ShellType::Zsh => "_netspeed-cli",
                crate::cli::ShellType::Fish => "netspeed-cli.fish",
                crate::cli::ShellType::PowerShell => "_netspeed-cli.ps1",
                crate::cli::ShellType::Elvish => "netspeed-cli.elv",
            };
            eprintln!("Shell completions for {shell:?}: {shell_name}");
            return PhaseOutcome::PhaseEarlyExit;
        }

        if early_exit.history {
            match crate::history::show() {
                Ok(()) => PhaseOutcome::PhaseEarlyExit,
                Err(e) => PhaseOutcome::PhaseError(e),
            }
        } else if early_exit.dry_run {
            orch.run_dry_run();
            PhaseOutcome::PhaseEarlyExit
        } else {
            PhaseOutcome::PhaseCompleted
        }
    })
}

pub(crate) fn run_header<'a>(
    orch: &'a Orchestrator,
    _ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    Box::pin(async move {
        if orch.is_verbose() {
            let version = env!("CARGO_PKG_VERSION");
            let nc = crate::terminal::no_color();

            if nc {
                eprintln!();
                eprintln!("  NetSpeed CLI v{version}  ·  speedtest.net");
                eprintln!();
            } else {
                use owo_colors::OwoColorize;
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
        PhaseOutcome::PhaseCompleted
    })
}

pub(crate) fn run_server_discovery<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    let is_verbose = orch.is_verbose();
    let spinner = if is_verbose {
        Some(crate::progress::create_spinner("Finding servers..."))
    } else {
        None
    };

    Box::pin(async move {
        // Discover servers asynchronously using injected service
        let result = ctx.services().server_service().fetch_servers().await;
        let (mut servers, client_location) = match result {
            Ok((servers, location)) => (servers, location),
            Err(e) => return PhaseOutcome::PhaseError(e),
        };
        ctx.set_client_location(client_location);

        if let Some(ref pb) = spinner {
            let theme = orch.config().theme();
            crate::progress::finish_ok(pb, &format!("Found {} servers", servers.len()), theme);
            eprintln!();
        }

        if orch.config().list() {
            if let Err(e) = crate::formatter::format_list(&servers) {
                return PhaseOutcome::PhaseError(e.into());
            }
            ctx.set_list_printed();
            return PhaseOutcome::PhaseEarlyExit;
        }

        if !orch.config().server_ids().is_empty() {
            servers.retain(|s| orch.config().server_ids().contains(&s.id));
        }
        if !orch.config().exclude_ids().is_empty() {
            servers.retain(|s| !orch.config().exclude_ids().contains(&s.id));
        }

        if servers.is_empty() {
            return PhaseOutcome::PhaseError(crate::error::Error::ServerNotFound(
                "No servers match your criteria.".to_string(),
            ));
        }

        let server = match ctx.services().server_service().select_best(&servers) {
            Ok(s) => s,
            Err(e) => return PhaseOutcome::PhaseError(e),
        };

        if is_verbose {
            let dist = crate::common::format_distance(server.distance);
            eprintln!();
            if crate::terminal::no_color() {
                eprintln!("  Server:   {} ({})", server.sponsor, server.name);
                eprintln!("  Location: {} ({dist})", server.country);
            } else {
                use owo_colors::OwoColorize;
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

        ctx.set_server(server);
        PhaseOutcome::PhaseCompleted
    })
}

pub(crate) fn run_ip_discovery<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    Box::pin(async move {
        let is_verbose = orch.is_verbose();
        let result = ctx.services().ip_service().discover_ip().await;
        match result {
            Ok(ip) => ctx.set_client_ip(ip),
            Err(e) => {
                if is_verbose {
                    eprintln!("Warning: Could not discover client IP: {e}");
                }
            }
        }
        PhaseOutcome::PhaseCompleted
    })
}

pub(crate) fn run_ping<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    let no_download = orch.config().no_download();
    let no_upload = orch.config().no_upload();
    if no_download && no_upload {
        return Box::pin(async { PhaseOutcome::PhaseCompleted });
    }

    let server = match ctx.take_server() {
        Some(s) => s,
        None => {
            return Box::pin(async {
                PhaseOutcome::PhaseError(crate::error::Error::context("No server selected"))
            });
        }
    };

    let is_verbose = orch.is_verbose();
    let spinner = if is_verbose {
        Some(crate::progress::create_spinner("Testing latency..."))
    } else {
        None
    };

    let services = ctx.services_arc();

    Box::pin(async move {
        let result = services.server_service().ping_server(&server).await;
        let ping_result = match result {
            Ok(r) => r,
            Err(e) => return PhaseOutcome::PhaseError(e),
        };

        if let Some(ref pb) = spinner {
            let theme = orch.config().theme();
            let msg = if crate::terminal::no_color() {
                format!("Latency: {:.2} ms", ping_result.0)
            } else {
                use owo_colors::OwoColorize;
                format!(
                    "Latency: {}",
                    format!("{:.2} ms", ping_result.0).cyan().bold()
                )
            };
            crate::progress::finish_ok(pb, &msg, theme);
        }

        ctx.set_ping_result((ping_result.0, ping_result.1, ping_result.2, ping_result.3));
        // Put server back for download/upload phases
        ctx.set_server(server);
        PhaseOutcome::PhaseCompleted
    })
}

pub(crate) fn run_download<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    let single = orch.config().single();
    let is_verbose = orch.is_verbose();
    // Only show spinner in non-verbose mode (verbose mode has progress bar which is better)
    let spinner = if !is_verbose {
        Some(crate::progress::create_spinner("Testing download..."))
    } else {
        None
    };

    Box::pin(async move {
        if orch.config().no_download() {
            return PhaseOutcome::PhaseCompleted;
        }

        let server = match ctx.take_server() {
            Some(s) => s,
            None => {
                return PhaseOutcome::PhaseError(crate::error::Error::context(
                    "No server selected",
                ));
            }
        };

        let client = orch.http_client();
        let progress = if is_verbose {
            Arc::new(crate::progress::Tracker::new_animated("Download"))
        } else {
            Arc::new(crate::progress::Tracker::with_target(
                "Download",
                indicatif::ProgressDrawTarget::hidden(),
            ))
        };

        match crate::download::run(client, &server, single, progress).await {
            Ok((avg, peak, total_bytes, samples)) => {
                if let Some(ref pb) = spinner {
                    let theme = orch.config().theme();
                    let msg = if crate::terminal::no_color() {
                        format!("Download: {:.2} Mbps", avg / 1_000_000.0)
                    } else {
                        use owo_colors::OwoColorize;
                        format!(
                            "Download: {}",
                            format!("{:.2} Mbps", avg / 1_000_000.0).green().bold()
                        )
                    };
                    crate::progress::finish_ok(pb, &msg, theme);
                }
                ctx.set_download_result(crate::task_runner::TestRunResult {
                    avg_bps: avg,
                    peak_bps: peak,
                    total_bytes,
                    duration_secs: 0.0,
                    speed_samples: samples,
                    latency_under_load: None,
                });
                // Put server back for upload phase
                ctx.set_server(server);
                PhaseOutcome::PhaseCompleted
            }
            Err(e) => PhaseOutcome::PhaseError(e),
        }
    })
}

pub(crate) fn run_upload<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    let single = orch.config().single();
    let is_verbose = orch.is_verbose();
    // Only show spinner in non-verbose mode (verbose mode has progress bar which is better)
    let spinner = if !is_verbose {
        Some(crate::progress::create_spinner("Testing upload..."))
    } else {
        None
    };

    Box::pin(async move {
        if orch.config().no_upload() {
            return PhaseOutcome::PhaseCompleted;
        }

        let server = match ctx.take_server() {
            Some(s) => s,
            None => {
                return PhaseOutcome::PhaseError(crate::error::Error::context(
                    "No server selected",
                ));
            }
        };

        let client = orch.http_client();
        let progress = if is_verbose {
            Arc::new(crate::progress::Tracker::new_animated("Upload"))
        } else {
            Arc::new(crate::progress::Tracker::with_target(
                "Upload",
                indicatif::ProgressDrawTarget::hidden(),
            ))
        };

        match crate::upload::run(client, &server, single, progress).await {
            Ok((avg, peak, total_bytes, samples)) => {
                if let Some(ref pb) = spinner {
                    let theme = orch.config().theme();
                    let msg = if crate::terminal::no_color() {
                        format!("Upload: {:.2} Mbps", avg / 1_000_000.0)
                    } else {
                        use owo_colors::OwoColorize;
                        format!(
                            "Upload: {}",
                            format!("{:.2} Mbps", avg / 1_000_000.0).green().bold()
                        )
                    };
                    crate::progress::finish_ok(pb, &msg, theme);
                }
                ctx.set_upload_result(crate::task_runner::TestRunResult {
                    avg_bps: avg,
                    peak_bps: peak,
                    total_bytes,
                    duration_secs: 0.0,
                    speed_samples: samples,
                    latency_under_load: None,
                });
                // Put server back for result phase
                ctx.set_server(server);
                PhaseOutcome::PhaseCompleted
            }
            Err(e) => PhaseOutcome::PhaseError(e),
        }
    })
}

// Bandwidth and result phases use async task_runner - handled in legacy for now

pub(crate) fn run_result<'a>(
    orch: &'a Orchestrator,
    ctx: &'a mut PhaseContext,
) -> BoxFuture<'a, PhaseOutcome> {
    Box::pin(async move {
        // Take server info before taking results
        let server_info = match ctx.take_server() {
            Some(s) => crate::types::ServerInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                sponsor: s.sponsor.clone(),
                country: s.country.clone(),
                distance: s.distance,
            },
            None => return PhaseOutcome::PhaseCompleted,
        };

        let (ping_result, download_result, upload_result) = ctx.take_results();

        let (ping, jitter, packet_loss, ping_samples) = match ping_result {
            Some((p, j, pl, s)) => (Some(p), Some(j), Some(pl), s),
            None => (None, None, None, Vec::new()),
        };

        let dl_result = download_result.unwrap_or_default();
        let ul_result = upload_result.unwrap_or_default();

        let mut result = crate::types::TestResult::from_test_runs(
            server_info,
            ping,
            jitter,
            packet_loss,
            &ping_samples,
            &dl_result,
            &ul_result,
            ctx.client_ip().map(|s| s.to_string()),
            ctx.client_location().cloned(),
        );

        let config = orch.config();
        result.phases = crate::types::TestPhases {
            ping: if config.no_download() && config.no_upload() {
                crate::types::PhaseResult::skipped("both bandwidth phases disabled")
            } else {
                crate::types::PhaseResult::completed()
            },
            download: if config.no_download() {
                crate::types::PhaseResult::skipped("disabled by user")
            } else {
                crate::types::PhaseResult::completed()
            },
            upload: if config.no_upload() {
                crate::types::PhaseResult::skipped("disabled by user")
            } else {
                crate::types::PhaseResult::completed()
            },
        };

        if config.should_save_history() {
            if let Err(e) = orch.saver().save(&result) {
                eprintln!("Warning: Failed to save test result: {e}");
            }
        }

        // Delegate to orchestrator for output
        match orch.output_results(
            &mut result,
            &dl_result,
            &ul_result,
            std::time::Duration::from_secs(0),
        ) {
            Ok(()) => PhaseOutcome::PhaseCompleted,
            Err(e) => PhaseOutcome::PhaseError(e),
        }
    })
}

// ============================================================================
// Default Phase Registry
// ============================================================================

pub fn create_default_executor() -> PhaseExecutor {
    PhaseExecutor::new()
        .register(run_early_exit)
        .register(run_header)
        .register(run_server_discovery)
        .register(run_ip_discovery)
        .register(run_ping)
        .register(run_download)
        .register(run_upload)
        .register(run_result)
}

/// Run all phases in order.
pub async fn run_all_phases(orch: &Orchestrator) -> Result<(), Error> {
    let executor = create_default_executor();
    executor.execute_all(orch).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_services() -> std::sync::Arc<dyn Services> {
        let client = reqwest::Client::new();
        std::sync::Arc::new(crate::services::ServiceContainer::new(client))
    }

    #[test]
    fn test_phase_context_default() {
        let ctx = PhaseContext::new(make_test_services());
        assert!(ctx.client_ip().is_none());
        assert!(ctx.server().is_none());
    }

    #[test]
    fn test_phase_context_builder() {
        let ctx = PhaseContext::new(make_test_services()).with_client_ip("192.168.1.1");

        assert_eq!(ctx.client_ip(), Some("192.168.1.1"));
    }

    #[test]
    fn test_phase_executor_register() {
        let _executor = PhaseExecutor::new()
            .register(run_early_exit)
            .register(run_header);
    }
}
