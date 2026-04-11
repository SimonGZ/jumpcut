#[cfg(feature = "cli")]
use clap::{Parser, ValueEnum};
#[cfg(feature = "cli")]
use jumpcut::parse;
#[cfg(feature = "cli")]
use jumpcut::ElementText;
#[cfg(feature = "cli")]
use serde_json;
#[cfg(feature = "cli")]
use std::fs;
#[cfg(feature = "cli")]
use std::io::{self, Read, Write};
#[cfg(feature = "cli")]
use std::path::{Path, PathBuf};

#[cfg(feature = "cli")]
const AUTO_OUTPUT_SENTINEL: &str = "__jumpcut_auto_output__";

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
    #[arg(short, long)]
    format: Option<String>,

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

    /// Override the layout/render profile instead of using fmt metadata
    #[arg(long, value_enum)]
    render_profile: Option<RenderProfile>,

    /// Suppress (CONT'D)/(MORE) style continued markers in render outputs
    #[arg(long)]
    no_continueds: bool,

    /// Input file, pass a dash ("-") to receive stdin
    input: PathBuf,

    /// Output file in the legacy positional form.
    #[arg(conflicts_with = "output_flag")]
    output: Option<PathBuf>,

    /// Output file. Pass bare -o/--output to auto-derive a file name from the input and format.
    #[arg(short = 'o', long = "output", value_name = "FILE", num_args = 0..=1, default_missing_value = AUTO_OUTPUT_SENTINEL, conflicts_with = "output")]
    output_flag: Option<PathBuf>,

    /// Optional Fountain file to prepend as metadata. Defaults to "metadata.fountain" if flag is present without a value.
    #[arg(short, long, value_name = "FILE", num_args = 0..=1, default_missing_value = "metadata.fountain")]
    metadata: Option<PathBuf>,
}

#[cfg(feature = "cli")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RenderProfile {
    FinalDraft,
    Balanced,
}

#[cfg(feature = "cli")]
fn main() {
    let opt = Args::parse();
    let mut content = String::new();

    // Handle metadata file first
    if let Some(ref metadata_arg_path) = opt.metadata {
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
    apply_cli_render_overrides(&mut screenplay, &opt);
    let requested_output = opt.output_flag.as_ref().or(opt.output.as_ref());
    let explicit_output = requested_output.filter(|path| !is_auto_output_marker(path));
    let format = infer_format(opt.format.as_deref(), explicit_output);
    let output_path = resolve_output_path(&opt.input, requested_output, &format).unwrap_or_else(|error| {
        eprintln!("Error: {error}");
        std::process::exit(2);
    });

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
            .to_html_with_options(jumpcut::rendering::html::HtmlRenderOptions {
                head: true,
                exact_wraps: opt.exact_wraps || opt.paginate,
                paginated: opt.paginate,
                render_continueds: !opt.no_continueds,
                embed_courier_prime: opt.embed_courier_prime,
                embedded_courier_prime_css: None,
            })
            .into_bytes(),
        "text" => screenplay
            .to_text(&jumpcut::rendering::text::TextRenderOptions {
                paginated: opt.paginate,
                line_numbers: opt.line_numbers,
                render_continueds: !opt.no_continueds,
            })
            .into_bytes(),
        "pdf" => screenplay.to_pdf_with_options(jumpcut::rendering::pdf::PdfRenderOptions {
            render_continueds: !opt.no_continueds,
        }),
        _ => b"nothing".to_vec(),
    };

    match output_path {
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

#[cfg(feature = "cli")]
fn infer_format(format_opt: Option<&str>, output_opt: Option<&PathBuf>) -> String {
    match format_opt {
        Some(f) => f.to_lowercase(),
        None => match output_opt {
            Some(out) => match out.extension().and_then(|e| e.to_str()) {
                Some("pdf") => "pdf".to_string(),
                Some("html") | Some("htm") => "html".to_string(),
                Some("txt") | Some("text") => "text".to_string(),
                Some("json") => "json".to_string(),
                Some("fdx") => "fdx".to_string(),
                _ => "fdx".to_string(),
            },
            None => "fdx".to_string(),
        },
    }
}

#[cfg(feature = "cli")]
fn resolve_output_path(
    input: &Path,
    output_opt: Option<&PathBuf>,
    format: &str,
) -> Result<Option<PathBuf>, String> {
    match output_opt {
        Some(path) if is_auto_output_marker(path) => auto_output_path(input, format)
            .map(Some)
            .ok_or_else(|| "cannot auto-derive an output path when input is stdin".to_string()),
        Some(path) => Ok(Some(path.clone())),
        None => Ok(None),
    }
}

#[cfg(feature = "cli")]
fn auto_output_path(input: &Path, format: &str) -> Option<PathBuf> {
    if input == Path::new("-") {
        return None;
    }

    Some(input.with_extension(default_extension_for_format(format)))
}

#[cfg(feature = "cli")]
fn default_extension_for_format(format: &str) -> &'static str {
    match format {
        "pdf" => "pdf",
        "html" => "html",
        "text" => "txt",
        "json" => "json",
        _ => "fdx",
    }
}

#[cfg(feature = "cli")]
fn is_auto_output_marker(path: &Path) -> bool {
    path.to_str() == Some(AUTO_OUTPUT_SENTINEL)
}

#[cfg(feature = "cli")]
fn apply_cli_render_overrides(screenplay: &mut jumpcut::Screenplay, opt: &Args) {
    if let Some(render_profile) = opt.render_profile {
        apply_render_profile_override(&mut screenplay.metadata, render_profile);
    }
}

#[cfg(feature = "cli")]
fn apply_render_profile_override(metadata: &mut jumpcut::Metadata, render_profile: RenderProfile) {
    const PROFILE_TOKENS: &[&str] = &["balanced", "clean-dashes", "no-dual-contds"];

    let mut tokens = metadata
        .get("fmt")
        .and_then(|values| values.first())
        .map(|value| value.plain_text())
        .unwrap_or_default()
        .split_whitespace()
        .filter(|token| {
            !PROFILE_TOKENS
                .iter()
                .any(|profile_token| token.eq_ignore_ascii_case(profile_token))
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    if matches!(render_profile, RenderProfile::Balanced) {
        tokens.push("balanced".to_string());
    }

    if tokens.is_empty() {
        metadata.remove("fmt");
    } else {
        metadata.insert(
            "fmt".to_string(),
            vec![ElementText::Plain(tokens.join(" "))],
        );
    }
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    use super::{
        apply_render_profile_override, infer_format, is_auto_output_marker, resolve_output_path,
        Args, RenderProfile,
    };
    use jumpcut::{ElementText, Metadata};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn render_profile_override_replaces_balanced_family_tokens_only() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "fmt".into(),
            vec![ElementText::Plain(
                "allow-lowercase-title balanced clean-dashes no-dual-contds dl-2.0".into(),
            )],
        );

        apply_render_profile_override(&mut metadata, RenderProfile::FinalDraft);

        let fmt = metadata
            .get("fmt")
            .and_then(|values| values.first())
            .map(|value| value.plain_text())
            .unwrap();
        assert_eq!(fmt, "allow-lowercase-title dl-2.0");
    }

    #[test]
    fn render_profile_override_adds_balanced_without_dropping_other_fmt_knobs() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "fmt".into(),
            vec![ElementText::Plain("allow-lowercase-title dl-2.0".into())],
        );

        apply_render_profile_override(&mut metadata, RenderProfile::Balanced);

        let fmt = metadata
            .get("fmt")
            .and_then(|values| values.first())
            .map(|value| value.plain_text())
            .unwrap();
        assert_eq!(fmt, "allow-lowercase-title dl-2.0 balanced");
    }

    #[test]
    fn format_inference_uses_explicit_format_arg_first() {
        assert_eq!(
            infer_format(Some("HTML"), Some(&PathBuf::from("out.pdf"))),
            "html"
        );
    }

    #[test]
    fn format_inference_falls_back_to_extension() {
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.pdf"))), "pdf");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.html"))), "html");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.htm"))), "html");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.txt"))), "text");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.text"))), "text");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.json"))), "json");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.fdx"))), "fdx");
    }

    #[test]
    fn format_inference_defaults_to_fdx_if_no_extension_or_unknown() {
        assert_eq!(infer_format(None, Some(&PathBuf::from("out"))), "fdx");
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.mp3"))), "fdx");
        assert_eq!(infer_format(None, None), "fdx");
    }

    #[test]
    fn cli_accepts_bare_output_flag_for_auto_output_path() {
        let parsed = Args::try_parse_from(["jumpcut", "big fish.fountain", "-o"]);
        assert!(
            parsed.is_ok(),
            "expected bare -o to parse as an auto-output flag"
        );
    }

    #[test]
    fn cli_accepts_bare_output_flag_with_explicit_pdf_format() {
        let parsed = Args::try_parse_from(["jumpcut", "big fish.fountain", "-o", "-f", "pdf"]);
        assert!(
            parsed.is_ok(),
            "expected bare -o with -f pdf to parse as an auto-output flag"
        );
    }

    #[test]
    fn bare_output_flag_uses_input_stem_and_default_fdx_extension() {
        let args = Args::try_parse_from(["jumpcut", "big fish.fountain", "-o"]).unwrap();
        let requested_output = args.output_flag.as_ref().or(args.output.as_ref());
        let explicit_output = requested_output.filter(|path| !is_auto_output_marker(path));
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(format, "fdx");
        assert_eq!(
            resolve_output_path(&args.input, requested_output, &format).unwrap(),
            Some(PathBuf::from("big fish.fdx"))
        );
    }

    #[test]
    fn bare_output_flag_uses_explicit_pdf_format_for_extension() {
        let args = Args::try_parse_from(["jumpcut", "big fish.fountain", "-o", "-f", "pdf"]).unwrap();
        let requested_output = args.output_flag.as_ref().or(args.output.as_ref());
        let explicit_output = requested_output.filter(|path| !is_auto_output_marker(path));
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(format, "pdf");
        assert_eq!(
            resolve_output_path(&args.input, requested_output, &format).unwrap(),
            Some(PathBuf::from("big fish.pdf"))
        );
    }

    #[test]
    fn bare_output_flag_cannot_auto_derive_from_stdin() {
        let args = Args::try_parse_from(["jumpcut", "-", "-o"]).unwrap();
        let requested_output = args.output_flag.as_ref().or(args.output.as_ref());
        let explicit_output = requested_output.filter(|path| !is_auto_output_marker(path));
        let format = infer_format(args.format.as_deref(), explicit_output);

        let error = resolve_output_path(&args.input, requested_output, &format).unwrap_err();
        assert_eq!(error, "cannot auto-derive an output path when input is stdin");
    }

    #[test]
    fn positional_output_path_still_parses_and_controls_format() {
        let args = Args::try_parse_from(["jumpcut", "input.fountain", "output.pdf"]).unwrap();
        let requested_output = args.output_flag.as_ref().or(args.output.as_ref());
        let explicit_output = requested_output.filter(|path| !is_auto_output_marker(path));
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(requested_output, Some(&PathBuf::from("output.pdf")));
        assert_eq!(format, "pdf");
        assert_eq!(
            resolve_output_path(&args.input, requested_output, &format).unwrap(),
            Some(PathBuf::from("output.pdf"))
        );
    }
}
