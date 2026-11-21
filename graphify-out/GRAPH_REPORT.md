# Graph Report - /Users/alexey.ivanov/Developer/netspeed-cli  (2026-11-21)

## Corpus Check
- 77 files · ~373,545 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2024 nodes · 4979 edges · 33 communities detected
- Extraction: 70% EXTRACTED · 30% INFERRED · 0% AMBIGUOUS · INFERRED: 1484 edges (avg confidence: 0.8)
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
- `grade_bufferbloat()` --calls--> `profile_interpretation()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/grades.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/mod.rs
- `format_jitter_tabular()` --calls--> `build_latency_section()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/common.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/sections.rs
- `format_loss_tabular()` --calls--> `build_latency_section()`  [INFERRED]
  /Users/alexey.ivanov/Developer/netspeed-cli/src/common.rs → /Users/alexey.ivanov/Developer/netspeed-cli/src/formatter/sections.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.02
Nodes (181): test_settings_from_config_default_user_agent(), test_settings_from_config_retry_enabled_by_default(), test_settings_from_config_timeout(), test_settings_from_config_with_ca_cert(), test_settings_from_config_with_pinning(), test_settings_from_config_with_source_ip(), test_settings_from_config_with_tls_version(), test_config_from_default_source() (+173 more)

### Community 1 - "Community 1"
Cohesion: 0.02
Nodes (90): estimates(), main(), regressions(), format_data_size(), bench_build_test_url(), bench_build_upload_url(), bench_calculate_bandwidth(), bench_calculate_bandwidth_zero_elapsed() (+82 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (112): defaults_enable_both_phases_and_use_normal_streaming(), rejects_invalid_source_address_before_network_activity(), rejects_timeout_outside_supported_range_before_network_activity(), run_speed_test(), TestOptions, is_valid_ipv4(), Error, ErrorCategory (+104 more)

### Community 3 - "Community 3"
Cohesion: 0.03
Nodes (135): format_distance(), format_latency_tabular(), ascii_report(), ascii_report_preserves_metrics_and_uses_plain_terminal_characters(), boxed_header(), connection_info(), data_summary(), format_rate() (+127 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (49): format_grade_line(), grade_badge(), grade_bufferbloat(), grade_download(), grade_jitter(), grade_overall(), grade_ping(), grade_upload() (+41 more)

### Community 5 - "Community 5"
Cohesion: 0.04
Nodes (87): AbortStreams, BandwidthResult, cancelling_measurement_aborts_streams(), concurrent_sampling_claims_one_interval(), finish_does_not_turn_a_short_tail_into_a_peak_sample(), interval_samples_detect_stalls_and_recovery(), LoopState, make_tracker() (+79 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (50): test_machine_error_identity_csv(), Config, default_early_exit(), dry_run_orch(), EarlyExitFlags, library_result_phase_does_not_save_history(), library_run_propagates_measurement_errors(), library_run_returns_report_and_disables_verbose_mode() (+42 more)

### Community 7 - "Community 7"
Cohesion: 0.07
Nodes (76): main(), Args, cli(), test_cli_help(), test_cli_version(), test_combined_flags(), test_csv_flag_parsing(), test_error_output_format() (+68 more)

### Community 8 - "Community 8"
Cohesion: 0.04
Nodes (64): bufferbloat_colorized(), bufferbloat_grade(), colorize_rating(), connection_rating(), degradation_str(), format_overall_rating(), format_speed_colored(), format_speed_plain() (+56 more)

### Community 9 - "Community 9"
Cohesion: 0.08
Nodes (65): backup_path(), clear(), clear_history_from_path(), concurrent_saves_preserve_all_entries(), corrupt_path(), Entry, get_history_path(), load_entries() (+57 more)

### Community 10 - "Community 10"
Cohesion: 0.05
Nodes (40): is_config_error(), is_list_sentinel(), is_network_error(), machine_error_format(), machine_error_identity(), MachineErrorBody, MachineErrorOutput, print_error() (+32 more)

### Community 11 - "Community 11"
Cohesion: 0.06
Nodes (62): all_categories(), compute_all_statuses(), compute_scenario_status(), format_scenario_grid(), headroom_level(), HeadroomLevel, print_scenario_grid(), render_capacity_bar() (+54 more)

### Community 12 - "Community 12"
Cohesion: 0.07
Nodes (27): minimum_tls_12_negotiates_both_versions(), git(), open_pr(), prepare(), push_pr(), release_checks(), run(), version_tuple() (+19 more)

### Community 13 - "Community 13"
Cohesion: 0.06
Nodes (37): calculate_bandwidth(), build_test_url(), DownloadConfig, extract_base_url(), hidden_tracker(), invalid_download_limits_fail_without_io(), local_server(), run() (+29 more)

### Community 14 - "Community 14"
Cohesion: 0.05
Nodes (30): bar_chart(), determine_stream_count(), format_data_size_tabular(), format_duration_tabular(), format_jitter_tabular(), format_loss_tabular(), format_speed_tabular(), get_terminal_width_bounded() (+22 more)

### Community 15 - "Community 15"
Cohesion: 0.08
Nodes (35): get_terminal_width(), bar_width_for_terminal(), create_spinner(), finish_ok(), format_progress_rate(), progress_prefix(), progress_style(), ProgressLayout (+27 more)

### Community 16 - "Community 16"
Cohesion: 0.07
Nodes (29): ClientLocation, compute_ci_95(), compute_cv(), CsvOutput, DefaultStats, deserialize_distance(), PhaseResult, PhaseState (+21 more)

### Community 17 - "Community 17"
Cohesion: 0.06
Nodes (26): test_ping_test_failure_mocked(), test_ping_test_mocked(), LocalDiscovery, FixtureServerService, calculate_distance(), fetch(), ping_test(), select_best_server() (+18 more)

### Community 18 - "Community 18"
Cohesion: 0.08
Nodes (24): current_level(), debug(), error(), format_json_entry(), is_verbose(), Level, log(), test_current_level_returns_info_by_default() (+16 more)

### Community 19 - "Community 19"
Cohesion: 0.1
Nodes (33): build(), build_profile_targets(), build_targets(), FileEstimate, format_targets(), format_time_estimate(), show(), Target (+25 more)

### Community 20 - "Community 20"
Cohesion: 0.07
Nodes (11): OutputFormatType, ShellType, test_validate_ca_cert_path_is_directory(), test_validate_ca_cert_path_not_found(), test_validate_ca_cert_path_valid(), test_validate_timeout_too_large(), test_validate_timeout_zero(), validate_ca_cert_path() (+3 more)

### Community 21 - "Community 21"
Cohesion: 0.23
Nodes (3): LocalCiHookTests, Exercise the pre-push candidate boundary using disposable Git repositories., WorkflowTriggerPolicyTests

### Community 22 - "Community 22"
Cohesion: 0.27
Nodes (6): ReleaseValidationEvidenceTests, main(), run(), validate_run(), validation_run_id(), verify()

### Community 23 - "Community 23"
Cohesion: 0.2
Nodes (5): ConfigProvider, File, NetworkConfig, ServerSelection, TestSelection

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
- **60 isolated node(s):** `TestServer`, `TestServersWrapper`, `TestServerConfig`, `Exercise the pre-push candidate boundary using disposable Git repositories.`, `TestMetrics` (+55 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 26`** (2 nodes): `speedtest.rs`, `run_all_phases()`
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

- **Why does `no_color()` connect `Community 3` to `Community 1`, `Community 4`, `Community 5`, `Community 6`, `Community 7`, `Community 10`, `Community 11`, `Community 15`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Why does `grade_overall()` connect `Community 4` to `Community 3`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **Why does `degradation_str()` connect `Community 8` to `Community 3`, `Community 7`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **Are the 45 inferred relationships involving `Output` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Output` has 45 INFERRED edges - model-reasoned connections that need verification._
- **Are the 48 inferred relationships involving `Args` (e.g. with `test_ca_cert_in_help()` and `test_pin_certs_in_help()`) actually correct?**
  _`Args` has 48 INFERRED edges - model-reasoned connections that need verification._
- **Are the 41 inferred relationships involving `no_color()` (e.g. with `print_error()` and `print_suggestion()`) actually correct?**
  _`no_color()` has 41 INFERRED edges - model-reasoned connections that need verification._
- **What connects `TestServer`, `TestServersWrapper`, `TestServerConfig` to the rest of the system?**
  _60 weakly-connected nodes found - possible documentation gaps or missing edges._
