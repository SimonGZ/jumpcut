use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::parse;
use crate::pagination::{
    normalize_screenplay, wrapping::ElementType, DialoguePartKind, FlowKind, Fragment,
    LayoutGeometry, LineRange, NormalizedElement, PageBreakFixture, PaginationConfig,
};

pub fn write_big_fish_packet(debug_dir: &Path) {
    let report = build_line_break_parity_report(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
    );
    fs::create_dir_all(debug_dir).unwrap();
    fs::write(debug_dir.join("parity.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
    fs::write(debug_dir.join("REVIEW.md"), render_big_fish_review(&report)).unwrap();
}

pub fn write_little_women_packet(debug_dir: &Path) {
    let report = build_line_break_parity_report(
        "little-women",
        "tests/fixtures/corpus/public/little-women/source/source.fountain",
        "tests/fixtures/corpus/public/little-women/canonical/page-breaks.json",
    );
    fs::create_dir_all(debug_dir).unwrap();
    fs::write(debug_dir.join("parity.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
    fs::write(debug_dir.join("REVIEW.md"), render_little_women_review(&report)).unwrap();
}

pub fn write_mostly_genius_packet(debug_dir: &Path) {
    let report = build_line_break_parity_report(
        "mostly-genius",
        "tests/fixtures/corpus/public/mostly-genius/source/source.fountain",
        "tests/fixtures/corpus/public/mostly-genius/canonical/page-breaks.json",
    );
    fs::create_dir_all(debug_dir).unwrap();
    fs::write(debug_dir.join("parity.json"), serde_json::to_string_pretty(&report).unwrap()).unwrap();
    fs::write(debug_dir.join("REVIEW.md"), render_mostly_genius_review(&report)).unwrap();
}

pub fn build_line_break_parity_report(
    screenplay_id: &str,
    fountain_path: &str,
    canonical_page_breaks_path: &str,
) -> LineBreakParityReport {
    let measurement = measurement_for_screenplay(screenplay_id);
    let fountain = fs::read_to_string(fountain_path).unwrap();
    let screenplay = parse(&fountain);
    let parsed = normalize_screenplay(screenplay_id, &screenplay);
    let canonical: PageBreakFixture =
        serde_json::from_str(&fs::read_to_string(canonical_page_breaks_path).unwrap()).unwrap();
    let pdf_pages = public_pdf_pages(screenplay_id);
    let elements: HashMap<String, NormalizedElement> = parsed
        .elements
        .into_iter()
        .map(|element| (element.element_id.clone(), element))
        .collect();

    let mut items = Vec::new();
    let mut exact_unique_count = 0;
    let mut exact_ambiguous_count = 0;
    let mut unsupported_count = 0;
    let mut disagreement_count = 0;

    for page in &canonical.pages {
        let page_lines = pdf_pages.get(&page.number).map(Vec::as_slice).unwrap_or(&[]);
        for item in &page.items {
            let result = build_line_break_parity_item(
                page.number,
                &item.element_id,
                &item.kind,
                &item.fragment,
                item.line_range,
                &item.dual_dialogue_group,
                item.dual_dialogue_side,
                elements.get(&item.element_id),
                page_lines,
                &measurement,
            );

            match result.match_kind.as_str() {
                "exact_unique" => {
                    exact_unique_count += 1;
                    if result.lines_agree == Some(false) {
                        disagreement_count += 1;
                    }
                }
                "exact_ambiguous" => exact_ambiguous_count += 1,
                _ => unsupported_count += 1,
            }

            items.push(result);
        }
    }

    LineBreakParityReport {
        screenplay: screenplay_id.into(),
        exact_unique_count,
        exact_ambiguous_count,
        unsupported_count,
        disagreement_count,
        measurement: LineBreakParityMeasurement {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, &measurement),
                debug_flow_geometry("Scene Heading", "Scene Heading", FlowKind::SceneHeading, &measurement),
                debug_flow_geometry("Transition", "Transition", FlowKind::Transition, &measurement),
                debug_flow_geometry("Cold Opening", "Cold Opening", FlowKind::ColdOpening, &measurement),
                debug_flow_geometry("New Act", "New Act", FlowKind::NewAct, &measurement),
                debug_flow_geometry("End of Act", "End of Act", FlowKind::EndOfAct, &measurement),
                debug_flow_geometry("Section", "Action (fallback)", FlowKind::Section, &measurement),
                debug_flow_geometry("Synopsis", "Action (fallback)", FlowKind::Synopsis, &measurement),
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry("Dialogue", "Dialogue", DialoguePartKind::Dialogue, &measurement),
                debug_dialogue_geometry("Character", "Character", DialoguePartKind::Character, &measurement),
                debug_dialogue_geometry(
                    "Parenthetical",
                    "Parenthetical",
                    DialoguePartKind::Parenthetical,
                    &measurement,
                ),
                debug_dialogue_geometry("Lyric", "Lyric", DialoguePartKind::Lyric, &measurement),
            ],
        },
        items,
    }
}

fn build_line_break_parity_item(
    page_number: u32,
    element_id: &str,
    kind: &str,
    fragment: &Fragment,
    line_range: Option<LineRange>,
    dual_dialogue_group: &Option<String>,
    dual_dialogue_side: Option<u8>,
    element: Option<&NormalizedElement>,
    page_lines: &[String],
    measurement: &LayoutGeometry,
) -> LineBreakParityItem {
    let Some(element) = element else {
        return LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: None,
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: None,
            expected_wrapped_lines: Vec::new(),
            match_kind: "missing-element".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        };
    };

    let Some(candidate_text) = canonical_pdf_text_for_item(fragment, line_range, element) else {
        return LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&element.text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: None,
            expected_wrapped_lines: Vec::new(),
            match_kind: "unsupported-fragment".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        };
    };

    let element_type = ElementType::from_item_kind(kind, dual_dialogue_side);
    let config = crate::pagination::wrapping::WrapConfig::from_geometry(measurement, element_type);
    let width_chars = config.exact_width_chars;
    let expected_wrapped_lines = crate::pagination::wrapping::wrap_text_for_element(&candidate_text, &config)
        .into_iter()
        .map(|line| normalize_pdf_match_text(&line))
        .collect::<Vec<_>>();
    let normalized_text = normalize_pdf_match_text(&candidate_text);
    let matches = exact_pdf_line_matches(page_lines, &normalized_text);

    match matches.as_slice() {
        [(start, end)] => {
            let pdf_lines = page_lines[*start as usize - 1..*end as usize]
                .iter()
                .map(|line| normalize_pdf_match_text(line))
                .collect::<Vec<_>>();
            let lines_agree = expected_wrapped_lines == pdf_lines;

            LineBreakParityItem {
                page_number,
                element_id: element_id.into(),
                kind: kind.into(),
                text_preview: Some(text_preview(&candidate_text)),
                dual_dialogue_group: dual_dialogue_group.clone(),
                dual_dialogue_side,
                width_chars: Some(width_chars),
                expected_wrapped_lines,
                match_kind: "exact_unique".into(),
                pdf_line_count: Some(end - start + 1),
                pdf_line_span: Some((*start, *end)),
                pdf_lines,
                candidate_spans: vec![(*start, *end)],
                lines_agree: Some(lines_agree),
            }
        }
        [] => LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&candidate_text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: Some(width_chars),
            expected_wrapped_lines,
            match_kind: "unmatched".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        },
        _ => LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&candidate_text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: Some(width_chars),
            expected_wrapped_lines,
            match_kind: "exact_ambiguous".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: matches,
            lines_agree: None,
        },
    }
}

fn render_big_fish_review(report: &LineBreakParityReport) -> String {
    let mut review = render_review_common("Big Fish", "big-fish", report);
    review.push_str("\nIf you only inspect one example, search for `el-00787` in `parity.json`.\n");
    review
}

fn render_little_women_review(report: &LineBreakParityReport) -> String {
    let disagreements: Vec<&LineBreakParityItem> = report.items.iter().filter(|item| item.lines_agree == Some(false)).collect();
    let mut review = render_review_common("Little Women", "little-women", report);
    if disagreements.is_empty() {
        review.push_str("\nNo exact-unique line-break disagreements were found.\n");
    } else {
        review.push_str("\nSearch for the listed `element_id` values in `parity.json` first.\n");
    }
    review
}

fn render_mostly_genius_review(report: &LineBreakParityReport) -> String {
    let disagreements: Vec<&LineBreakParityItem> =
        report.items.iter().filter(|item| item.lines_agree == Some(false)).collect();
    let act_markers = report
        .items
        .iter()
        .filter(|item| item.kind == "New Act" || item.kind == "End of Act" || item.kind == "Cold Opening")
        .count();
    let mut review = render_review_common("Mostly Genius", "mostly-genius", report);
    review.push_str(&format!(
        "\nMulticam-specific markers in this report: {act_markers}\n"
    ));
    if disagreements.is_empty() {
        review.push_str("\nNo exact-unique line-break disagreements were found.\n");
    } else {
        review.push_str("\nSearch for multicam dialogue blocks and act-marker elements first in `parity.json`.\n");
    }
    review
}

fn render_review_common(title: &str, slug: &str, report: &LineBreakParityReport) -> String {
    let disagreements: Vec<&LineBreakParityItem> = report.items.iter().filter(|item| item.lines_agree == Some(false)).collect();
    let mut review = format!(
        "# {title} Line-Break Parity Review\n\n\
Files in this packet:\n\n\
- `target/pagination-debug/{slug}-linebreak-parity/REVIEW.md`\n\
- `target/pagination-debug/{slug}-linebreak-parity/parity.json`\n\n\
Coverage summary:\n\n\
- exact unique items: {exact_unique}\n\
- exact ambiguous items: {exact_ambiguous}\n\
- unsupported/unmatched items: {unsupported}\n\
- exact-unique line disagreements: {disagreements}\n\n\
How to read `parity.json`:\n\n\
- `expected_wrapped_lines` are our current wrapped lines for the recoverable text fragment\n\
- `pdf_lines` are the exact PDF-extracted lines when the page match is unique\n\
- `lines_agree = false` means the text match was trustworthy but our wrapping disagreed with the PDF\n\
- `match_kind = exact_ambiguous` means the same text appears multiple times on that page; do not trust it as ground truth\n\
- `match_kind = unsupported-fragment` usually means a split dialogue fragment whose exact per-page text cannot be reconstructed from the canonical fixture alone\n\n\
Read these first:\n\n",
        exact_unique = report.exact_unique_count,
        exact_ambiguous = report.exact_ambiguous_count,
        unsupported = report.unsupported_count,
        disagreements = report.disagreement_count,
    );

    for item in disagreements.iter().take(10) {
        review.push_str(&format!(
            "- `{element_id}` page {page} `{kind}` width={width:?}\n  expected: {expected:?}\n  pdf: {pdf:?}\n",
            element_id = item.element_id,
            page = item.page_number,
            kind = item.kind,
            width = item.width_chars,
            expected = item.expected_wrapped_lines,
            pdf = item.pdf_lines,
        ));
    }

    review
}

fn measurement_for_screenplay(screenplay_id: &str) -> LayoutGeometry {
    let path = Path::new("tests/fixtures/corpus/public")
        .join(screenplay_id)
        .join("source/source.fountain");
    let fountain = fs::read_to_string(path).unwrap();
    let screenplay = parse(&fountain);
    PaginationConfig::from_screenplay(&screenplay, 54.0).geometry
}

fn debug_flow_geometry(label: &str, source_style: &str, kind: FlowKind, geometry: &LayoutGeometry) -> DebugGeometry {
    let element_type = ElementType::from_flow_kind(&kind);
    let (left_indent_in, right_indent_in) = match kind {
        FlowKind::Action | FlowKind::SceneHeading | FlowKind::Section | FlowKind::Synopsis => {
            (geometry.action_left, geometry.action_right)
        }
        FlowKind::Transition => (geometry.transition_left, geometry.transition_right),
        FlowKind::ColdOpening => (geometry.cold_opening_left, geometry.cold_opening_right),
        FlowKind::NewAct => (geometry.new_act_left, geometry.new_act_right),
        FlowKind::EndOfAct => (geometry.end_of_act_left, geometry.end_of_act_right),
    };
    DebugGeometry {
        kind: label.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: crate::pagination::margin::calculate_element_width(geometry, element_type),
    }
}

fn debug_dialogue_geometry(
    label: &str,
    source_style: &str,
    kind: DialoguePartKind,
    geometry: &LayoutGeometry,
) -> DebugGeometry {
    let element_type = ElementType::from_dialogue_part_kind(&kind);
    let (left_indent_in, right_indent_in) = match kind {
        DialoguePartKind::Character => (geometry.character_left, geometry.character_right),
        DialoguePartKind::Parenthetical => (geometry.parenthetical_left, geometry.parenthetical_right),
        DialoguePartKind::Lyric => (geometry.lyric_left, geometry.lyric_right),
        DialoguePartKind::Dialogue => (geometry.dialogue_left, geometry.dialogue_right),
    };
    DebugGeometry {
        kind: label.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: crate::pagination::margin::calculate_element_width(geometry, element_type),
    }
}

fn canonical_pdf_text_for_item(
    fragment: &Fragment,
    line_range: Option<LineRange>,
    element: &NormalizedElement,
) -> Option<String> {
    match (fragment, line_range) {
        (Fragment::Whole, None) => Some(element.text.clone()),
        (_, Some(LineRange(start, end))) => Some(slice_explicit_lines(&element.text, start, end)),
        _ => None,
    }
}

fn exact_pdf_line_matches(page_lines: &[String], candidate_text: &str) -> Vec<(u32, u32)> {
    let mut matches = Vec::new();
    for start in 0..page_lines.len() {
        let mut accumulated = String::new();
        for end in start..page_lines.len() {
            if !accumulated.is_empty() {
                accumulated.push(' ');
            }
            accumulated.push_str(&page_lines[end]);
            let normalized = normalize_pdf_match_text(&accumulated);
            if normalized == candidate_text {
                matches.push((start as u32 + 1, end as u32 + 1));
            }
            if normalized.len() > candidate_text.len() + 40 {
                break;
            }
        }
    }
    matches
}

fn normalize_pdf_match_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn public_pdf_pages(screenplay_id: &str) -> HashMap<u32, Vec<String>> {
    let path = Path::new("tests/fixtures/corpus/public")
        .join(screenplay_id)
        .join("extracted/pdf-pages.json");
    let pdf_pages: PublicPdfPages = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    pdf_pages
        .pages
        .into_iter()
        .map(|page| (page.number, page.text.lines().map(str::to_string).collect()))
        .collect()
}

fn text_preview(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(80).collect()
}

fn slice_explicit_lines(text: &str, start: u32, end: u32) -> String {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index as u32 + 1;
            (line_no >= start && line_no <= end).then_some(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Serialize)]
pub struct LineBreakParityReport {
    pub screenplay: String,
    pub exact_unique_count: usize,
    pub exact_ambiguous_count: usize,
    pub unsupported_count: usize,
    pub disagreement_count: usize,
    pub measurement: LineBreakParityMeasurement,
    pub items: Vec<LineBreakParityItem>,
}

#[derive(Serialize)]
pub struct LineBreakParityMeasurement {
    pub flow_geometries: Vec<DebugGeometry>,
    pub dialogue_geometries: Vec<DebugGeometry>,
}

#[derive(Serialize)]
pub struct LineBreakParityItem {
    pub page_number: u32,
    pub element_id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dual_dialogue_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dual_dialogue_side: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_chars: Option<usize>,
    pub expected_wrapped_lines: Vec<String>,
    pub match_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_line_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_line_span: Option<(u32, u32)>,
    pub pdf_lines: Vec<String>,
    pub candidate_spans: Vec<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_agree: Option<bool>,
}

#[derive(Serialize)]
pub struct DebugGeometry {
    pub kind: String,
    pub source_style: String,
    pub left_indent_in: f32,
    pub right_indent_in: f32,
    pub width_chars: usize,
}

#[derive(Deserialize)]
struct PublicPdfPages {
    pages: Vec<PublicPdfPage>,
}

#[derive(Deserialize)]
struct PublicPdfPage {
    number: u32,
    text: String,
}
