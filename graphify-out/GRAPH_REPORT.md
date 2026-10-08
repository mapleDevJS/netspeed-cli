# Graph Report - /Users/alexey.ivanov/vibe.dev/apps/netspeed-cli  (2026-10-08)

## Corpus Check
- 68 files · ~337,578 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1808 nodes · 4233 edges · 33 communities detected
- Extraction: 73% EXTRACTED · 27% INFERRED · 0% AMBIGUOUS · INFERRED: 1151 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]

## God Nodes (most connected - your core abstractions)
1. `Args` - 44 edges
2. `make_test_result()` - 42 edges
3. `no_color()` - 39 edges
4. `PhaseContext` - 34 edges
5. `Config` - 33 edges
6. `create_client()` - 31 edges
7. `make_tracker()` - 30 edges
8. `cli()` - 29 edges
9. `save_result_to_path()` - 27 edges
10. `validate_with_strict()` - 23 edges

## Surprising Connections (you probably didn't know these)
- `test_e2e_full_speedtest_flow()` --calls--> `select_best_server()`  [INFERRED]
  /Users/alexey.ivanov/vibe.dev/apps/netspeed-cli/tests/e2e_test.rs → src/domain/server.rs
- `test_output_strategy_resolver()` --calls--> `resolve_output_format()`  [INFERRED]
  /Users/alexey.ivanov/vibe.dev/netspeed-cli/tests/solidity_tests.rs → src/output_strategy.rs
- `bench_calculate_bandwidth()` --calls--> `calculate_bandwidth()`  [INFERRED]
  benches/core_benchmarks.rs → /Users/alexey.ivanov/vibe.dev/apps/netspeed-cli/src/common.rs
- `bench_calculate_bandwidth_zero_elapsed()` --calls--> `calculate_bandwidth()`  [INFERRED]
  benches/core_benchmarks.rs → /Users/alexey.ivanov/vibe.dev/apps/netspeed-cli/src/common.rs
- `bench_calculate_distance()` --calls--> `calculate_distance()`  [INFERRED]
  benches/core_benchmarks.rs → src/domain/server.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.02
Nodes (187): test_settings_from_config_default_user_agent(), test_settings_from_config_retry_enabled_by_default(), test_settings_from_config_timeout(), test_settings_from_config_with_ca_cert(), test_settings_from_config_with_pinning(), test_settings_from_config_with_source_ip(), test_settings_from_config_with_tls_version(), test_config_from_default_source() (+179 more)

### Community 1 - "Community 1"
Cohesion: 0.03
Nodes (124): bar_chart(), determine_stream_count(), format_data_size_tabular(), format_distance(), format_duration_tabular(), format_jitter_tabular(), format_latency_tabular(), format_loss_tabular() (+116 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (58): test_machine_error_identity_csv(), Config, default_early_exit(), dry_run_orch(), EarlyExitFlags, orch_from_source(), Orchestrator, StorageBuilder (+50 more)

### Community 3 - "Community 3"
Cohesion: 0.03
Nodes (107): is_valid_ipv4(), Error, ErrorCategory, test_context_error_display(), test_context_with_source(), test_context_without_source(), test_debug_trait(), test_error_source_chain() (+99 more)

### Community 4 - "Community 4"
Cohesion: 0.03
Nodes (117): AbortStreams, BandwidthResult, cancelling_measurement_aborts_streams(), concurrent_sampling_claims_one_interval(), finish_does_not_turn_a_short_tail_into_a_peak_sample(), interval_samples_detect_stalls_and_recovery(), LoopState, make_tracker() (+109 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (56): boxed_header(), format_grade_line(), grade_badge(), grade_download(), grade_jitter(), grade_overall(), grade_ping(), grade_stability() (+48 more)

### Community 6 - "Community 6"
Cohesion: 0.06
Nodes (71): main(), Args, create_mock_speedtest_server(), test_e2e_download_only(), test_e2e_full_speedtest_flow(), test_e2e_upload_only(), cli(), test_cli_help() (+63 more)

### Community 7 - "Community 7"
Cohesion: 0.04
Nodes (64): bufferbloat_colorized(), bufferbloat_grade(), colorize_rating(), connection_rating(), degradation_str(), format_overall_rating(), format_speed_colored(), format_speed_plain() (+56 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (62): backup_path(), clear(), clear_history_from_path(), concurrent_saves_preserve_all_entries(), corrupt_path(), Entry, get_history_path(), load_entries() (+54 more)

### Community 9 - "Community 9"
Cohesion: 0.06
Nodes (62): all_categories(), compute_all_statuses(), compute_scenario_status(), format_scenario_grid(), headroom_level(), HeadroomLevel, print_scenario_grid(), render_capacity_bar() (+54 more)

### Community 10 - "Community 10"
Cohesion: 0.06
Nodes (47): format_data_size(), bench_build_test_url(), bench_build_upload_url(), bench_calculate_bandwidth(), bench_calculate_bandwidth_zero_elapsed(), bench_calculate_distance(), bench_extract_base_url(), bench_format_data_size() (+39 more)

### Community 11 - "Community 11"
Cohesion: 0.05
Nodes (36): is_config_error(), is_list_sentinel(), is_network_error(), machine_error_format(), machine_error_identity(), MachineErrorBody, MachineErrorOutput, print_error() (+28 more)

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (30): ClientLocation, compute_ci_95(), compute_cv(), CsvOutput, DefaultStats, deserialize_distance(), PhaseResult, PhaseState (+22 more)

### Community 13 - "Community 13"
Cohesion: 0.07
Nodes (22): calculate_distance(), fetch(), measure_latency_under_load(), select_best_server(), ServerDiscovery, test_calculate_distance_nyc_to_la(), test_calculate_distance_same_location(), test_select_best_server_empty() (+14 more)

### Community 14 - "Community 14"
Cohesion: 0.08
Nodes (24): current_level(), debug(), error(), format_json_entry(), is_verbose(), Level, log(), test_current_level_returns_info_by_default() (+16 more)

### Community 15 - "Community 15"
Cohesion: 0.1
Nodes (33): build(), build_profile_targets(), build_targets(), FileEstimate, format_targets(), format_time_estimate(), show(), Target (+25 more)

### Community 16 - "Community 16"
Cohesion: 0.13
Nodes (18): minimum_tls_12_negotiates_both_versions(), custom_history_paths_are_isolated_and_clear_removes_backups(), FileStorage, history_round_trips_complete_reports_and_reads_legacy_entries(), HistoryStorage, LoadHistory, make_test_result(), MockStorage (+10 more)

### Community 17 - "Community 17"
Cohesion: 0.08
Nodes (11): OutputFormatType, ShellType, test_validate_ca_cert_path_is_directory(), test_validate_ca_cert_path_not_found(), test_validate_ca_cert_path_valid(), test_validate_timeout_too_large(), test_validate_timeout_zero(), validate_ca_cert_path() (+3 more)

### Community 18 - "Community 18"
Cohesion: 0.13
Nodes (5): test_test_metrics_impl_default(), test_test_run_result_default_explicit(), test_test_run_result_default_values(), TestMetrics, TestRunResult

### Community 19 - "Community 19"
Cohesion: 0.46
Nodes (15): make_config(), make_test_run(), make_upload_run(), resolve_output_format(), test_resolve_output_format_compact(), test_resolve_output_format_csv(), test_resolve_output_format_dashboard(), test_resolve_output_format_default_none() (+7 more)

### Community 20 - "Community 20"
Cohesion: 0.18
Nodes (4): resolve(), test_resolve_theme_default(), test_validate_invalid(), Theme

### Community 21 - "Community 21"
Cohesion: 0.27
Nodes (7): test_default_values(), test_retry_delay_beyond_max_attempts(), test_retry_delay_exhausted(), test_retry_delay_first_attempt(), test_retry_delay_second_attempt(), test_retry_delay_third_attempt(), TestConfig

### Community 22 - "Community 22"
Cohesion: 0.2
Nodes (5): ConfigProvider, File, NetworkConfig, ServerSelection, TestSelection

### Community 23 - "Community 23"
Cohesion: 0.29
Nodes (4): estimates(), main(), regressions(), BenchmarkTests

### Community 24 - "Community 24"
Cohesion: 0.22
Nodes (5): ConfigSource, NetworkSource, OutputSource, ServerSource, TestSource

### Community 25 - "Community 25"
Cohesion: 0.67
Nodes (1): NetspeedCli

### Community 26 - "Community 26"
Cohesion: 1.0
Nodes (0):

### Community 27 - "Community 27"
Cohesion: 1.0
Nodes (0):

### Community 28 - "Community 28"
Cohesion: 1.0
Nodes (0):

### Community 29 - "Community 29"
Cohesion: 1.0
Nodes (0):

### Community 30 - "Community 30"
Cohesion: 1.0
Nodes (0):

### Community 31 - "Community 31"
Cohesion: 1.0
Nodes (0):

### Community 32 - "Community 32"
Cohesion: 1.0
Nodes (0):

## Knowledge Gaps
- **56 isolated node(s):** `TestServer`, `TestServersWrapper`, `TestServerConfig`, `TestMetrics`, `ServerConfig` (+51 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 26`** (2 nodes): `run_all_phases()`, `speedtest.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 27`** (1 nodes): `commitlint.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 28`** (1 nodes): `_netspeed-cli.ps1`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 29`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 30`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (1 nodes): `measurement.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 32`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `no_color()` connect `Community 1` to `Community 2`, `Community 4`, `Community 5`, `Community 6`, `Community 9`, `Community 10`, `Community 11`, `Community 20`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Why does `format_simple()` connect `Community 1` to `Community 4`, `Community 6`, `Community 7`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Are the 43 inferred relationships involving `Args` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Args` has 43 INFERRED edges - model-reasoned connections that need verification._
- **Are the 35 inferred relationships involving `no_color()` (e.g. with `print_error()` and `print_suggestion()`) actually correct?**
  _`no_color()` has 35 INFERRED edges - model-reasoned connections that need verification._
- **What connects `TestServer`, `TestServersWrapper`, `TestServerConfig` to the rest of the system?**
  _56 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.02 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.03 - nodes in this community are weakly interconnected._
