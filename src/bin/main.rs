#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use jumpcut::parse;
#[cfg(feature = "cli")]
use serde_json;
#[cfg(feature = "cli")]
use std::fs;
#[cfg(feature = "cli")]
use std::io::{self, Read, Write};
#[cfg(feature = "cli")]
use std::path::PathBuf;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(
    name = "JumpCut",
    about = "A tool for converting Fountain screenplay documents into Final Draft (FDX) and HTML formats.",
    version
)]
#[cfg(feature = "cli")]
struct Args {
    /// Formats (FDX, HTML, JSON)
    #[arg(short, long, default_value = "fdx")]
    format: String,

    /// Input file, pass a dash ("-") to receive stdin
    input: PathBuf,

    /// Output file, stdout if not present
    output: Option<PathBuf>,

    /// Optional Fountain file to prepend as metadata. Defaults to "metadata.fountain" if flag is present without a value.
    #[arg(short, long, value_name = "FILE", num_args = 0..=1, default_missing_value = "metadata.fountain")]
    metadata: Option<PathBuf>,
}
#[cfg(feature = "cli")]
fn main() {
    let opt = Args::parse();
    let mut content = String::new();

    // Handle metadata file first
    if let Some(metadata_arg_path) = opt.metadata {
        let mut actual_metadata_file_path = metadata_arg_path.clone();

        // If the default_missing_value was used, determine the correct path
        if metadata_arg_path.to_str() == Some("metadata.fountain") {
            if opt.input.is_file() {
                // If input is a file, metadata.fountain is relative to input's directory
                if let Some(parent) = opt.input.parent() {
                    actual_metadata_file_path = parent.join("metadata.fountain");
                } else {
                    // If input has no parent (e.g., just "file.txt" in CWD), use CWD
                    actual_metadata_file_path = PathBuf::from("metadata.fountain");
                }
            } else {
                // If input is stdin or not a file, metadata.fountain is relative to CWD
                actual_metadata_file_path = PathBuf::from("metadata.fountain");
            }
        }
        // If metadata_arg_path was not "metadata.fountain", it's an explicit path, use it directly.

        match fs::read_to_string(&actual_metadata_file_path) {
            Ok(metadata_content) => {
                content.push_str(&metadata_content);
                content.push_str("\n"); // Prepend with line break
            }
            Err(e) => {
                eprintln!(
                    "Error reading metadata file '{}': {}",
                    actual_metadata_file_path.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    // Now read the main input content
    if opt.input.is_file() {
        content.push_str(&std::fs::read_to_string(&opt.input).expect("Could not read file."));
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
