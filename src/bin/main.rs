#[cfg(feature = "cli")]
use clap::{Parser, ValueEnum};
#[cfg(feature = "cli")]
use jumpcut::{parse, parse_fdx};
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
#[derive(Parser)]
#[command(
    name = "JumpCut",
    about = "A tool for converting Fountain and Final Draft screenplay documents into Fountain, FDX, HTML, JSON, text, and optional PDF formats.",
    version
)]
#[cfg(feature = "cli")]
struct Args {
    /// Formats (Fountain, FDX, HTML, JSON, text, PDF)
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

    /// Suppress title-page output for HTML and PDF renders
    #[arg(long)]
    no_title_page: bool,

    /// Input file, pass a dash ("-") to receive stdin
    input: PathBuf,

    /// Output file in the legacy positional form.
    #[arg(conflicts_with_all = ["output_flag", "write"])]
    output: Option<PathBuf>,

    /// Output file.
    #[arg(short = 'o', long = "output", value_name = "FILE", conflicts_with_all = ["output", "write"])]
    output_flag: Option<PathBuf>,

    /// Auto-derive an output file path from the input stem and format.
    #[arg(short = 'w', long = "write", conflicts_with_all = ["output", "output_flag"])]
    write: bool,

    /// Optional Fountain file to merge as metadata. Defaults to "metadata.fountain" if flag is present without a value.
    #[arg(short, long, value_name = "FILE", num_args = 0..=1, default_missing_value = "metadata.fountain")]
    metadata: Option<PathBuf>,
}

#[cfg(feature = "cli")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum RenderProfile {
    Industry,
    Balanced,
}

#[cfg(feature = "cli")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputFormat {
    Fountain,
    Fdx,
}

#[cfg(feature = "cli")]
fn main() {
    let opt = Args::parse();
    let metadata = read_cli_metadata(&opt).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    let content = read_cli_input(&opt.input).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });

    let mut screenplay = parse_cli_input(&opt.input, &content, metadata).unwrap_or_else(|error| {
        eprintln!("Error: {error}");
        std::process::exit(1);
    });
    apply_cli_render_overrides(&mut screenplay, &opt);
    let explicit_output = opt.output_flag.as_ref().or(opt.output.as_ref());
    let format = infer_format(opt.format.as_deref(), explicit_output);
    let output_path = resolve_output_path(&opt.input, explicit_output, opt.write, &format)
        .unwrap_or_else(|error| {
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

    if format != "html" && (!pdf_output_enabled() || format != "pdf") && opt.no_title_page {
        eprintln!(
            "Error: --no-title-page is only supported with --format html{}.",
            if pdf_output_enabled() {
                " or --format pdf"
            } else {
                ""
            }
        );
        std::process::exit(2);
    }

    let output_bytes = match format.as_str() {
        "fountain" => screenplay.to_fountain().into_bytes(),
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
                render_title_page: !opt.no_title_page,
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
        #[cfg(feature = "pdf")]
        "pdf" => screenplay.to_pdf_with_options(jumpcut::rendering::pdf::PdfRenderOptions {
            render_continueds: !opt.no_continueds,
            render_title_page: !opt.no_title_page,
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
                Some("fountain") => "fountain".to_string(),
                #[cfg(feature = "pdf")]
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
    explicit_output_opt: Option<&PathBuf>,
    write: bool,
    format: &str,
) -> Result<Option<PathBuf>, String> {
    match explicit_output_opt {
        Some(path) => Ok(Some(path.clone())),
        None if write => {
            let output = auto_output_path(input, format)
                .ok_or_else(|| "cannot auto-derive an output path when input is stdin".to_string())?;
            if output == input {
                return Err(
                    "auto-derived output path matches the input path; specify --format or --output"
                        .to_string(),
                );
            }
            Ok(Some(output))
        }
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
        "fountain" => "fountain",
        #[cfg(feature = "pdf")]
        "pdf" => "pdf",
        "html" => "html",
        "text" => "txt",
        "json" => "json",
        _ => "fdx",
    }
}

#[cfg(feature = "cli")]
fn pdf_output_enabled() -> bool {
    cfg!(feature = "pdf")
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

#[cfg(feature = "cli")]
fn read_cli_metadata(opt: &Args) -> Result<jumpcut::Metadata, String> {
    let Some(metadata_arg_path) = &opt.metadata else {
        return Ok(jumpcut::Metadata::new());
    };
    let actual_metadata_file_path = resolve_metadata_path(&opt.input, metadata_arg_path);
    let metadata_content = fs::read_to_string(&actual_metadata_file_path).map_err(|error| {
        format!(
            "Error reading metadata file '{}': {}",
            actual_metadata_file_path.display(),
            error
        )
    })?;
    Ok(parse(&metadata_content).metadata)
}

#[cfg(feature = "cli")]
fn resolve_metadata_path(input: &Path, metadata_arg_path: &Path) -> PathBuf {
    if metadata_arg_path.to_str() != Some("metadata.fountain") {
        return metadata_arg_path.to_path_buf();
    }

    if input != Path::new("-") {
        input.parent().unwrap_or_else(|| Path::new("")).join(metadata_arg_path)
    } else {
        metadata_arg_path.to_path_buf()
    }
}

#[cfg(feature = "cli")]
fn read_cli_input(input: &Path) -> Result<String, String> {
    if input.is_file() {
        fs::read_to_string(input)
            .map_err(|error| format!("Could not read file '{}': {error}", input.display()))
    } else if input.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|_| "Invalid text piped to function.".to_string())?;
        Ok(buffer)
    } else {
        Err("Did not receive a valid file.".to_string())
    }
}

#[cfg(feature = "cli")]
fn parse_cli_input(
    input: &Path,
    content: &str,
    metadata: jumpcut::Metadata,
) -> Result<jumpcut::Screenplay, String> {
    let input_format = infer_input_format(input, content);
    let mut screenplay = match input_format {
        InputFormat::Fountain => parse(content),
        InputFormat::Fdx => parse_fdx(content).map_err(|error| error.to_string())?,
    };

    if !metadata.is_empty() {
        let mut combined_metadata = metadata;
        combined_metadata.extend(screenplay.metadata);
        screenplay.metadata = combined_metadata;
    }

    Ok(screenplay)
}

#[cfg(feature = "cli")]
fn infer_input_format(input: &Path, content: &str) -> InputFormat {
    match input.extension().and_then(|value| value.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("fdx") => InputFormat::Fdx,
        _ if looks_like_fdx(content) => InputFormat::Fdx,
        _ => InputFormat::Fountain,
    }
}

#[cfg(feature = "cli")]
fn looks_like_fdx(content: &str) -> bool {
    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    trimmed.starts_with("<FinalDraft")
        || (trimmed.starts_with("<?xml") && trimmed.contains("<FinalDraft"))
}

#[cfg(all(test, feature = "cli"))]
mod tests {
    #[cfg(not(feature = "pdf"))]
    use super::pdf_output_enabled;
    use super::{
        apply_render_profile_override, infer_format, infer_input_format, looks_like_fdx,
        parse_cli_input, resolve_metadata_path, resolve_output_path, Args, InputFormat,
        RenderProfile,
    };
    use clap::Parser;
    use jumpcut::{ElementText, Metadata};
    use std::path::{Path, PathBuf};

    #[test]
    fn render_profile_override_replaces_balanced_family_tokens_only() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "fmt".into(),
            vec![ElementText::Plain(
                "allow-lowercase-title balanced clean-dashes no-dual-contds dl-2.0".into(),
            )],
        );

        apply_render_profile_override(&mut metadata, RenderProfile::Industry);

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
    fn cli_accepts_industry_render_profile_name() {
        let parsed =
            Args::try_parse_from(["jumpcut", "script.fountain", "--render-profile", "industry"]);
        assert!(parsed.is_ok());
    }

    #[test]
    fn cli_rejects_removed_final_draft_render_profile_name() {
        let parsed = Args::try_parse_from([
            "jumpcut",
            "script.fountain",
            "--render-profile",
            "final-draft",
        ]);
        assert!(parsed.is_err());
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
        #[cfg(feature = "pdf")]
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.pdf"))), "pdf");
        #[cfg(not(feature = "pdf"))]
        assert_eq!(infer_format(None, Some(&PathBuf::from("out.pdf"))), "fdx");
        assert_eq!(
            infer_format(None, Some(&PathBuf::from("out.fountain"))),
            "fountain"
        );
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
    fn input_format_inference_uses_fdx_extension() {
        assert_eq!(
            infer_input_format(Path::new("script.fdx"), "INT. HOUSE - DAY"),
            InputFormat::Fdx
        );
    }

    #[test]
    fn input_format_inference_detects_fdx_from_xml_content() {
        assert_eq!(
            infer_input_format(
                Path::new("-"),
                "<?xml version=\"1.0\"?><FinalDraft DocumentType=\"Script\"></FinalDraft>"
            ),
            InputFormat::Fdx
        );
        assert!(looks_like_fdx(
            "\u{feff}\n  <FinalDraft DocumentType=\"Script\"></FinalDraft>"
        ));
    }

    #[test]
    fn input_format_inference_keeps_plain_fountain_as_fountain() {
        assert_eq!(
            infer_input_format(Path::new("script.fountain"), "Title: Example\n\nINT. HOUSE - DAY"),
            InputFormat::Fountain
        );
    }

    #[test]
    fn cli_accepts_write_flag_for_auto_output_path() {
        let parsed = Args::try_parse_from(["jumpcut", "big fish.fountain", "-w"]);
        assert!(
            parsed.is_ok(),
            "expected -w to parse as an auto-output flag"
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn cli_accepts_write_flag_with_explicit_pdf_format() {
        let parsed = Args::try_parse_from(["jumpcut", "big fish.fountain", "-w", "-f", "pdf"]);
        assert!(
            parsed.is_ok(),
            "expected -w with -f pdf to parse as an auto-output flag"
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn cli_accepts_no_title_page_for_html_and_pdf() {
        assert!(Args::try_parse_from([
            "jumpcut",
            "script.fountain",
            "-f",
            "html",
            "--no-title-page"
        ])
        .is_ok());
        assert!(Args::try_parse_from([
            "jumpcut",
            "script.fountain",
            "-f",
            "pdf",
            "--no-title-page"
        ])
        .is_ok());
    }

    #[cfg(not(feature = "pdf"))]
    #[test]
    fn cli_only_accepts_no_title_page_for_html_when_pdf_output_is_disabled() {
        assert!(Args::try_parse_from([
            "jumpcut",
            "script.fountain",
            "-f",
            "html",
            "--no-title-page"
        ])
        .is_ok());
        assert!(!pdf_output_enabled());
    }

    #[test]
    fn write_flag_uses_input_stem_and_default_fdx_extension() {
        let args = Args::try_parse_from(["jumpcut", "big fish.fountain", "-w"]).unwrap();
        let explicit_output = args.output_flag.as_ref().or(args.output.as_ref());
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(format, "fdx");
        assert_eq!(
            resolve_output_path(&args.input, explicit_output, args.write, &format).unwrap(),
            Some(PathBuf::from("big fish.fdx"))
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn write_flag_uses_explicit_pdf_format_for_extension() {
        let args =
            Args::try_parse_from(["jumpcut", "big fish.fountain", "-w", "-f", "pdf"]).unwrap();
        let explicit_output = args.output_flag.as_ref().or(args.output.as_ref());
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(format, "pdf");
        assert_eq!(
            resolve_output_path(&args.input, explicit_output, args.write, &format).unwrap(),
            Some(PathBuf::from("big fish.pdf"))
        );
    }

    #[test]
    fn write_flag_uses_explicit_fountain_format_for_extension() {
        let args =
            Args::try_parse_from(["jumpcut", "big fish.fdx", "-w", "-f", "fountain"]).unwrap();
        let explicit_output = args.output_flag.as_ref().or(args.output.as_ref());
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(format, "fountain");
        assert_eq!(
            resolve_output_path(&args.input, explicit_output, args.write, &format).unwrap(),
            Some(PathBuf::from("big fish.fountain"))
        );
    }

    #[test]
    fn write_flag_cannot_auto_derive_from_stdin() {
        let args = Args::try_parse_from(["jumpcut", "-", "-w"]).unwrap();
        let explicit_output = args.output_flag.as_ref().or(args.output.as_ref());
        let format = infer_format(args.format.as_deref(), explicit_output);

        let error =
            resolve_output_path(&args.input, explicit_output, args.write, &format).unwrap_err();
        assert_eq!(
            error,
            "cannot auto-derive an output path when input is stdin"
        );
    }

    #[test]
    fn write_flag_rejects_auto_deriving_same_path_as_fdx_input() {
        let args = Args::try_parse_from(["jumpcut", "script.fdx", "-w"]).unwrap();
        let explicit_output = args.output_flag.as_ref().or(args.output.as_ref());
        let format = infer_format(args.format.as_deref(), explicit_output);

        let error =
            resolve_output_path(&args.input, explicit_output, args.write, &format).unwrap_err();
        assert_eq!(
            error,
            "auto-derived output path matches the input path; specify --format or --output"
        );
    }

    #[test]
    fn metadata_default_path_is_relative_to_input_directory() {
        assert_eq!(
            resolve_metadata_path(
                Path::new("fixtures/script.fountain"),
                Path::new("metadata.fountain")
            ),
            PathBuf::from("fixtures/metadata.fountain")
        );
    }

    #[test]
    fn parse_cli_input_accepts_fdx_and_merges_metadata_without_corrupting_xml() {
        let mut metadata = Metadata::new();
        metadata.insert("contact".into(), vec!["fallback@example.com".into()]);
        let screenplay = parse_cli_input(
            Path::new("script.fdx"),
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<FinalDraft DocumentType="Script" Template="No" Version="4">
  <TitlePage>
    <Content>
      <Paragraph Alignment="Center"><Text>Imported Script</Text></Paragraph>
    </Content>
  </TitlePage>
  <Content>
    <Paragraph Type="Action"><Text>Body.</Text></Paragraph>
  </Content>
</FinalDraft>"#,
            metadata,
        )
        .expect("fdx should parse");

        assert_eq!(
            screenplay.metadata.get("title"),
            Some(&vec!["Imported Script".into()])
        );
        assert_eq!(
            screenplay.metadata.get("contact"),
            Some(&vec!["fallback@example.com".into()])
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn positional_output_path_still_parses_and_controls_format() {
        let args = Args::try_parse_from(["jumpcut", "input.fountain", "output.pdf"]).unwrap();
        let requested_output = args.output_flag.as_ref().or(args.output.as_ref());
        let explicit_output = requested_output;
        let format = infer_format(args.format.as_deref(), explicit_output);

        assert_eq!(requested_output, Some(&PathBuf::from("output.pdf")));
        assert_eq!(format, "pdf");
        assert_eq!(
            resolve_output_path(&args.input, requested_output, args.write, &format).unwrap(),
            Some(PathBuf::from("output.pdf"))
        );
    }

    #[test]
    fn bare_output_flag_without_a_value_is_rejected() {
        let parsed = Args::try_parse_from(["jumpcut", "big fish.fountain", "-o"]);
        assert!(parsed.is_err(), "expected bare -o to be rejected");
    }
}
