use jumpcut::parse;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "JumpCut",
    about = "A tool for converting Fountain screenplay documents into Final Draft (FDX) and HTML formats."
)]
struct Opt {
    /// Formats (FDX or HTML)
    #[structopt(short, long, default_value = "fdx")]
    format: String,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let content = std::fs::read_to_string(opt.input).expect("Could not read file.");
    let screenplay = parse(&content);
    match opt.output {
        Some(outfile) => {
            fs::write(outfile, format!("{:#?}", screenplay)).expect("Unable to write file.")
        }
        None => {
            let stdout = io::stdout();
            let mut handle = io::BufWriter::new(stdout);
            writeln!(handle, "{:#?}", screenplay).expect("Unable to write to buffer.");
        }
    }
}
