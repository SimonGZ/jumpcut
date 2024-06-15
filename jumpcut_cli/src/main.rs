use jumpcut::parse;
use serde_json;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "JumpCut",
    about = "A tool for converting Fountain screenplay documents into Final Draft (FDX) and HTML formats."
)]
struct Args {
    /// Formats (FDX, HTML, JSON)
    #[arg(short, long, default_value = "fdx")]
    format: String,

    /// Input file, pass a dash ("-") to receive stdin
    input: PathBuf,

    /// Output file, stdout if not present
    output: Option<PathBuf>,
}

fn main() {
    let opt = Args::parse();
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

    let mut screenplay = parse(&content);

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
            output_text = screenplay.to_final_draft();
        }
        x if x.to_lowercase() == "html" => {
            output_text = screenplay.to_html(true);
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
