use jumpcut::parse;
use std::fs;
use std::io::{self, Read, Write};
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

    /// Input file, pass a dash ("-") to receive stdin
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let mut content = String::new();
    if opt.input.is_file() {
        content.push_str(&std::fs::read_to_string(opt.input).expect("Could not read file."));
    } else {
        if opt.input.to_str() == Some("-") {
            let mut buffer = String::new();
            let stdin = io::stdin().read_to_string(&mut buffer);
            match stdin {
                Err(_) => panic!("Invalid text piped to function."),
                Ok(_) => content.push_str(&buffer),
            }
        } else {
            eprintln!("Error: Did not receive a valid file.");
            std::process::exit(1);
        }
    }

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
