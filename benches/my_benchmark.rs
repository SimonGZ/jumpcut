use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lazy_static::lazy_static;
use regex::Regex;
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

fn lines_to_hunks_complete(lines: Lines) -> Vec<Vec<&str>> {
    let initial: Vec<Vec<&str>> = vec![vec![]];
    lines.fold(initial, |mut acc, l: &str| match l.trim() {
        // HANDLE BLANK LINES
        "" => {
            // If there are exactly two spaces in the line, it's intentional
            if l.len() == 2 {
                acc.last_mut().unwrap().push(l);
            // If the previous element was also blank, create an empty string
            } else if acc.last().unwrap().is_empty() {
                acc.last_mut().unwrap().push("");
            // Otherwise, start a new element by pushing a new empty vec
            } else {
                acc.push(vec![]);
            }
            acc
        }
        /* HANDLE SECTIONS
         * They don't follow the simple rules of blank line before or after.
         * So we need this special case to handle them.
         */
        l if l.starts_with('#') => {
            // If the previous hunk was empty, use it.
            if acc.last().unwrap().is_empty() {
                acc.last_mut().unwrap().push(l);
            // If previous hunk wasn't empty, create a new one.
            } else {
                acc.push(vec![l]);
            }
            // Sections are isolated, so start a new empty hunk for next element.
            acc.push(vec![]);
            acc
        }
        // HANDLE NORMAL, NON-EMPTY LINES
        l => {
            acc.last_mut().unwrap().push(l);
            acc
        }
    })
}

/// Strips out problematic unicode and the boneyard element
fn prepare_text(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/\*[^*]*\*/|\p{gc:Cf}").unwrap();
    }
    RE.replace_all(text, "").to_string()
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
        BenchmarkId::new("Prepare Text", "short dirty"),
        &short_dirty,
        |b, s| b.iter(|| prepare_text(s)),
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
        BenchmarkId::new("Prepare Text", "big dirty"),
        &big_dirty,
        |b, s| b.iter(|| prepare_text(s)),
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
    unicode_group.bench_with_input(
        BenchmarkId::new("Prepare Text", "big clean"),
        &clean_text,
        |b, s| b.iter(|| prepare_text(s)),
    );
    unicode_group.finish();

    let mut hunk_group = c.benchmark_group("Hunks");

    let medium_text: Lines = clean_text.lines();

    hunk_group.bench_with_input(
        BenchmarkId::new("Lines to Hunks", "complete"),
        &medium_text,
        |b, s| b.iter(|| lines_to_hunks_complete(s.clone())),
    );

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
