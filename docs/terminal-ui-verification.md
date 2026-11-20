# Terminal UI implementation and verification

## Current implementation

### Architecture decision

The selected approach is a hybrid: one scoped inline live renderer followed by
a persistent report, built with the existing console and theme infrastructure.
Measurement callbacks publish only a bounded presentation snapshot; result and
machine-output writers remain separate. No alternate screen or input loop is needed.

| Approach | Suitability for this CLI |
| --- | --- |
| Modernize existing progress bars | Small dependency cost, but competing bars and report messages need centralized ownership. |
| Ratatui with a terminal backend | Structured widgets and test buffers are attractive; adds a renderer/backend dependency and lifecycle integration for this small observational display. No Ratatui dependency was selected or MSRV claim made for it. |
| Hybrid using existing console | Selected: bounded live frames, pure layout tests, ordinary scrollback, existing cross-platform support, and no new locked package. |

Design references reviewed were [hyperfine](https://github.com/sharkdp/hyperfine)
for concise completed measurement reports,
[fastfetch](https://github.com/fastfetch-cli/fastfetch) for readable contextual fields,
[oha](https://github.com/hatoo/oha) for live measurement feedback,
[btop](https://github.com/aristocratos/btop) and
[bottom](https://github.com/ClementTsang/bottom) for grouping metrics and histories,
and [Lip Gloss](https://github.com/charmbracelet/lipgloss) for consistent styling.
These are design references, not code or branding dependencies. Fullscreen monitor
interaction is unnecessary for an automatic, finite test.

The CLI phase context owns a scoped inline dashboard. A dedicated worker refreshes
stderr at 10 Hz and clears its temporary rows before the persistent report. The
dashboard displays phase, elapsed time, measured rate, collected sample history,
transferred data, and available connection context. Terminal dimensions are checked
on refresh and each line reserves the final column against wrapping.

Four density modes use both dimensions: minimal below 50 columns or 5 rows,
compact below 75 columns or 7 rows, standard below 110 columns or 9 rows,
and expanded otherwise. Charts retain at most 16, 32, or 48 glyphs in the latter
three modes. Completed download results remain visible during upload where space
permits. Explicit run and transfer states distinguish completed, skipped, failed,
interrupted, pending, and unavailable values, including successful zero throughput.

The implementation reuses console, the existing rate formatter, and existing
themes. It reuses libc, already present in the lockfile, as a Unix runtime dependency for a separate nonblocking terminal descriptor. It adds no new package to the dependency graph. Sample publication uses a nonblocking lock and
retains at most 48 display samples. Statistical sample collection remains in the
measurement state. The chart identifies its horizontal sequence as samples. Publications carry a sequence number and timestamp. A dropped publication leaves a gap, and rendering works from a cloned snapshot outside the publication lock.

Discovery, transfer, and IP warnings are buffered during live rendering and emitted after
cleanup. The buffer retains 32 messages and reports the number of further messages.
The legacy transfer renderer has a drop guard that stops and joins its worker on
success, failure, and cancellation. Transfer duration is captured before cleanup.

Default detailed output uses the dashboard report. Its pure builder supports width
and height inputs and writes the completed report in one stderr operation. Speeds,
peaks, skipped phases, measured latency, loss, load latency, bufferbloat, variation,
completion timestamp, server context, and interpretation appear when available.
Short histories produce at most one chart glyph per collected sample. Historical
phase metadata is authoritative for skipped results. Jitter and packet loss remain
visible when ping is unavailable. The terse minimal format converts rates for
--bytes. The default detailed --minimal report converts display glyphs and server
text to ASCII, using ? for unsupported characters. Compact sections omit the
former fixed-scale gauge and retain text ratings.

## Audit fixes

- Moved terminal rendering away from bandwidth sampling and restored ordered sample bookkeeping.
- Fixed worker lifetime on errors and cancellation and excluded shutdown time from reported transfer duration.
- Reused byte-aware formatting for detailed speed and peak output and corrected byte conversion in the terse minimal format.
- Corrected live-mode selection for explicit-format precedence and static minimal output.
- Removed the fixed-scale gauge from compact speed sections.
- Made the default detailed ASCII report use plain display characters.
- Preserved historical skipped-phase metadata and independently available health metrics.
- Reserved terminal width in report output and added complete report width checks.
- Corrected A+ header wrapping found in actual 40-column PTY capture inspection; grade-aware allocation is tested across every grade and theme with equal-width border assertions.
- Restored health diagnostics after consolidating detailed and dashboard output.
- Replaced four existing unsupported let-chain expressions to make the locked build compile on Rust 1.86.

## Implementation files

- The scoped renderer and terminal backpressure handling live in [live.rs](../src/live.rs), coordinated through [phases.rs](../src/phases.rs) and [orchestrator.rs](../src/orchestrator.rs).
- Measurement publication and cleanup are implemented in [progress.rs](../src/progress.rs), [bandwidth_loop.rs](../src/bandwidth_loop.rs), and [task_runner.rs](../src/task_runner.rs).
- Report construction, interpretation, ASCII conversion, and machine-output writer tests are in [formatter](../src/formatter/).
- Warning routing uses [services.rs](../src/services.rs) and [servers.rs](../src/servers.rs). Interruption uses [interrupt.rs](../src/interrupt.rs) and [main.rs](../src/main.rs).
- Deterministic terminal lifecycle regressions are in [pty_lifecycle_test.rs](../tests/pty_lifecycle_test.rs), with progress microbenchmarks in [core_benchmarks.rs](../benches/core_benchmarks.rs).

## Verification performed

Verification ran on macOS ARM64 and in a disposable Linux ARM64 container. Commands using local HTTP fixtures required
loopback access outside the restricted sandbox.

- Formatting, strict all-target Clippy, and diff whitespace checks passed.
- The latest macOS full cargo test passed, with 975 library tests passing and 17 ignored, plus integration suites and 79 passing doc tests. The earlier Linux full run passed 969 library tests, its integration suites, and 79 doc tests.
- All seven macOS PTY tests passed, including discovery HTTP failure cleanup, short height, resizing, actual-binary interruption with stdout piped, explicit-format precedence over legacy flags, and nine static-mode configurations. The earlier Linux lifecycle run passed its four-case PTY suite.
- Complete report width assertions passed at 40, 60, 80, and 120 columns and heights 4, 8, and 24, across dark, light, high-contrast, and monochrome themes.
- Explicit socket suite passed 5 socket regressions, 2 mock-network tests, 5 upload/fetch tests, and 3 end-to-end tests.
- Locked Rust 1.86 check and optimized release build passed.
- Rustdoc with warnings denied and 31 Python tooling tests passed.
- Local packaged-source verification passed with `cargo package --locked --offline --allow-dirty`, including a normal build without `DOCS_RS`. The archive contained 133 files (about 281 KiB compressed), including the new renderer, interruption module, and PTY tests. `--allow-dirty` preserved the uncommitted candidate; no publication was performed.
- Cargo-deny passed advisories, bans, licenses, and sources using the cached database, with dependency-duplication warnings.
- A subsequent `cargo deny --locked check` with advisory fetching enabled also passed all four checks. The existing duplicate-dependency warnings remain warnings, not failed policy checks.

An actual 80-column PTY capture was inspected. It showed live transfer rates and
sample glyphs followed by the persistent report. The capture used a local HTTP
fixture, so its throughput values describe loopback behavior. This provides text
inspection evidence. Native terminal screenshots remain unverified. Linux checks are recorded below.

## Design evidence

Two deterministic render variants were inspected with identical 80 Mb/s, stall,
recovery, transferred-byte, and Unicode server fixtures.

The minimal view uses two rows.

```text
  Download  1.0s
  80.00 Mb/s
```

The diagnostic view reveals the collected samples and connection context.

```text
  Download  1.0s
  80.00 Mb/s
  █▁ ▅ samples
  976.6 KB transferred
  Download measuring   Upload pending
  東京 Test Server
```

The diagnostic view is selected where dimensions permit it. The minimal view is
the small-height fallback. These buffer variants were inspected during completion
work after the initial renderer existed; prototype selection was therefore a
retrospective comparison rather than the requested pre-implementation gate.

## Further contract and failure verification

The interruption helper exposes a named Interrupted error. The supported
result-returning embedding facade retains its existing interface. It is handled by the
existing CLI interruption path and preserves exit status 130.

The actual release binary was interrupted while a local proxy held discovery
traffic. It returned 130, emitted Interrupted. on stderr, left stdout empty, and
produced no final report. The proxy prevented external speed-test traffic. This check is now an automated
PTY regression. It also exercises an explicit dashboard format overriding the
legacy --json flag, with stdout piped and stderr attached to a terminal. A second
actual-binary regression covers minimal, quiet, no-live-ui, JSON, JSONL, CSV, dumb
terminals, CI, and reduced motion and asserts that live controls are absent.

The serialized type definitions match HEAD. JSON and JSONL now share an internal
writer function, and CSV has an internal writer function, while the existing
public entry points keep their signatures and stdout behavior. Fixed-value
contract tests assert every JSON field and value, JSONL single-line output, and
exact CSV header and row bytes, including the existing header behavior. Formatter
and integration tests passed.

The interpretation tests cover every profile, observed packet loss, degraded load
latency, invalid and partial results, deterministic repeated output, and unchanged
serialized input values. The conclusions use existing grades and prioritize the
weakest available observation, with profile priorities breaking ties.

A deterministic bandwidth replay compared hidden and live publication paths. Both
produced the same statistical samples, peak, and byte count, including stalls.
Publication under a deliberately held presentation lock returned without blocking.
The display history remained bounded at 48 entries. Rendering holds no model lock
during terminal I/O. Timestamp gaps above two sampling intervals appear as chart
gaps, and width reduction preserves the newest chart values.

## Performance evidence and platform limits

The final four-density renderer built 10,000 diagnostic frames in 25.31 ms
(2.53 microseconds/frame) and published 100,000 updates in 6.92 ms
(69 ns/publication). This includes the completed-transfer overview. The previous
release-mode fixture built 10,000 diagnostic frames in 10.47 ms and published
100,000 samples in 4.11 ms on macOS ARM64. A noisier run took 27.80 ms for the
frames. These correspond to roughly 1.05–2.78 microseconds per frame and 41–45 ns
per publication in the measured runs. At a 10 Hz refresh ceiling this frame-build
cost is small, while terminal write costs depend on the terminal.

The standalone deterministic workload used 0.01 seconds user CPU, 0.00 seconds
system CPU, and 4,177,920 bytes maximum resident memory according to macOS time.
These figures include the test harness. They do not isolate renderer allocation
counts or native terminal I/O costs. The hot publication path reuses its allocated
history, and terminal writes occur after releasing the model lock.

A macOS leaks run of the standalone release renderer workload reported 190 live
allocations totaling 13 KB at exit and zero detected leaks. The tool restricted
object inspection because the executable lacked debugging entitlements. This is
whole-process allocation evidence rather than per-frame allocation accounting.

Live terminal-write failure stops drawing, attempts temporary-row cleanup, and
queues a static fallback notice while measurements continue. Output initialization
failure uses the existing fallback path. All work is local.

On Unix, live output opens the actual stderr terminal using a separate nonblocking
descriptor. The original stderr descriptor retains its flags. A full-PTY-buffer
test passed, returning WouldBlock while the original flags remained unchanged.
The renderer then stops live drawing and queues its fallback notice. If reopening
the terminal fails, the CLI uses static output without starting legacy spinners
or a transfer rendering worker.

Linux ARM64 verification passed in the existing Docker environment. The final
dependency graph passed the Rust 1.86 production check, and stable passed the full
library, integration, PTY, and doc-test suites. The explicit socket suites also
passed. The first test build attempted Rust 1.86 and found that the existing
wiremock 0.6.5 development dependency requires newer let-chain syntax. Production
checks therefore use 1.86 and tests use stable, matching the repository CI split.
A parallel linker run exceeded the Docker VM's 2 GB memory limit; a single-job
retry passed. A verification-copy omission of the checked-in completion files was
corrected before the successful full run.

The unused crossterm dependency was removed after source inspection confirmed it
had no callers. The console renderer and existing compatibility APIs supply the
remaining terminal functionality.

Native Windows checks, screen-reader interaction, and terminal screenshots remain
unverified. The [native acceptance runbook](terminal-ui-native-acceptance.md)
provides commands, candidate recording, pass criteria, and an unexecuted results
matrix for those checks. No Windows execution environment is available here. Windows retains
the console backend, and its blocked-console behavior requires native acceptance. A permanently blocked
original stderr stream can still block ordinary final-report or diagnostic writes,
as with other synchronous CLI output. The Unix live worker itself uses nonblocking
writes and releases the terminal during cleanup. The tested workload establishes
deterministic sampling equivalence and bounded publication, but cannot establish
zero impact on arbitrary network tests or terminals.

Windows cross-compilation was also not executed: the installed Rust target is
macOS ARM64 only, and this host has neither a Windows compiler/SDK nor a MinGW
cross compiler. Installing a Rust target alone would not supply the native tools
needed by the TLS dependencies. This is a tooling limitation, not a failed Windows
build or a native Windows acceptance result.


The final checks emitted an existing manifest warning about homepage duplicating
repository and sandbox-related Xcode cache diagnostics. The checks completed
successfully. The latest output-selection, ASCII, partial-result, lifecycle, density-mode, and actual-binary
regressions were verified on macOS after the Linux core verification described
above.
