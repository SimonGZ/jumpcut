use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jumpcut::parse;
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut parse_group = c.benchmark_group("Parse");
    parse_group.measurement_time(Duration::from_secs(8));

    let scd_text = include_str!("108.fountain");
    let big_fish_text = include_str!("Big-Fish.fountain");

    parse_group.bench_with_input(BenchmarkId::new("Parse", "108"), &scd_text, |b, s| {
        b.iter(|| parse(s))
    });
    parse_group.bench_with_input(
        BenchmarkId::new("Parse", "Big Fish"),
        &big_fish_text,
        |b, s| b.iter(|| parse(s)),
    );
    parse_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
