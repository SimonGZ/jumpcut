use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::str::Lines;
use unicode_categories::UnicodeCategories;

fn lines_to_hunks(lines: Lines) -> Vec<Vec<&str>> {
    let initial: Vec<Vec<&str>> = vec![vec![]];
    lines.fold(initial, |mut acc, l: &str| match l {
        l if l.is_empty() || l.trim().is_empty() => {
            if l.len() == 2 {
                acc.last_mut()
                    .expect("There should always be at least one vec")
                    .push(l);
            } else {
                acc.push(vec![]);
            }
            acc
        }
        l => {
            acc.last_mut()
                .expect("There should always be at least on vec")
                .push(l);
            acc
        }
    })
}

fn lines_to_hunks_simple_match(lines: Lines) -> Vec<Vec<&str>> {
    let initial: Vec<Vec<&str>> = vec![vec![]];
    lines.fold(initial, |mut acc, l: &str| match l.trim() {
        "" => {
            if l.len() == 2 {
                acc.last_mut()
                    .expect("There should always be at least one vec")
                    .push(l);
            } else {
                acc.push(vec![]);
            }
            acc
        }
        l => {
            acc.last_mut()
                .expect("There should always be at least on vec")
                .push(l);
            acc
        }
    })
}
fn remove_problematic_unicode(text: &str) -> String {
    text.chars().filter(|x| !x.is_other_format()).collect()
}

fn remove_problematic_unicode2(text: &str) -> String {
    text.replace(|c: char| c.is_other_format(), "")
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut unicode_group = c.benchmark_group("Problematic Unicode");

    let clean_text = include_str!("108.fountain");
    let big_dirty = include_str!("108-dirty.fountain");
    let short_dirty = "Hello\u{200B}, \u{200D}\u{FEFF}World!";

    unicode_group.bench_with_input(
        BenchmarkId::new("Filter", "short dirty"),
        &short_dirty,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    unicode_group.bench_with_input(
        BenchmarkId::new("Replace", "short dirty"),
        &short_dirty,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    unicode_group.bench_with_input(
        BenchmarkId::new("Filter", "big dirty"),
        &big_dirty,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    unicode_group.bench_with_input(
        BenchmarkId::new("Replace", "big dirty"),
        &big_dirty,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    unicode_group.bench_with_input(
        BenchmarkId::new("Filter", "big clean"),
        &clean_text,
        |b, s| b.iter(|| remove_problematic_unicode(s)),
    );
    unicode_group.bench_with_input(
        BenchmarkId::new("Replace", "big clean"),
        &clean_text,
        |b, s| b.iter(|| remove_problematic_unicode2(s)),
    );
    unicode_group.finish();

    let mut hunk_group = c.benchmark_group("Hunks");

    let medium_text: Lines = clean_text.lines();

    hunk_group.bench_with_input(
        BenchmarkId::new("Lines to Hunks", "if pattern"),
        &medium_text,
        |b, s| b.iter(|| lines_to_hunks(s.clone())),
    );

    hunk_group.bench_with_input(
        BenchmarkId::new("Lines to Hunks", "simple pattern"),
        &medium_text,
        |b, s| b.iter(|| lines_to_hunks_simple_match(s.clone())),
    );
    hunk_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
