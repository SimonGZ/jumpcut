use jumpcut::parse;
use serde_json;
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
    /// Formats (FDX, HTML, JSON)
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

    let mut output_text = String::new();

    match opt.format {
        x if x.to_lowercase() == "json" => {
            let j = serde_json::to_string_pretty(&screenplay);
            match j {
                Ok(json) => output_text = json,
                Err(e) => eprintln!("{}", e),
            }
        }
        x if x.to_lowercase() == "fdx" => {
            output_text = "fdx".to_string();
        }
        x if x.to_lowercase() == "html" => {
            output_text = "html".to_string();
        }
        _ => output_text = "nothing".to_string(),
    }

    match opt.output {
        Some(outfile) => fs::write(outfile, output_text).expect("Unable to write file."),
        None => {
            let stdout = io::stdout();
            let mut handle = io::BufWriter::new(stdout);
            writeln!(handle, "{}", output_text).expect("Unable to write to buffer.");
        }
    }
}
