use fountain_converter::parse;
use std::fs;

fn main() {
    let scd = fs::read_to_string(
        "/Users/simonganz/Programming/Rust/fountain_converter/benches/108.fountain",
    )
    .expect("file to load");
    parse(&scd);
}
