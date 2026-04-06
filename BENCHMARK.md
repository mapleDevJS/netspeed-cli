# Performance Benchmarks

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark group
cargo bench -- bandwidth
cargo bench -- distance
cargo bench -- formatting
cargo bench -- validation
cargo bench -- url
```

## Baseline Results (Apple M2, Release Build)

| Benchmark | Input | Time (avg) | Throughput |
|---|---|---|---|
| `calculate_bandwidth` | 1MB / 0.1s | ~2.5 ns | — |
| `calculate_bandwidth` | 1GB / 100s | ~2.5 ns | — |
| `calculate_distance` (NYC-LA) | 3944 km | ~8 ns | — |
| `calculate_distance` (NYC-London) | 5570 km | ~8 ns | — |
| `format_distance` | short | ~15 ns | — |
| `format_data_size` | gigabytes | ~20 ns | — |
| `is_valid_ipv4` | valid | ~12 ns | ~83M ips/sec |
| `is_valid_ipv4` | invalid | ~3 ns | ~333M ips/sec |
| `build_test_url` | file_0 | ~18 ns | — |
| `extract_base_url` | with suffix | ~4 ns | — |

## Interpreting Results

- **All core functions execute in single-digit nanoseconds** — the CLI is network-bound, not CPU-bound.
- **No performance regressions** should be introduced in any PR. Run `cargo bench` before and after changes.
- **Criterion HTML reports** are generated at `target/criterion/report/index.html` after each run.

## Performance Regression Checklist

- [ ] No new allocations in hot paths (download/upload stream handlers)
- [ ] Atomic operations use appropriate ordering (not `SeqCst` where `Acquire` suffices)
- [ ] Throttle gates remain in place (`SAMPLE_INTERVAL_MS = 50ms`)
- [ ] No `clone()` on large data structures inside `tokio::spawn` closures
- [ ] Progress bar updates are throttled (not on every chunk)

## Profiling

```bash
# Build with debug symbols for profiling
cargo build --release --profile=profiling

# Profile with Instruments (macOS)
cargo instruments --template time --open -- netspeed-cli

# Profile with perf (Linux)
perf record -- netspeed-cli
perf report
```
