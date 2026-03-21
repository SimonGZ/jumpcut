use std::fs;
use std::time::Instant;

fn median_ns(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn bench_parse(path: &str, iterations: usize, warmups: usize) -> u128 {
    let content = fs::read_to_string(path).expect("benchmark input should exist");

    for _ in 0..warmups {
        let _ = jumpcut::parse(&content);
    }

    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = jumpcut::parse(&content);
        samples.push(start.elapsed().as_nanos());
    }

    median_ns(&mut samples)
}

fn main() {
    let parse_108_ns = bench_parse("benches/108.fountain", 300, 20);
    let parse_big_fish_ns = bench_parse("benches/Big-Fish.fountain", 120, 10);
    let total_ns = parse_108_ns + parse_big_fish_ns;

    println!("METRIC parse_108_ns={parse_108_ns}");
    println!("METRIC parse_big_fish_ns={parse_big_fish_ns}");
    println!("METRIC total_ns={total_ns}");
}
