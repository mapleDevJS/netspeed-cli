//! Orchestrates the full speed test lifecycle.
//!
//! Delegates phase execution to the [`phases`](crate::phases) module,
//! which provides OCP via function-based phase definitions.

use crate::config::{Config, ConfigProvider, ConfigSource};

use crate::phase_runner::{DefaultPhaseRunner, PhaseRunner};
use crate::result_processor::{DefaultResultProcessor, ResultProcessor};

use crate::error::Error;
use crate::http;
// HttpClient and ReqwestClient are injected via DI; no direct import needed
use crate::profiles::UserProfile;
use crate::storage::{LoadHistory, SaveResult};
use crate::task_runner::TestRunResult;
use crate::terminal;
use crate::types::TestResult;

/// Early-exit flags extracted from Args — these control flow, not configuration.
#[derive(Clone)]
pub(crate) struct EarlyExitFlags {
    pub(crate) show_config_path: bool,
    pub(crate) generate_completion: Option<crate::cli::ShellType>,
    pub(crate) history: bool,
    pub(crate) dry_run: bool,
}

impl EarlyExitFlags {
    pub(crate) fn from_args(args: &crate::cli::Args) -> Self {
        Self {
            show_config_path: args.show_config_path,
            generate_completion: args.generate_completion,
            history: args.history,
            dry_run: args.dry_run,
        }
    }
}

/// Builder for storage components - enables dependency injection.
pub struct StorageBuilder {
    saver: Option<std::sync::Arc<dyn SaveResult + Send + Sync>>,
    history: Option<std::sync::Arc<dyn LoadHistory + Send + Sync>>,
}

impl StorageBuilder {
    pub fn new() -> Self {
        Self {
            saver: None,
            history: None,
        }
    }

    pub fn with_saver(mut self, saver: impl SaveResult + 'static) -> Self {
        self.saver = Some(std::sync::Arc::new(saver));
        self
    }

    pub fn with_saver_arc(mut self, saver: std::sync::Arc<dyn SaveResult + Send + Sync>) -> Self {
        self.saver = Some(saver);
        self
    }

    pub fn with_history(mut self, history: impl LoadHistory + 'static) -> Self {
        self.history = Some(std::sync::Arc::new(history));
        self
    }

    pub fn with_history_arc(
        mut self,
        history: std::sync::Arc<dyn LoadHistory + Send + Sync>,
    ) -> Self {
        self.history = Some(history);
        self
    }

    /// Build storage components, defaulting to FileStorage if not provided.
    fn build(
        self,
    ) -> (
        std::sync::Arc<dyn SaveResult + Send + Sync>,
        std::sync::Arc<dyn LoadHistory + Send + Sync>,
    ) {
        let saver = self.saver.unwrap_or_else(|| {
            std::sync::Arc::new(crate::storage::FileStorage::new())
                as std::sync::Arc<dyn SaveResult + Send + Sync>
        });
        let history = self.history.unwrap_or_else(|| {
            std::sync::Arc::new(crate::storage::FileStorage::new())
                as std::sync::Arc<dyn LoadHistory + Send + Sync>
        });
        (saver, history)
    }
}

impl Default for StorageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrates the full speed test lifecycle.
///
/// A thin wrapper that holds configuration/resources and delegates
/// phase execution to the phases module.
pub struct Orchestrator {
    pub(crate) config: std::sync::Arc<dyn ConfigProvider>,
    pub(crate) client: reqwest::Client,

    early_exit: EarlyExitFlags,
    saver: std::sync::Arc<dyn SaveResult + Send + Sync>,
    history: std::sync::Arc<dyn LoadHistory + Send + Sync>,
    processor: std::sync::Arc<dyn ResultProcessor + Send + Sync>,

    phase_runner: std::sync::Arc<dyn PhaseRunner + Send + Sync>,
    services: std::sync::Arc<dyn crate::services::Services>,
}

impl Orchestrator {
    // Internal shortcut to the underlying Config
    fn cfg(&self) -> &Config {
        self.config.config()
    }

    /// Create a new orchestrator from CLI arguments.
    pub fn new(
        args: crate::cli::Args,
        file_config: Option<crate::config::File>,
    ) -> Result<Self, Error> {
        let source = ConfigSource::from_args(&args);

        let (config, profile_validation) =
            Config::from_args_with_file(&source, file_config.clone());

        for warning in &profile_validation.warnings {
            eprintln!("Warning: {warning}");
        }

        let file_validation = config.validate_and_report(&source, file_config);
        for error in &file_validation.errors {
            eprintln!("Error: {error}");
        }

        let combined_valid = profile_validation.valid && file_validation.valid;
        if config.strict() && !combined_valid {
            return Err(Error::Context {
                msg: "Configuration validation failed".to_string(),
                source: None,
            });
        }

        let early_exit = EarlyExitFlags::from_args(&args);
        Self::from_config(config, early_exit)
    }

    /// Create an orchestrator from pre-built config.
    pub(crate) fn from_config(config: Config, early_exit: EarlyExitFlags) -> Result<Self, Error> {
        Self::from_config_with_storage(config, early_exit, StorageBuilder::new())
    }

    /// Create an orchestrator from pre-built config with custom storage.
    pub(crate) fn from_config_with_storage(
        config: Config,
        early_exit: EarlyExitFlags,
        storage: StorageBuilder,
    ) -> Result<Self, Error> {
        let http_settings = http::Settings::from(&config);
        let client = http::create_client(&http_settings)?;

        let (saver, history) = storage.build();
        let services = std::sync::Arc::new(crate::services::ServiceContainer::new(client.clone()));

        Ok(Self {
            config: std::sync::Arc::new(config),
            client,

            early_exit,
            saver,
            history,
            processor: std::sync::Arc::new(DefaultResultProcessor),
            phase_runner: std::sync::Arc::new(DefaultPhaseRunner::new()),
            services,
        })
    }

    /// Access the service container.
    #[must_use]
    pub fn services(&self) -> &dyn crate::services::Services {
        self.services.as_ref()
    }

    /// Clone the services Arc for creating PhaseContext.
    pub fn services_arc(&self) -> std::sync::Arc<dyn crate::services::Services> {
        self.services.clone()
    }

    /// Run the full speed test workflow.
    pub async fn run(&self) -> Result<(), Error> {
        self.phase_runner.run_all(self).await
    }

    /// Whether verbose output should be shown.
    #[must_use]
    pub fn is_verbose(&self) -> bool {
        if self.cfg().quiet() {
            return false;
        }
        let format_non_verbose = self.cfg().format().is_some_and(|f| f.is_non_verbose());
        !self.cfg().simple()
            && !self.cfg().json()
            && !self.cfg().csv()
            && !self.cfg().list()
            && !format_non_verbose
    }

    /// Check if this is a simple/quiet mode.
    #[must_use]
    pub fn is_simple_mode(&self) -> bool {
        self.cfg().simple()
            || self.cfg().quiet()
            || self.cfg().format() == Some(crate::config::Format::Simple)
    }

    /// Access the configuration (read-only).
    #[must_use]
    pub fn config(&self) -> &Config {
        self.cfg()
    }

    /// Access early-exit flags.
    #[must_use]
    pub(crate) fn early_exit(&self) -> &EarlyExitFlags {
        &self.early_exit
    }

    /// Access result saver (for persisting a result).
    #[must_use]
    pub fn saver(&self) -> &dyn SaveResult {
        self.saver.as_ref()
    }

    /// Access history provider (optional).
    #[must_use]
    pub fn history(&self) -> &dyn LoadHistory {
        self.history.as_ref()
    }

    /// Access the HTTP client for async operations.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Output results after test completion.
    pub(crate) fn output_results(
        &self,
        result: &mut TestResult,
        dl_result: &TestRunResult,
        ul_result: &TestRunResult,
        elapsed: std::time::Duration,
    ) -> Result<(), Error> {
        let profile = UserProfile::from_name(self.cfg().profile().unwrap_or("power-user"))
            .unwrap_or(UserProfile::PowerUser);
        // Grade results via injected processor (OCP)
        self.processor.process(result, profile);

        let output_format = crate::output_strategy::resolve_output_format(
            self.cfg(),
            dl_result,
            ul_result,
            elapsed,
        );

        if self.is_verbose() {
            self.reveal_results(result, self.cfg().theme(), profile);
        }

        output_format.format(result, self.cfg().bytes())?;
        Ok(())
    }

    /// Show the scan completion reveal before outputting detailed results.
    fn reveal_results(
        &self,
        result: &TestResult,
        theme: crate::theme::Theme,
        profile: UserProfile,
    ) {
        let nc = terminal::no_color();

        let sample_count = result.download_samples.as_ref().map_or(0, Vec::len)
            + result.upload_samples.as_ref().map_or(0, Vec::len)
            + result.ping_samples.as_ref().map_or(0, Vec::len);

        let overall_grade = crate::grades::grade_overall(
            result.ping,
            result.jitter,
            result.download,
            result.upload,
            profile,
        );

        let grade_badge = overall_grade.color_str(nc, theme);
        let grade_plain = overall_grade.as_str().to_string();
        crate::progress::reveal_scan_complete(sample_count, &grade_badge, &grade_plain, nc, theme);
        crate::progress::reveal_pause();
    }

    fn print_kv(nc: bool, key: &str, value: &str) {
        if nc {
            eprintln!("  {key}: {value}");
        } else {
            use owo_colors::OwoColorize;
            eprintln!("  {}: {}", key.dimmed(), value.cyan());
        }
    }

    /// Validate configuration and print confirmation without running tests.
    pub(crate) fn run_dry_run(&self) {
        let nc = terminal::no_color();
        let config = self.config();

        if nc {
            eprintln!("Configuration valid:");
        } else {
            use owo_colors::OwoColorize;
            eprintln!("{}", "Configuration valid:".green().bold());
        }

        Self::print_kv(nc, "Timeout", &format!("{}s", config.timeout()));
        Self::print_kv(nc, "Format", self.format_description());
        if config.quiet() {
            Self::print_kv(nc, "Quiet", "enabled");
        }
        if let Some(source) = config.source() {
            Self::print_kv(nc, "Source IP", source);
        }
        if config.no_download() {
            Self::print_kv(nc, "Download test", "disabled");
        }
        if config.no_upload() {
            Self::print_kv(nc, "Upload test", "disabled");
        }
        if config.single() {
            Self::print_kv(nc, "Streams", "single");
        }
        if let Some(ca_cert) = config.ca_cert() {
            Self::print_kv(nc, "CA cert", ca_cert);
        }
        if let Some(tls_version) = config.tls_version() {
            Self::print_kv(nc, "TLS version", tls_version);
        }
        if config.pin_certs() {
            Self::print_kv(nc, "Cert pinning", "enabled");
        }

        if nc {
            eprintln!("\nDry run complete. Run without --dry-run to perform speed test.");
        } else {
            use owo_colors::OwoColorize;
            eprintln!(
                "\n{}",
                "Dry run complete. Run without --dry-run to perform speed test.".bright_black()
            );
        }
    }

    /// Return a human-readable description of the output format.
    fn format_description(&self) -> &'static str {
        match self.cfg().format() {
            Some(f) => f.label(),
            None => "Detailed (default)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigSource, OutputSource};

    fn orch_from_source(source: &ConfigSource, early_exit: EarlyExitFlags) -> Orchestrator {
        let config = Config::from_source(source);
        Orchestrator::from_config(config, early_exit).unwrap()
    }

    fn default_early_exit() -> EarlyExitFlags {
        EarlyExitFlags {
            show_config_path: false,
            generate_completion: None,
            history: false,
            dry_run: false,
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let source = ConfigSource::default();
        let config = Config::from_source(&source);
        let orch = Orchestrator::from_config(config, default_early_exit());
        assert!(orch.is_ok());
    }

    #[test]
    fn test_is_verbose_default() {
        let source = ConfigSource::default();
        let orch = orch_from_source(&source, default_early_exit());
        assert!(orch.is_verbose());
    }

    #[test]
    fn test_is_verbose_quiet() {
        let source = ConfigSource {
            output: OutputSource {
                quiet: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_verbose());
    }

    #[test]
    fn test_is_simple_mode_default() {
        let source = ConfigSource::default();
        let orch = orch_from_source(&source, default_early_exit());
        assert!(!orch.is_simple_mode());
    }

    #[test]
    fn test_is_simple_mode_simple() {
        let source = ConfigSource {
            output: OutputSource {
                simple: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        let orch = orch_from_source(&source, default_early_exit());
        assert!(orch.is_simple_mode());
    }

    #[test]
    fn test_dry_run_succeeds() {
        let source = ConfigSource::default();
        let early_exit = EarlyExitFlags {
            dry_run: true,
            ..default_early_exit()
        };
        let orch = orch_from_source(&source, early_exit);
        orch.run_dry_run();
    }

    #[test]
    fn test_early_exit_flags_default() {
        let flags = default_early_exit();
        assert!(!flags.show_config_path);
        assert!(flags.generate_completion.is_none());
        assert!(!flags.history);
        assert!(!flags.dry_run);
    }

    #[test]
    fn test_storage_builder_defaults() {
        let shared = std::sync::Arc::new(crate::storage::MockStorage::new());
        let saver = shared.clone() as std::sync::Arc<dyn crate::storage::SaveResult + Send + Sync>;
        let history =
            shared.clone() as std::sync::Arc<dyn crate::storage::LoadHistory + Send + Sync>;

        let builder = StorageBuilder::new()
            .with_saver_arc(saver)
            .with_history_arc(history);
        let (saver, history) = builder.build();
        <dyn crate::storage::SaveResult>::save(&*saver, &crate::types::TestResult::default())
            .unwrap();
        let _ = <dyn crate::storage::LoadHistory>::load_recent(&*history, 1);
    }

    #[test]
    fn test_storage_builder_custom() {
        let shared = std::sync::Arc::new(crate::storage::MockStorage::new());
        let saver = shared.clone() as std::sync::Arc<dyn crate::storage::SaveResult + Send + Sync>;
        let history =
            shared.clone() as std::sync::Arc<dyn crate::storage::LoadHistory + Send + Sync>;

        let builder = StorageBuilder::new()
            .with_saver_arc(saver)
            .with_history_arc(history);

        let (saver, history) = builder.build();

        let result = crate::types::TestResult::default();
        <dyn crate::storage::SaveResult>::save(&*saver, &result).unwrap();
        let loaded = <dyn crate::storage::LoadHistory>::load_recent(&*history, 10).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn test_orchestrator_exposes_services() {
        let args = crate::cli::Args::default();
        let orch = Orchestrator::new(args, None).unwrap();
        let _services = orch.services();
    }

    // run_dry_run branch coverage — one test per conditional field.
    // We don't capture stderr; the goal is to exercise each branch without panic.
    fn dry_run_orch(
        output: OutputSource,
        test: crate::config::TestSource,
        network: crate::config::NetworkSource,
    ) -> Orchestrator {
        let source = ConfigSource {
            output,
            test,
            network,
            ..Default::default()
        };
        orch_from_source(
            &source,
            EarlyExitFlags {
                dry_run: true,
                ..default_early_exit()
            },
        )
    }

    #[test]
    fn test_dry_run_no_color_mode() {
        // NO_COLOR is set by the serial test suite; exercise the nc=true branch explicitly
        let orch = dry_run_orch(Default::default(), Default::default(), Default::default());
        orch.run_dry_run(); // must not panic
    }

    #[test]
    fn test_dry_run_quiet_branch() {
        let orch = dry_run_orch(
            OutputSource {
                quiet: Some(true),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
        );
        orch.run_dry_run();
    }

    #[test]
    fn test_dry_run_no_download_branch() {
        let orch = dry_run_orch(
            Default::default(),
            crate::config::TestSource {
                no_download: Some(true),
                ..Default::default()
            },
            Default::default(),
        );
        orch.run_dry_run();
    }

    #[test]
    fn test_dry_run_no_upload_branch() {
        let orch = dry_run_orch(
            Default::default(),
            crate::config::TestSource {
                no_upload: Some(true),
                ..Default::default()
            },
            Default::default(),
        );
        orch.run_dry_run();
    }

    #[test]
    fn test_dry_run_single_stream_branch() {
        let orch = dry_run_orch(
            Default::default(),
            crate::config::TestSource {
                single: Some(true),
                ..Default::default()
            },
            Default::default(),
        );
        orch.run_dry_run();
    }

    #[test]
    #[ignore = "requires a bound local IP; tested in http::tests"]
    fn test_dry_run_source_ip_branch() {
        let orch = dry_run_orch(
            Default::default(),
            Default::default(),
            crate::config::NetworkSource {
                source: Some("127.0.0.1:0".to_string()),
                ..Default::default()
            },
        );
        orch.run_dry_run();
    }

    #[test]
    #[ignore = "requires Rustls CryptoProvider; tested in http::tests"]
    fn test_dry_run_tls_version_branch() {
        let orch = dry_run_orch(
            Default::default(),
            Default::default(),
            crate::config::NetworkSource {
                tls_version: Some("1.3".to_string()),
                ..Default::default()
            },
        );
        orch.run_dry_run();
    }

    #[test]
    #[ignore = "requires Rustls CryptoProvider; tested in http::tests"]
    fn test_dry_run_pin_certs_branch() {
        let orch = dry_run_orch(
            Default::default(),
            Default::default(),
            crate::config::NetworkSource {
                pin_certs: Some(true),
                ..Default::default()
            },
        );
        orch.run_dry_run();
    }
}
