//! Core performance benchmarks for netspeed-cli.
//!
//! Benchmarks critical pure functions in:
//! - Bandwidth calculation (`bandwidth_loop::calculate_bandwidth`)
//! - Distance calculation (`servers::calculate_distance`)
//! - Formatting utilities (`formatter::formatting::format_distance`, `format_data_size`)
//! - IP validation (`common::is_valid_ipv4`)
//! - URL construction (`download::build_test_url`, `upload::build_upload_unit`)

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use netspeed_cli::bandwidth_loop::calculate_bandwidth;
use netspeed_cli::common;
use netspeed_cli::formatter::formatting::{format_data_size, format_distance};

mod bandwidth {
    use super::*;

    pub fn bench_calculate_bandwidth(c: &mut Criterion) {
        let mut group = c.benchmark_group("bandwidth/calculate_bandwidth");

        for (bytes, elapsed) in [
            (1_000_000u64, 0.1f64),
            (10_000_000u64, 1.0f64),
            (100_000_000u64, 10.0f64),
            (1_000_000_000u64, 100.0f64),
        ] {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{bytes}B/{elapsed}s")),
                &(bytes, elapsed),
                |b, &(bytes, elapsed)| {
                    b.iter(|| {
                        calculate_bandwidth(
                            std::hint::black_box(bytes),
                            std::hint::black_box(elapsed),
                        )
                    });
                },
            );
        }

        group.finish();
    }

    pub fn bench_calculate_bandwidth_zero_elapsed(c: &mut Criterion) {
        c.bench_function("bandwidth/zero_elapsed", |b| {
            b.iter(|| {
                calculate_bandwidth(
                    std::hint::black_box(1_000_000u64),
                    std::hint::black_box(0.0f64),
                )
            });
        });
    }
}

mod distance {
    use super::*;

    pub fn bench_calculate_distance(c: &mut Criterion) {
        let mut group = c.benchmark_group("distance/calculate_distance");

        let routes = [
            ("NYC-LA", 40.7128f64, -74.0060f64, 34.0522f64, -118.2437f64),
            (
                "NYC-London",
                40.7128f64,
                -74.0060f64,
                51.5074f64,
                -0.1278f64,
            ),
            (
                "Tokyo-Sydney",
                35.6762f64,
                139.6503f64,
                -33.8688f64,
                151.2093f64,
            ),
            (
                "Same location",
                40.7128f64,
                -74.0060f64,
                40.7128f64,
                -74.0060f64,
            ),
        ];

        for (name, lat1, lon1, lat2, lon2) in routes {
            group.bench_with_input(
                BenchmarkId::from_parameter(name),
                &(lat1, lon1, lat2, lon2),
                |b, &(lat1, lon1, lat2, lon2)| {
                    b.iter(|| {
                        netspeed_cli::servers::calculate_distance(
                            std::hint::black_box(lat1),
                            std::hint::black_box(lon1),
                            std::hint::black_box(lat2),
                            std::hint::black_box(lon2),
                        )
                    });
                },
            );
        }

        group.finish();
    }
}

mod formatting {
    use super::*;

    pub fn bench_format_distance(c: &mut Criterion) {
        let mut group = c.benchmark_group("formatting/format_distance");

        for (value, label) in [
            (12.5f64, "short"),
            (99.9f64, "boundary"),
            (150.5f64, "medium"),
            (5570.0f64, "long"),
        ] {
            group.bench_with_input(BenchmarkId::from_parameter(label), &value, |b, &value| {
                b.iter(|| format_distance(std::hint::black_box(value)));
            });
        }

        group.finish();
    }

    pub fn bench_format_data_size(c: &mut Criterion) {
        let mut group = c.benchmark_group("formatting/format_data_size");

        for (bytes, label) in [
            (512u64, "bytes"),
            (500 * 1024, "kilobytes"),
            (10 * 1024 * 1024, "megabytes"),
            (4 * 1024 * 1024 * 1024, "gigabytes"),
        ] {
            group.bench_with_input(BenchmarkId::from_parameter(label), &bytes, |b, &bytes| {
                b.iter(|| format_data_size(std::hint::black_box(bytes)));
            });
        }

        group.finish();
    }
}

mod validation {
    use super::*;

    pub fn bench_is_valid_ipv4(c: &mut Criterion) {
        let mut group = c.benchmark_group("validation/is_valid_ipv4");

        for (ip, label) in [
            ("192.168.1.1", "valid"),
            ("10.0.0.1", "valid_private"),
            ("255.255.255.255", "broadcast"),
            ("999.999.999.999", "invalid_octets"),
            ("1.2.3", "too_few"),
            ("1.2.3.4.5", "too_many"),
            ("abc", "not_ip"),
        ] {
            group.bench_with_input(BenchmarkId::from_parameter(label), &ip, |b, &ip| {
                b.iter(|| common::is_valid_ipv4(std::hint::black_box(ip)));
            });
        }

        group.finish();
    }
}

mod url_construction {
    use super::*;
    use netspeed_cli::download::{build_test_url, extract_base_url};
    use netspeed_cli::upload::build_upload_url;

    pub fn bench_build_test_url(c: &mut Criterion) {
        let mut group = c.benchmark_group("url/build_test_url");

        let base = "http://server.example.com/speedtest/upload.php";

        for index in 0..4 {
            group.bench_with_input(
                BenchmarkId::from_parameter(format!("file_{index}")),
                &(base, index),
                |b, &(base, index)| {
                    b.iter(|| {
                        build_test_url(std::hint::black_box(base), std::hint::black_box(index))
                    });
                },
            );
        }

        group.finish();
    }

    pub fn bench_build_upload_url(c: &mut Criterion) {
        c.bench_function("url/build_upload_url", |b| {
            b.iter(|| {
                build_upload_url(std::hint::black_box("http://server.example.com/speedtest"))
            });
        });
    }

    pub fn bench_extract_base_url(c: &mut Criterion) {
        c.bench_function("url/extract_base_url", |b| {
            b.iter(|| {
                extract_base_url(std::hint::black_box(
                    "http://server.example.com/speedtest/upload.php",
                ))
            });
        });
    }
}

criterion_group!(
    benches,
    bandwidth::bench_calculate_bandwidth,
    bandwidth::bench_calculate_bandwidth_zero_elapsed,
    distance::bench_calculate_distance,
    formatting::bench_format_distance,
    formatting::bench_format_data_size,
    validation::bench_is_valid_ipv4,
    url_construction::bench_build_test_url,
    url_construction::bench_build_upload_url,
    url_construction::bench_extract_base_url,
);
criterion_main!(benches);
