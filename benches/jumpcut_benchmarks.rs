use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jumpcut::{blank_attributes, parse, Element::Action, ElementText};
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

    let mut markup_group = c.benchmark_group("Markup");
    markup_group.measurement_time(Duration::from_secs(8));

    let dense_markup = "***THE _STARS_ ALIGN*** as **everyone** *rushes* to the stage.";
    markup_group.bench_with_input(
        BenchmarkId::new("Markup", "dense"),
        &dense_markup,
        |b, content| {
            b.iter(|| {
                let mut element = Action(
                    ElementText::Plain((*content).to_string()),
                    blank_attributes(),
                );
                element.parse_and_convert_markup();
            })
        },
    );

    let long_dialogue = r#"***STAR***
_*I can feel the pull.*_

***STAR***
*Hold steady.*

***STAR***
**Now!**"#;
    markup_group.bench_with_input(
        BenchmarkId::new("Markup", "multi_line"),
        &long_dialogue,
        |b, content| {
            b.iter(|| {
                let mut element = Action(
                    ElementText::Plain((*content).to_string()),
                    blank_attributes(),
                );
                element.parse_and_convert_markup();
            })
        },
    );
    markup_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
