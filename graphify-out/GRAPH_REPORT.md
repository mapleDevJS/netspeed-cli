# Graph Report - /Users/alexey.ivanov/Developer/netspeed-cli  (2026-11-21)

## Corpus Check
- 77 files · ~374,011 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2029 nodes · 4992 edges · 35 communities detected
- Extraction: 70% EXTRACTED · 30% INFERRED · 0% AMBIGUOUS · INFERRED: 1487 edges (avg confidence: 0.8)
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
- `no_color()` --calls--> `test_no_color_default()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/terminal.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/progress.rs
- `main()` --calls--> `render()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/build.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/dashboard.rs
- `cli()` --calls--> `Env`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/tests/integration_tls_cli_test.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/terminal.rs
- `pty_fixture_child()` --calls--> `run_or_interrupt()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/tests/pty_lifecycle_test.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/interrupt.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.02
Nodes (177): test_settings_from_config_default_user_agent(), test_settings_from_config_retry_enabled_by_default(), test_settings_from_config_timeout(), test_settings_from_config_with_ca_cert(), test_settings_from_config_with_pinning(), test_settings_from_config_with_source_ip(), test_settings_from_config_with_tls_version(), test_config_from_default_source() (+169 more)

### Community 1 - "Community 1"
Cohesion: 0.02
Nodes (113): is_valid_ipv4(), bench_is_valid_ipv4(), Error, ErrorCategory, test_context_error_display(), test_context_with_source(), test_context_without_source(), test_debug_trait() (+105 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (65): chart_keeps_zero_and_missing_sample_gap(), completed_zero_and_unavailable_transfer_are_distinct(), display_publication_is_bounded_and_nonblocking(), fixture(), Handle, Layout, lifecycle_keeps_completed_data_and_distinguishes_skips_failure_and_interruption(), live_frames_fit_all_supported_sizes_and_byte_mode() (+57 more)

### Community 3 - "Community 3"
Cohesion: 0.03
Nodes (130): format_distance(), format_jitter_tabular(), format_latency_tabular(), format_loss_tabular(), bench_format_distance(), ascii_report(), ascii_report_preserves_metrics_and_uses_plain_terminal_characters(), connection_info() (+122 more)

### Community 4 - "Community 4"
Cohesion: 0.03
Nodes (85): AbortStreams, BandwidthResult, cancelling_measurement_aborts_streams(), concurrent_sampling_claims_one_interval(), finish_does_not_turn_a_short_tail_into_a_peak_sample(), interval_samples_detect_stalls_and_recovery(), LoopState, make_tracker() (+77 more)

### Community 5 - "Community 5"
Cohesion: 0.03
Nodes (63): main(), hidden_tracker(), local_server(), socket_regression_download_caps_exclude_warmup_and_span_streams(), socket_regression_download_phase_deadline_and_empty_responses(), create_mock_speedtest_server(), test_e2e_download_only(), test_e2e_full_speedtest_flow() (+55 more)

### Community 6 - "Community 6"
Cohesion: 0.03
Nodes (46): boxed_header(), header_layout_fits_supported_terminal_widths(), format_grade_line(), grade_badge(), grade_download(), grade_jitter(), grade_overall(), grade_ping() (+38 more)

### Community 7 - "Community 7"
Cohesion: 0.05
Nodes (75): grade_bufferbloat(), grade_stability(), build_minimal_line(), csv_fixture_preserves_fields_units_and_legacy_header_behavior(), format_compact(), format_csv(), format_detailed(), format_json() (+67 more)

### Community 8 - "Community 8"
Cohesion: 0.04
Nodes (52): HttpClient, ReqwestClient, TransferState, LocalDiscovery, build_footer(), calculate_distance(), fetch(), select_best_server() (+44 more)

### Community 9 - "Community 9"
Cohesion: 0.04
Nodes (41): is_config_error(), is_list_sentinel(), is_network_error(), machine_error_format(), machine_error_identity(), MachineErrorBody, MachineErrorOutput, print_error() (+33 more)

### Community 10 - "Community 10"
Cohesion: 0.08
Nodes (64): backup_path(), clear(), clear_history_from_path(), concurrent_saves_preserve_all_entries(), corrupt_path(), Entry, get_averages(), get_history_path() (+56 more)

### Community 11 - "Community 11"
Cohesion: 0.06
Nodes (62): all_categories(), compute_all_statuses(), compute_scenario_status(), format_scenario_grid(), headroom_level(), HeadroomLevel, print_scenario_grid(), render_capacity_bar() (+54 more)

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (26): minimum_tls_12_negotiates_both_versions(), git(), open_pr(), prepare(), push_pr(), release_checks(), run(), version_tuple() (+18 more)

### Community 13 - "Community 13"
Cohesion: 0.07
Nodes (42): format_data_size(), get_terminal_width(), bench_calculate_bandwidth(), bench_calculate_bandwidth_zero_elapsed(), bench_calculate_distance(), bench_format_data_size(), bench_progress_update_cost(), bar_width_for_terminal() (+34 more)

### Community 14 - "Community 14"
Cohesion: 0.12
Nodes (53): Args, cli(), test_cli_help(), test_cli_version(), test_combined_flags(), test_csv_flag_parsing(), test_error_output_format(), test_exit_code_on_error() (+45 more)

### Community 15 - "Community 15"
Cohesion: 0.07
Nodes (30): ClientLocation, compute_ci_95(), compute_cv(), CsvOutput, DefaultStats, deserialize_distance(), PhaseResult, PhaseState (+22 more)

### Community 16 - "Community 16"
Cohesion: 0.05
Nodes (28): bar_chart(), determine_stream_count(), format_data_size_tabular(), format_duration_tabular(), format_speed_tabular(), get_terminal_width_bounded(), test_bar_chart_empty_val(), test_bar_chart_full() (+20 more)

### Community 17 - "Community 17"
Cohesion: 0.1
Nodes (31): defaults_enable_both_phases_and_use_normal_streaming(), rejects_invalid_source_address_before_network_activity(), rejects_timeout_outside_supported_range_before_network_activity(), run_speed_test(), TestOptions, default_early_exit(), dry_run_orch(), EarlyExitFlags (+23 more)

### Community 18 - "Community 18"
Cohesion: 0.08
Nodes (24): current_level(), debug(), error(), format_json_entry(), is_verbose(), Level, log(), test_current_level_returns_info_by_default() (+16 more)

### Community 19 - "Community 19"
Cohesion: 0.1
Nodes (33): build(), build_profile_targets(), build_targets(), FileEstimate, format_targets(), format_time_estimate(), show(), Target (+25 more)

### Community 20 - "Community 20"
Cohesion: 0.09
Nodes (7): LocalCiHookTests, Exercise the pre-push candidate boundary using disposable Git repositories., WorkflowTriggerPolicyTests, resolve(), test_resolve_theme_default(), test_validate_invalid(), Theme

### Community 21 - "Community 21"
Cohesion: 0.16
Nodes (12): DefaultLatencyMonitor, IpDiscoverer, LatencyMonitor, make_client(), ServerFetcher, ServerPinger, ServerSelector, ServerService (+4 more)

### Community 22 - "Community 22"
Cohesion: 0.21
Nodes (4): estimates(), main(), regressions(), BenchmarkTests

### Community 23 - "Community 23"
Cohesion: 0.27
Nodes (6): ReleaseValidationEvidenceTests, main(), run(), validate_run(), validation_run_id(), verify()

### Community 24 - "Community 24"
Cohesion: 0.2
Nodes (5): ConfigProvider, File, NetworkConfig, ServerSelection, TestSelection

### Community 25 - "Community 25"
Cohesion: 0.28
Nodes (3): is_valid_ipv4(), is_valid_ipv6(), validate_ip_address()

### Community 26 - "Community 26"
Cohesion: 0.22
Nodes (5): ConfigSource, NetworkSource, OutputSource, ServerSource, TestSource

### Community 27 - "Community 27"
Cohesion: 0.67
Nodes (1): NetspeedCli

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

### Community 33 - "Community 33"
Cohesion: 1.0
Nodes (0):

### Community 34 - "Community 34"
Cohesion: 1.0
Nodes (0):

## Knowledge Gaps
- **60 isolated node(s):** `TestServer`, `TestServersWrapper`, `TestServerConfig`, `Exercise the pre-push candidate boundary using disposable Git repositories.`, `TestMetrics` (+55 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 28`** (2 nodes): `speedtest.rs`, `run_all_phases()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 29`** (1 nodes): `commitlint.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 30`** (1 nodes): `_netspeed-cli.ps1`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 32`** (1 nodes): `lib.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 33`** (1 nodes): `measurement.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 34`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `no_color()` connect `Community 3` to `Community 2`, `Community 4`, `Community 5`, `Community 6`, `Community 7`, `Community 9`, `Community 11`, `Community 13`, `Community 17`, `Community 20`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Why does `grade_overall()` connect `Community 6` to `Community 3`, `Community 7`?**
  _High betweenness centrality (0.041) - this node is a cross-community bridge._
- **Why does `validate_run()` connect `Community 23` to `Community 8`?**
  _High betweenness centrality (0.025) - this node is a cross-community bridge._
- **Are the 45 inferred relationships involving `Output` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Output` has 45 INFERRED edges - model-reasoned connections that need verification._
- **Are the 48 inferred relationships involving `Args` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Args` has 48 INFERRED edges - model-reasoned connections that need verification._
- **Are the 41 inferred relationships involving `no_color()` (e.g. with `print_error()` and `print_suggestion()`) actually correct?**
  _`no_color()` has 41 INFERRED edges - model-reasoned connections that need verification._
- **What connects `TestServer`, `TestServersWrapper`, `TestServerConfig` to the rest of the system?**
  _60 weakly-connected nodes found - possible documentation gaps or missing edges._
