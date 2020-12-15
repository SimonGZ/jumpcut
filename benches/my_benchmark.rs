use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jumpcut::parse;
use lazy_static::lazy_static;
use regex::Regex;

const VALID_KEYS: [&str; 10] = [
    "title:",
    "credit:",
    "author:",
    "authors:",
    "source:",
    "date:",
    "draft date:",
    "contact:",
    "format:",
    "template:",
];

fn has_key_value(txt: &str) -> bool {
    let ltxt = txt.to_lowercase();
    VALID_KEYS.iter().any(|key| ltxt.starts_with(key))
}

fn has_key_value_regex(txt: &str) -> bool {
    lazy_static! {
        static ref KEY: Regex = Regex::new(r"(?P<key>^[^\s][^:\n\r]+):(?P<value>.*)").unwrap();
    }
    KEY.is_match(txt)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut parse_group = c.benchmark_group("Parse");

    let scd_text = include_str!("108.fountain");
    let big_fish_text = include_str!("Big-Fish.fountain");

    let titlepage_text = "Title:\n    _**BRICK & STEEL**_\n    _**FULL RETIRED**_\nCredit: Written by\nAuthor: Stu Maschwitz\nSource: Story by KTM\nDraft date: 1/20/2012\nContact:\n    Next Level Productions\n    1588 Mission Dr.\n    Solvang, CA 93463";

    parse_group.bench_with_input(
        BenchmarkId::new("Has Key Value", "starts_with"),
        &titlepage_text,
        |b, s| b.iter(|| has_key_value(s)),
    );
    parse_group.bench_with_input(
        BenchmarkId::new("Has Key Value", "regex"),
        &titlepage_text,
        |b, s| b.iter(|| has_key_value_regex(s)),
    );
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
