# Graph Report - /Users/alexey.ivanov/Developer/netspeed-cli  (2026-11-21)

## Corpus Check
- 77 files · ~373,628 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2026 nodes · 4984 edges · 37 communities detected
- Extraction: 70% EXTRACTED · 30% INFERRED · 0% AMBIGUOUS · INFERRED: 1485 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]

## God Nodes (most connected - your core abstractions)
1. `Output` - 50 edges
2. `Args` - 49 edges
3. `make_test_result()` - 48 edges
4. `no_color()` - 45 edges
5. `PhaseContext` - 35 edges
6. `Config` - 33 edges
7. `cli()` - 30 edges
8. `make_tracker()` - 30 edges
9. `create_client()` - 30 edges
10. `run_server_discovery()` - 29 edges

## Surprising Connections (you probably didn't know these)
- `no_color()` --calls--> `test_resolve_theme_default()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/terminal.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/theme.rs
- `grade_bufferbloat()` --calls--> `profile_interpretation()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/grades.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/mod.rs
- `format_jitter_tabular()` --calls--> `build_latency_section()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/common.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/sections.rs
- `format_loss_tabular()` --calls--> `build_latency_section()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/common.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/sections.rs
- `format_connection_info()` --calls--> `format_detailed()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/sections.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/mod.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.02
Nodes (109): format_data_size(), get_terminal_width(), bench_build_test_url(), bench_build_upload_url(), bench_calculate_bandwidth(), bench_calculate_bandwidth_zero_elapsed(), bench_calculate_distance(), bench_extract_base_url() (+101 more)

### Community 1 - "Community 1"
Cohesion: 0.02
Nodes (168): test_config_from_default_source(), test_config_from_source_all_format_variants(), test_config_from_source_fully_custom(), test_config_from_source_only_network(), test_config_from_source_only_output(), test_config_from_source_only_servers(), test_config_from_source_only_test_selection(), test_config_source_composes_sub_sources() (+160 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (118): main(), format_distance(), format_latency_tabular(), ascii_report(), ascii_report_preserves_metrics_and_uses_plain_terminal_characters(), connection_info(), data_summary(), format_rate() (+110 more)

### Community 3 - "Community 3"
Cohesion: 0.03
Nodes (98): test_context_with_source(), test_error_source_chain(), build_tls_config(), create_client(), default_root_store(), DomainRestrictedVerifier, is_transient_error(), load_custom_ca_cert() (+90 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (59): boxed_header(), header_layout_fits_supported_terminal_widths(), format_grade_line(), grade_badge(), grade_bufferbloat(), grade_download(), grade_jitter(), grade_overall() (+51 more)

### Community 5 - "Community 5"
Cohesion: 0.03
Nodes (84): calculate_bandwidth(), is_valid_ipv4(), build_test_url(), download_stream(), DownloadConfig, extract_base_url(), hidden_tracker(), invalid_download_limits_fail_without_io() (+76 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (56): AbortStreams, BandwidthResult, cancelling_measurement_aborts_streams(), concurrent_sampling_claims_one_interval(), finish_does_not_turn_a_short_tail_into_a_peak_sample(), interval_samples_detect_stalls_and_recovery(), LoopState, make_tracker() (+48 more)

### Community 7 - "Community 7"
Cohesion: 0.05
Nodes (73): grade_stability(), build_minimal_line(), csv_fixture_preserves_fields_units_and_legacy_header_behavior(), format_compact(), format_csv(), format_detailed(), format_json(), format_jsonl() (+65 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (69): Args, cli(), test_cli_help(), test_cli_version(), test_combined_flags(), test_csv_flag_parsing(), test_error_output_format(), test_exit_code_on_error() (+61 more)

### Community 9 - "Community 9"
Cohesion: 0.08
Nodes (65): backup_path(), clear(), clear_history_from_path(), concurrent_saves_preserve_all_entries(), corrupt_path(), Entry, get_averages(), get_history_path() (+57 more)

### Community 10 - "Community 10"
Cohesion: 0.04
Nodes (41): is_config_error(), is_list_sentinel(), is_network_error(), machine_error_format(), machine_error_identity(), MachineErrorBody, MachineErrorOutput, print_error() (+33 more)

### Community 11 - "Community 11"
Cohesion: 0.06
Nodes (62): all_categories(), compute_all_statuses(), compute_scenario_status(), format_scenario_grid(), headroom_level(), HeadroomLevel, print_scenario_grid(), render_capacity_bar() (+54 more)

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (32): default_early_exit(), dry_run_orch(), EarlyExitFlags, FailingPhaseRunner, library_result_phase_does_not_save_history(), library_run_propagates_measurement_errors(), library_run_returns_report_and_disables_verbose_mode(), live_progress_respects_explicit_disable_and_machine_formats() (+24 more)

### Community 13 - "Community 13"
Cohesion: 0.06
Nodes (36): defaults_enable_both_phases_and_use_normal_streaming(), rejects_invalid_source_address_before_network_activity(), rejects_timeout_outside_supported_range_before_network_activity(), run_speed_test(), TestOptions, Error, ErrorCategory, test_context_error_display() (+28 more)

### Community 14 - "Community 14"
Cohesion: 0.05
Nodes (25): estimates(), main(), regressions(), create_mock_speedtest_server(), test_e2e_download_only(), test_e2e_full_speedtest_flow(), test_e2e_upload_only(), test_ping_test_failure_mocked() (+17 more)

### Community 15 - "Community 15"
Cohesion: 0.05
Nodes (30): bar_chart(), determine_stream_count(), format_data_size_tabular(), format_duration_tabular(), format_jitter_tabular(), format_loss_tabular(), format_speed_tabular(), get_terminal_width_bounded() (+22 more)

### Community 16 - "Community 16"
Cohesion: 0.07
Nodes (30): ClientLocation, compute_ci_95(), compute_cv(), CsvOutput, DefaultStats, deserialize_distance(), PhaseResult, PhaseState (+22 more)

### Community 17 - "Community 17"
Cohesion: 0.08
Nodes (23): current_level(), debug(), error(), format_json_entry(), is_verbose(), Level, log(), test_current_level_returns_info_by_default() (+15 more)

### Community 18 - "Community 18"
Cohesion: 0.1
Nodes (33): build(), build_profile_targets(), build_targets(), FileEstimate, format_targets(), format_time_estimate(), show(), Target (+25 more)

### Community 19 - "Community 19"
Cohesion: 0.16
Nodes (9): git(), main(), open_pr(), prepare(), push_pr(), release_checks(), run(), version_tuple() (+1 more)

### Community 20 - "Community 20"
Cohesion: 0.14
Nodes (13): DefaultIpService, DefaultLatencyMonitor, IpDiscoverer, LatencyMonitor, make_client(), ServerFetcher, ServerPinger, ServerSelector (+5 more)

### Community 21 - "Community 21"
Cohesion: 0.32
Nodes (18): dashboard_receives_byte_mode_and_skipped_phase_state(), make_config(), make_test_run(), make_upload_run(), resolve_output_format(), test_resolve_output_format_compact(), test_resolve_output_format_csv(), test_resolve_output_format_dashboard() (+10 more)

### Community 22 - "Community 22"
Cohesion: 0.13
Nodes (5): test_test_metrics_impl_default(), test_test_run_result_default_explicit(), test_test_run_result_default_values(), TestMetrics, TestRunResult

### Community 23 - "Community 23"
Cohesion: 0.21
Nodes (3): LocalCiHookTests, Exercise the pre-push candidate boundary using disposable Git repositories., WorkflowTriggerPolicyTests

### Community 24 - "Community 24"
Cohesion: 0.18
Nodes (4): resolve(), test_resolve_theme_default(), test_validate_invalid(), Theme

### Community 25 - "Community 25"
Cohesion: 0.27
Nodes (6): ReleaseValidationEvidenceTests, main(), run(), validate_run(), validation_run_id(), verify()

### Community 26 - "Community 26"
Cohesion: 0.2
Nodes (5): ConfigProvider, File, NetworkConfig, ServerSelection, TestSelection

### Community 27 - "Community 27"
Cohesion: 0.28
Nodes (3): is_valid_ipv4(), is_valid_ipv6(), validate_ip_address()

### Community 28 - "Community 28"
Cohesion: 0.22
Nodes (5): ConfigSource, NetworkSource, OutputSource, ServerSource, TestSource

### Community 29 - "Community 29"
Cohesion: 0.67
Nodes (1): NetspeedCli

### Community 30 - "Community 30"
Cohesion: 1.0
Nodes (0):

### Community 31 - "Community 31"
Cohesion: 1.0
Nodes (0):

### Community 32 - "Community 32"
Cohesion: 1.0
Nodes (0):

### Community 33 - "Community 33"
Cohesion: 1.0
Nodes (0):

### Community 34 - "Community 34"
Cohesion: 1.0
Nodes (0):

### Community 35 - "Community 35"
Cohesion: 1.0
Nodes (0):

### Community 36 - "Community 36"
Cohesion: 1.0
Nodes (0):

## Knowledge Gaps
- **60 isolated node(s):** `TestServer`, `TestServersWrapper`, `TestServerConfig`, `Exercise the pre-push candidate boundary using disposable Git repositories.`, `TestMetrics` (+55 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 30`** (2 nodes): `speedtest.rs`, `run_all_phases()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (1 nodes): `commitlint.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 32`** (1 nodes): `_netspeed-cli.ps1`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 33`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 34`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 35`** (1 nodes): `measurement.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 36`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `no_color()` connect `Community 0` to `Community 2`, `Community 4`, `Community 6`, `Community 7`, `Community 8`, `Community 10`, `Community 11`, `Community 12`, `Community 24`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Why does `UserProfile` connect `Community 4` to `Community 0`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Why does `run_server_discovery()` connect `Community 0` to `Community 2`, `Community 5`, `Community 7`, `Community 13`, `Community 20`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Are the 45 inferred relationships involving `Output` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Output` has 45 INFERRED edges - model-reasoned connections that need verification._
- **Are the 48 inferred relationships involving `Args` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Args` has 48 INFERRED edges - model-reasoned connections that need verification._
- **Are the 41 inferred relationships involving `no_color()` (e.g. with `print_error()` and `print_suggestion()`) actually correct?**
  _`no_color()` has 41 INFERRED edges - model-reasoned connections that need verification._
- **What connects `TestServer`, `TestServersWrapper`, `TestServerConfig` to the rest of the system?**
  _60 weakly-connected nodes found - possible documentation gaps or missing edges._
