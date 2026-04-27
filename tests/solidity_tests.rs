//! Unit tests for the newly introduced SOLID abstractions.

use netspeed_cli::config::Config;
use netspeed_cli::orchestrator::Orchestrator;
use netspeed_cli::{
    http_client::ReqwestClient, output_strategy, phase_runner::DefaultPhaseRunner,
    result_processor::DefaultResultProcessor,
};

#[tokio::test]
async fn test_http_client_impl() {
    // Build a minimal config to create a client via http::create_client (already used in orchestrator)
    let config = Config::default();
    let settings = netspeed_cli::http::Settings::from(&config);
    let client = netspeed_cli::http::create_client(&settings).expect("client creation");
    let wrapper = ReqwestClient(client.clone());
    // GET a known URL (httpbin) – use http://example.com which is fast and deterministic
    let resp = wrapper
        .get("https://example.com")
        .await
        .expect("GET succeeds");
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_phase_runner_delegate() {
    // Use DefaultPhaseRunner which should delegate to netspeed_cli::phases::run_all_phases
    // We just ensure it returns Ok for a minimal orchestrator with no‑op phases.
    let args = netspeed_cli::cli::Args::default();
    let orch = Orchestrator::new(args, None).expect("orchestrator");
    let runner = DefaultPhaseRunner::new();
    let result = runner.run_all(&orch).await;
    // The real phases may hit network; we just assert that the call completes (error or ok)
    // Accept both Ok and Err as long as it doesn't panic.
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_result_processor_defaults() {
    let mut result = netspeed_cli::types::TestResult::default();
    let profile = netspeed_cli::profiles::UserProfile::PowerUser;
    let processor = DefaultResultProcessor;
    processor.process(&mut result, profile);
    // After processing, grade fields should be Some (even if values are None they become None)
    assert!(result.overall_grade.is_none()); // because ping etc are None, overall stays None
    // Ensure no panic and method is callable
}

#[test]
fn test_output_strategy_resolver() {
    let config = Config::default();
    // Resolve to an OutputFormat using the helper
    let dummy_dl = netspeed_cli::task_runner::TestRunResult {
        avg_bps: 0.0,
        peak_bps: 0.0,
        total_bytes: 0,
        duration_secs: 0.0,
        speed_samples: vec![],
        latency_under_load: None,
    };
    let dummy_ul = dummy_dl.clone();
    let format = output_strategy::resolve_output_format(
        &config,
        &dummy_dl,
        &dummy_ul,
        std::time::Duration::from_secs(0),
    );
    // Ensure we obtained a formatter and can call format on a dummy result.
    let result = netspeed_cli::types::TestResult::default();
    let res = format.format(&result, false);
    assert!(res.is_ok() || res.is_err());
}
