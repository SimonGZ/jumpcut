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
    about = "A tool for converting Fountain screenplay documents into Final Draft (FDX), HTML, and text formats.",
    version
)]
#[cfg(feature = "cli")]
struct Args {
    /// Formats (FDX, HTML, JSON, text, PDF)
    #[arg(short, long, default_value = "fdx")]
    format: String,

    /// Render text output with pagination
    #[arg(long)]
    paginate: bool,

    /// Render HTML output with exact Final Draft-style wraps
    #[arg(long)]
    exact_wraps: bool,

    /// Embed Courier Prime font files directly into HTML CSS
    #[arg(long)]
    embed_courier_prime: bool,

    /// Show line numbers in text output
    #[arg(long)]
    line_numbers: bool,

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
    let format = opt.format.to_lowercase();

    if format != "text" && format != "html" && opt.paginate {
        eprintln!("Error: --paginate is only supported with --format text or --format html.");
        std::process::exit(2);
    }

    if format != "text" && opt.line_numbers {
        eprintln!("Error: --line-numbers is only supported with --format text.");
        std::process::exit(2);
    }

    if format != "html" && opt.exact_wraps {
        eprintln!("Error: --exact-wraps is only supported with --format html.");
        std::process::exit(2);
    }

    let output_bytes = match format.as_str() {
        "json" => match serde_json::to_string_pretty(&screenplay) {
            Ok(json) => json.into_bytes(),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        "fdx" => screenplay.to_final_draft().into_bytes(),
        "html" => screenplay
            .to_html_with_options(jumpcut::html_output::HtmlRenderOptions {
                head: true,
                exact_wraps: opt.exact_wraps || opt.paginate,
                paginated: opt.paginate,
                render_continueds: true,
                embed_courier_prime: opt.embed_courier_prime,
                embedded_courier_prime_css: None,
            })
            .into_bytes(),
        "text" => screenplay
            .to_text(&jumpcut::text_output::TextRenderOptions {
                paginated: opt.paginate,
                line_numbers: opt.line_numbers,
                render_continueds: true,
            })
            .into_bytes(),
        "pdf" => screenplay.to_pdf(),
        _ => b"nothing".to_vec(),
    };

    match opt.output {
        Some(outfile) => fs::write(outfile, output_bytes).expect("Unable to write file."),
        None => {
            let stdout = io::stdout();
            let mut handle = io::BufWriter::new(stdout);
            handle
                .write_all(&output_bytes)
                .expect("Unable to write to buffer.");
        }
    }
}
