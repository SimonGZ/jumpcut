use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use unicode_categories::UnicodeCategories;

fn remove_problematic_unicode(text: &str) -> String {
    text.chars().filter(|x| !x.is_other_format()).collect()
}

fn remove_problematic_unicode2(text: &str) -> String {
    text.replace(|c: char| c.is_other_format(), "")
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Problematic Unicode");

    let clean_text = include_str!("108.fountain");
    let big_dirty = include_str!("108-dirty.fountain");
    let short_dirty = "Hello\u{200B}, \u{200D}\u{FEFF}World!";

    group.bench_with_input(
        BenchmarkId::new("Filter", "short dirty"),
        &short_dirty,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("Replace", "short dirty"),
        &short_dirty,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("Filter", "big dirty"),
        &big_dirty,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("Replace", "big dirty"),
        &big_dirty,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("Filter", "big clean"),
        &clean_text,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    group.bench_with_input(
        BenchmarkId::new("Replace", "big clean"),
        &clean_text,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
