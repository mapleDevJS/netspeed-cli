# Native terminal UI acceptance

Use this checklist with the implementation and automated evidence in
[terminal-ui-verification.md](terminal-ui-verification.md). Native execution is
still pending. Automated buffer and PTY checks do not establish screen-reader
behavior, native Windows console behavior, or visual contrast.

## Record the candidate

Record the date, OS, terminal application/version, shell, Rust version, font,
theme, terminal dimensions, and whether a screen reader is running. Record the
commit and working-tree changes: this candidate includes uncommitted changes,
so a commit identifier alone does not identify the tested source.

```sh
git rev-parse HEAD
git status --short
git diff --stat
rustc --version
cargo build --release --locked
```

Save `git diff --binary` to a patch outside the repository. Keep the patch and
copies of untracked implementation files with the evidence;
`git diff` omits untracked files. Keep captures outside the repository or remove
them from the worktree after collecting them. Build on the native platform being
tested. Use stable for development-dependency tests; production compilation also
needs the declared Rust 1.86 check described in the verification record.

## Repeatable automated checks

On Unix, run the local HTTP/PTY fixtures without external speed-test traffic:

```sh
cargo test --locked --test pty_lifecycle_test -- --nocapture
```

The suite covers successful discovery and transfers, resizing, short terminals,
HTTP discovery failure, interruption, stream separation, and static modes. It is
Unix-only. A Windows run that reports zero tests for this target is not a pass
for Windows lifecycle acceptance.

For deterministic buffer and interpretation checks on every platform:

```sh
cargo test --locked --lib live::tests
cargo test --locked --lib formatter::
```

These tests cannot replace observing the native terminal. On Windows the Unix
nonblocking-descriptor test is unavailable by design.

## Native observation

Run the built executable directly in the terminal: `./target/release/netspeed-cli`
on macOS/Linux or `.\target\release\netspeed-cli.exe` in PowerShell. These commands
perform real network measurements and use the existing history behavior. Inspect
appearance and lifecycle rather than expecting deterministic speed values.

| Check | Pass criteria | Result |
| --- | --- | --- |
| Default invocation | Inline dashboard appears automatically; discovery, latency, download, and upload are clear. | NOT RUN |
| 40, 60, 80, 120 columns | Primary rate stays readable; no horizontal overflow or broken borders. | NOT RUN |
| Short window | At 4 and 8 rows, live content degrades safely and final results remain accessible in scrollback. | NOT RUN |
| Resize during transfer | Shrink and expand repeatedly; no panic, displaced diagnostic output, or loss of completed results. | NOT RUN |
| Successful completion | Temporary rows clear; exactly one final report remains; prior scrollback is retained. | NOT RUN |
| Ctrl+C | Interrupt during discovery and during transfer; prompt returns, cursor is visible, no success report appears, exit status is 130. | NOT RUN |
| Static fallback | Repeat with `--no-live-ui`; output remains readable without animated frames. | NOT RUN |
| Accessibility | Repeat with `NO_COLOR=1`, `PREFER_REDUCED_MOTION=1`, and monochrome theme; meaning does not depend on color. | NOT RUN |
| Themes | Check dark, light, high-contrast, and monochrome; values, labels, and chart remain distinguishable. | NOT RUN |
| Partial tests | `--no-upload` and `--no-download` clearly identify skipped measurements; no successful zero is substituted. | NOT RUN |
| Units | `--bytes` consistently changes live and final rate units. | NOT RUN |
| Screen reader | With live rendering disabled, phases and final results can be read in a useful order; record exact reader and settings. | NOT RUN |
| Windows console | Native Windows Terminal completes and interrupts safely; terminal state and prompt remain usable. | NOT RUN |

Check exit status immediately after interruption using `echo $?` in a POSIX shell
or `$LASTEXITCODE` in PowerShell. Use `--help` for supported theme names and flags.
Do not infer Windows safety from Unix PTY results.

For stream separation, run the native command with stdout redirected and stderr
still attached to the terminal. The live UI should remain visible. With
`--format json`, `--format jsonl`, or `--format csv`, stdout should contain only
the selected machine format and there should be no live rendering. Parse the
JSON/JSONL output with a JSON parser; compare CSV against the existing contract,
including its historical header behavior.

## Evidence and decision

For each row, record PASS, FAIL, or NOT RUN with terminal dimensions, exact command,
exit status, and a capture or observation. Save actual screenshots at representative
widths and both light and dark backgrounds. Preserve interruption and resize
observations separately from successful completion. Redact public IP or server
metadata before sharing captures.

A failure requires a reproducible report and a regression fix before acceptance.
An unavailable platform remains NOT RUN. Do not translate unobserved cases into
passes or use cross-compilation as proof of native terminal behavior. No commit,
push, package publication, or release is part of this checklist.

## Execution record: 2026-11-20

The available host was macOS 27.2.1 (build 26B131), ARM64. Kernel PTY execution
was available. Native Terminal application inspection was attempted but macOS
denied Apple events with error -1743. No native Windows runtime or usable
screen-reader automation session was available. The native observation matrix
above therefore remains NOT RUN; the following results are automated evidence.

| Executed check | Result |
| --- | --- |
| macOS PTY lifecycle suite | PASS: 7 tests, including resizing, failure cleanup, interruption, and static-mode controls. |
| Formatter suite before capture correction | PASS: 209 tests; capture inspection nevertheless exposed a narrow-header defect. |
| Corrected header regression | PASS: 1,144 combinations of 11 grades, 13 widths, 4 themes, and styled/plain branches; boxed rows must have equal widths and retain the grade. |
| Final full test suite after correction | PASS: 975 library tests, integration suites including all 7 PTY tests, and 79 doc tests; existing ignored tests remain ignored. |
| Final strict Clippy, release build, locked Rust 1.86 check | PASS. |
| Fixture transcript captures | PASS: 40x24, 60x24, 80x24, 120x24, 80x4, and 40x8; each exited 0 and retained exactly one final report. |

The first capture inspection showed `A+ · 100` wrapping the 40-column boxed
header. The renderer now reserves the actual grade width before truncating its
label and uses the same label allocation for styled and plain output. The final
captures were inspected again; the header remained on three aligned rows at every
captured size. Below 24 columns it uses a single truncated line rather than a box.

Actual corrected 40-column PTY output (`NO_COLOR=1`):

```text
  ┌───────────────────────────────────┐
  │  ◉ NETSPEED  ·  Excep…  A+ · 100  │
  └───────────────────────────────────┘
```

Local evidence is saved at
`/private/tmp/netspeed-native-acceptance-20261120T125525Z/`: raw `.ansi` streams,
plain transcripts, test/build logs, `results.json`, per-file source hashes,
`candidate.patch`, and `candidate-source.tar.gz` (including untracked source).
The fixture executable is also identified by its SHA-256 in the results. This
temporary directory is local evidence, not a committed or published artifact;
preserve it separately if needed for later acceptance. Captured throughput values
come from local HTTP fixtures and do not describe internet performance. These
transcripts establish text rendering evidence, not screenshots or contrast review.

### GUI access follow-up

A subsequent unsandboxed read-only Terminal version query succeeded (2.15), so
the original -1743 denial was not consistently reproducible. A request for window
count and visual profiles timed out with -1712. A separate read-only automation
permission preflight also stalled and was terminated. This does not establish
that automation permission is granted or that GUI inspection is available.

The task process and Terminal process belonged to UID 501, while the active
desktop belonged to UID 503. Resume GUI checks from the intended UID 501 desktop
before changing privacy settings. If macOS requests consent there, approve only
Terminal control for the requesting application. Existing consent can be reviewed
in **System Settings > Privacy & Security > Automation**, following
[Apple's instructions](https://support.apple.com/en-ie/guide/mac-help/mchl07817563/27/mac/27).
Do not assume the requesting application label; use the one actually shown by
macOS. No privacy database was edited and no global permission reset was performed.

Use a bounded read-only probe to confirm that actual application state is
accessible before opening test windows:

```sh
osascript -e 'with timeout of 10 seconds' \
  -e 'tell application "Terminal" to get {version, count of windows}' \
  -e 'end timeout'
```

A successful version-only query is insufficient. Window access and actual native
captures must succeed before any GUI acceptance row is marked PASS.
