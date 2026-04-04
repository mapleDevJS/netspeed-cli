use criterion::{black_box, criterion_group, criterion_main, Criterion};
use netspeed_cli::servers::{calculate_distance, calculate_distances, select_best_server};
use netspeed_cli::types::Server;
use netspeed_cli::utils::{calculate_bps, format_speed};

fn bench_distance_calculation(c: &mut Criterion) {
    c.bench_function("distance_nyc_to_la", |b| {
        b.iter(|| {
            calculate_distance(
                black_box(40.7128),
                black_box(-74.0060),
                black_box(34.0522),
                black_box(-118.2437),
            )
        });
    });

    c.bench_function("distance_london_to_paris", |b| {
        b.iter(|| {
            calculate_distance(
                black_box(51.5074),
                black_box(-0.1278),
                black_box(48.8566),
                black_box(2.3522),
            )
        });
    });

    c.bench_function("distance_same_point", |b| {
        b.iter(|| {
            calculate_distance(
                black_box(40.0),
                black_box(-74.0),
                black_box(40.0),
                black_box(-74.0),
            )
        });
    });
}

fn bench_bps_calculation(c: &mut Criterion) {
    c.bench_function("calculate_bps", |b| {
        b.iter(|| calculate_bps(black_box(50_000_000), black_box(5.0)));
    });
}

fn bench_format_speed(c: &mut Criterion) {
    c.bench_function("format_speed_bits", |b| {
        b.iter(|| format_speed(black_box(150_000_000.0), false));
    });

    c.bench_function("format_speed_bytes", |b| {
        b.iter(|| format_speed(black_box(150_000_000.0), true));
    });
}

fn bench_server_selection(c: &mut Criterion) {
    // Create 100 servers for realistic benchmark
    let servers: Vec<Server> = (0..100)
        .map(|i| Server {
            id: format!("{}", i),
            url: format!("http://server{}.test.com/", i),
            name: format!("Server {}", i),
            sponsor: format!("ISP {}", i),
            country: "US".to_string(),
            lat: 30.0 + (i as f64 * 0.1) % 20.0,
            lon: -120.0 + (i as f64 * 0.1) % 60.0,
            distance: (i as f64) * 50.0,
            latency: 10.0 + (i as f64 * 0.5) % 100.0,
        })
        .collect();

    c.bench_function("select_best_server_100", |b| {
        b.iter(|| select_best_server(black_box(&servers)));
    });
}

fn bench_distance_sorting(c: &mut Criterion) {
    let servers: Vec<Server> = (0..50)
        .map(|i| Server {
            id: format!("{}", i),
            url: format!("http://server{}.test.com/", i),
            name: format!("Server {}", i),
            sponsor: format!("ISP {}", i),
            country: "US".to_string(),
            lat: 30.0 + (i as f64 * 0.1) % 20.0,
            lon: -120.0 + (i as f64 * 0.1) % 60.0,
            distance: 0.0,
            latency: 0.0,
        })
        .collect();

    c.bench_function("calculate_distances_50", |b| {
        b.iter(|| {
            let mut s = servers.clone();
            calculate_distances(black_box(&mut s), black_box(40.7128), black_box(-74.0060));
        });
    });
}

criterion_group!(
    benches,
    bench_distance_calculation,
    bench_bps_calculation,
    bench_format_speed,
    bench_server_selection,
    bench_distance_sorting,
);
criterion_main!(benches);
