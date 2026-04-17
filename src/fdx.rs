use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::{BTreeMap, HashSet};

use crate::parser::{is_cold_opening, is_end_act, is_new_act};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig};
use crate::{
    blank_attributes, Element, ElementText, ImportedAlignment, ImportedDialogueContinueds,
    ImportedElementKind, ImportedElementStyle, ImportedLayoutOverrides, ImportedMoresAndContinueds,
    ImportedPageLayoutOverrides, ImportedSceneContinueds, ImportedTitlePage,
    ImportedTitlePageAlignment, ImportedTitlePagePage, ImportedTitlePageParagraph,
    ImportedTitlePageHeaderFooter, ImportedTitlePageTabStop, ImportedTitlePageTabStopKind,
    Metadata, Screenplay, TextRun,
};

#[derive(Debug)]
pub struct FdxParseError(String);

impl std::fmt::Display for FdxParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for FdxParseError {}

pub fn parse_fdx(xml: &str) -> Result<Screenplay, FdxParseError> {
    let imported_settings = extract_import_settings(xml)?;
    let mut metadata = extract_import_metadata(&imported_settings);
    let (title_page_metadata, imported_title_page) = extract_title_page_data(xml)?;
    metadata.extend(title_page_metadata);

    let blocks = parse_blocks(xml)?;

    let mut used_paragraph_types = HashSet::new();
    for block in &blocks {
        if let FdxBlock::Paragraph(p) = block {
            used_paragraph_types.insert(p.paragraph_type.clone());
        }
    }

    let imported_layout =
        imported_settings_to_layout_overrides(&imported_settings, &used_paragraph_types);
    let elements = group_dialogue_blocks(blocks.into_iter().filter_map(block_to_element).collect());

    let mut screenplay = Screenplay {
        metadata,
        imported_layout,
        imported_title_page,
        elements,
    };
    screenplay.apply_structural_act_break_policy();
    Ok(screenplay)
}

#[derive(Debug)]
struct FdxParagraph {
    paragraph_type: String,
    alignment: Option<String>,
    starts_new_page: bool,
    number: Option<String>,
    text: ElementText,
}

#[derive(Debug)]
enum FdxBlock {
    Paragraph(FdxParagraph),
    DualDialogue(Vec<FdxParagraph>),
}

#[derive(Debug)]
struct TextChunk {
    content: String,
    styles: HashSet<String>,
}

#[derive(Clone, Debug, Default)]
struct ImportedFdxSettings {
    page_width: Option<f32>,
    page_height: Option<f32>,
    top_margin: Option<f32>,
    bottom_margin: Option<f32>,
    header_margin: Option<f32>,
    footer_margin: Option<f32>,
    mores_and_continueds: ImportedMoresAndContinueds,
    paragraph_styles: BTreeMap<String, ImportedParagraphStyle>,
}

#[derive(Clone, Debug, Default)]
struct ImportedParagraphStyle {
    first_indent: Option<f32>,
    left_indent: Option<f32>,
    right_indent: Option<f32>,
    space_before: Option<f32>,
    spacing: Option<f32>,
    alignment: Option<ImportedAlignment>,
    starts_new_page: Option<bool>,
    underline: Option<bool>,
    bold: Option<bool>,
    italic: Option<bool>,
}

#[derive(Debug)]
struct FdxTitlePageParagraph {
    alignment: Option<String>,
    left_indent: Option<f64>,
    space_before: Option<f64>,
    starts_new_page: bool,
    tab_stops: Vec<ImportedTitlePageTabStop>,
    text: ElementText,
    adornment_styles: HashSet<String>,
}

fn parse_blocks(xml: &str) -> Result<Vec<FdxBlock>, FdxParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut in_content = false;
    let mut paragraph_depth = 0usize;
    let mut in_text = false;
    let mut in_dual_dialogue = false;

    let mut paragraph_type = None;
    let mut paragraph_alignment = None;
    let mut paragraph_starts_new_page = false;
    let mut paragraph_number = None;
    let mut text_chunks: Vec<TextChunk> = Vec::new();
    let mut text_styles: HashSet<String> = HashSet::new();
    let mut blocks = Vec::new();
    let mut dual_dialogue_paragraphs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"Content" if paragraph_depth == 0 => {
                    in_content = true;
                }
                b"Paragraph" if in_content => {
                    paragraph_depth += 1;
                    if is_active_paragraph(paragraph_depth, in_dual_dialogue) {
                        begin_paragraph(
                            &reader,
                            &event,
                            &mut paragraph_type,
                            &mut paragraph_alignment,
                            &mut paragraph_starts_new_page,
                            &mut paragraph_number,
                            &mut text_chunks,
                        )?;
                    }
                }
                b"DualDialogue" if in_content => {
                    in_dual_dialogue = true;
                    dual_dialogue_paragraphs.clear();
                }
                b"Text" if paragraph_depth > 0 => {
                    in_text = true;
                    text_styles = parse_style_names(optional_attr(&reader, &event, b"Style")?);
                }
                _ => {}
            },
            Ok(Event::Empty(event)) => match event.name().as_ref() {
                b"Paragraph" if in_content => {
                    let paragraph = FdxParagraph {
                        paragraph_type: required_attr(&reader, &event, b"Type")?
                            .unwrap_or_default(),
                        alignment: optional_attr(&reader, &event, b"Alignment")?,
                        starts_new_page: optional_attr(&reader, &event, b"StartsNewPage")?
                            .as_deref()
                            == Some("Yes"),
                        number: optional_attr(&reader, &event, b"Number")?,
                        text: ElementText::Plain(String::new()),
                    };
                    if in_dual_dialogue {
                        dual_dialogue_paragraphs.push(paragraph);
                    } else {
                        blocks.push(FdxBlock::Paragraph(paragraph));
                    }
                }
                b"Text" if paragraph_depth > 0 => {
                    text_chunks.push(TextChunk {
                        content: String::new(),
                        styles: parse_style_names(optional_attr(&reader, &event, b"Style")?),
                    });
                }
                _ => {}
            },
            Ok(Event::Text(event)) if in_text => {
                let decoded = event
                    .decode()
                    .map_err(|err| FdxParseError(err.to_string()))?;
                text_chunks.push(TextChunk {
                    content: unescape(&decoded)
                        .map_err(|err| FdxParseError(err.to_string()))?
                        .into_owned(),
                    styles: text_styles.clone(),
                });
            }
            Ok(Event::GeneralRef(event)) if in_text => {
                let decoded = event
                    .decode()
                    .map_err(|err| FdxParseError(err.to_string()))?;
                let entity = format!("&{decoded};");
                text_chunks.push(TextChunk {
                    content: unescape(&entity)
                        .map_err(|err| FdxParseError(err.to_string()))?
                        .into_owned(),
                    styles: text_styles.clone(),
                });
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"Text" => {
                    in_text = false;
                }
                b"Paragraph" if paragraph_depth > 0 => {
                    if is_active_paragraph(paragraph_depth, in_dual_dialogue) {
                        let paragraph = FdxParagraph {
                            paragraph_type: paragraph_type.take().unwrap_or_default(),
                            alignment: paragraph_alignment.take(),
                            starts_new_page: paragraph_starts_new_page,
                            number: paragraph_number.take(),
                            text: collapse_text_chunks(std::mem::take(&mut text_chunks)),
                        };
                        if in_dual_dialogue {
                            dual_dialogue_paragraphs.push(paragraph);
                        } else {
                            blocks.push(FdxBlock::Paragraph(paragraph));
                        }
                        paragraph_starts_new_page = false;
                    }
                    paragraph_depth -= 1;
                }
                b"DualDialogue" if in_dual_dialogue => {
                    blocks.push(FdxBlock::DualDialogue(std::mem::take(
                        &mut dual_dialogue_paragraphs,
                    )));
                    in_dual_dialogue = false;
                }
                b"Content" if in_content => {
                    in_content = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => return Err(FdxParseError(err.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(blocks)
}

fn begin_paragraph(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    paragraph_type: &mut Option<String>,
    paragraph_alignment: &mut Option<String>,
    paragraph_starts_new_page: &mut bool,
    paragraph_number: &mut Option<String>,
    text_chunks: &mut Vec<TextChunk>,
) -> Result<(), FdxParseError> {
    *paragraph_type = required_attr(reader, event, b"Type")?;
    *paragraph_alignment = optional_attr(reader, event, b"Alignment")?;
    *paragraph_starts_new_page =
        optional_attr(reader, event, b"StartsNewPage")?.as_deref() == Some("Yes");
    *paragraph_number = optional_attr(reader, event, b"Number")?;
    text_chunks.clear();
    Ok(())
}

fn is_active_paragraph(paragraph_depth: usize, in_dual_dialogue: bool) -> bool {
    (!in_dual_dialogue && paragraph_depth == 1) || (in_dual_dialogue && paragraph_depth == 2)
}

fn block_to_element(block: FdxBlock) -> Option<Element> {
    match block {
        FdxBlock::Paragraph(paragraph) => paragraph_to_element(paragraph),
        FdxBlock::DualDialogue(paragraphs) => {
            let dialogue_blocks = group_dialogue_blocks(
                paragraphs
                    .into_iter()
                    .filter_map(paragraph_to_element)
                    .collect(),
            )
            .into_iter()
            .filter(|element| matches!(element, Element::DialogueBlock(_)))
            .collect::<Vec<_>>();

            if dialogue_blocks.is_empty() {
                None
            } else {
                Some(Element::DualDialogueBlock(dialogue_blocks))
            }
        }
    }
}

fn required_attr(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, FdxParseError> {
    optional_attr(reader, event, name)
}

fn optional_attr(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, FdxParseError> {
    for attr in event.attributes() {
        let attr = attr.map_err(|err| FdxParseError(err.to_string()))?;
        if attr.key.as_ref() == name {
            return attr
                .decode_and_unescape_value(reader.decoder())
                .map(|value| Some(value.into_owned()))
                .map_err(|err| FdxParseError(err.to_string()));
        }
    }
    Ok(None)
}

fn parse_style_names(style: Option<String>) -> HashSet<String> {
    style
        .unwrap_or_default()
        .split('+')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn extract_import_metadata(settings: &ImportedFdxSettings) -> Metadata {
    let fmt = normalize_settings_to_fmt(settings);
    let mut metadata = Metadata::new();
    if !fmt.is_empty() {
        metadata.insert("fmt".into(), vec![fmt.into()]);
    }
    metadata
}

fn extract_title_page_data(
    xml: &str,
) -> Result<(Metadata, Option<ImportedTitlePage>), FdxParseError> {
    let header_footer = parse_title_page_header_footer(xml)?;
    let paragraphs = parse_title_page_paragraphs(xml)?;
    if paragraphs.is_empty() {
        return Ok((Metadata::new(), None));
    }

    let imported_title_page = build_imported_title_page(&paragraphs, header_footer);
    let metadata = map_title_page_paragraphs_to_metadata(
        &paragraphs,
        imported_title_page
            .as_ref()
            .map(|title_page| title_page.pages.len())
            .unwrap_or(0),
    );

    Ok((metadata, imported_title_page))
}

fn parse_title_page_header_footer(xml: &str) -> Result<ImportedTitlePageHeaderFooter, FdxParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut in_title_page = false;
    let mut in_header_and_footer = false;
    let mut in_header = false;
    let mut header_has_page_number = false;
    let mut header_visible = false;
    let mut header_first_page = false;
    let mut starting_page = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"TitlePage" => in_title_page = true,
                b"HeaderAndFooter" if in_title_page => {
                    in_header_and_footer = true;
                    header_visible =
                        optional_attr(&reader, &event, b"HeaderVisible")?.as_deref() == Some("Yes");
                    header_first_page = optional_attr(&reader, &event, b"HeaderFirstPage")?
                        .as_deref()
                        == Some("Yes");
                    starting_page = optional_attr(&reader, &event, b"StartingPage")?
                        .and_then(|value| value.parse::<u32>().ok());
                }
                b"Header" if in_header_and_footer => in_header = true,
                b"DynamicLabel" if in_header => {
                    if optional_attr(&reader, &event, b"Type")?.as_deref() == Some("Page #") {
                        header_has_page_number = true;
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(event)) => {
                if event.name().as_ref() == b"DynamicLabel"
                    && in_header
                    && optional_attr(&reader, &event, b"Type")?.as_deref() == Some("Page #")
                {
                    header_has_page_number = true;
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"Header" => in_header = false,
                b"HeaderAndFooter" => in_header_and_footer = false,
                b"TitlePage" => break,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => return Err(FdxParseError(err.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(ImportedTitlePageHeaderFooter {
        header_visible,
        header_first_page,
        header_has_page_number,
        starting_page,
    })
}

fn parse_title_page_paragraphs(xml: &str) -> Result<Vec<FdxTitlePageParagraph>, FdxParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut in_title_page = false;
    let mut in_title_content = false;
    let mut paragraph_depth = 0usize;
    let mut in_text = false;

    let mut paragraph_alignment = None;
    let mut paragraph_left_indent: Option<f64> = None;
    let mut paragraph_space_before: Option<f64> = None;
    let mut paragraph_starts_new_page = false;
    let mut paragraph_tab_stops: Vec<ImportedTitlePageTabStop> = Vec::new();
    let mut text_chunks: Vec<TextChunk> = Vec::new();
    let mut text_styles: HashSet<String> = HashSet::new();
    let mut adornment_styles: HashSet<String> = HashSet::new();
    let mut paragraphs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"TitlePage" => in_title_page = true,
                b"Content" if in_title_page && paragraph_depth == 0 => in_title_content = true,
                b"Paragraph" if in_title_content => {
                    paragraph_depth += 1;
                    if paragraph_depth == 1 {
                        paragraph_alignment = optional_attr(&reader, &event, b"Alignment")?;
                        paragraph_left_indent = optional_attr(&reader, &event, b"LeftIndent")?
                            .and_then(|value| value.parse::<f64>().ok());
                        paragraph_space_before = optional_attr(&reader, &event, b"SpaceBefore")?
                            .and_then(|value| value.parse::<f64>().ok());
                        paragraph_starts_new_page = optional_attr(&reader, &event, b"StartsNewPage")?
                            .as_deref()
                            == Some("Yes");
                        paragraph_tab_stops.clear();
                        text_chunks.clear();
                        adornment_styles.clear();
                    }
                }
                b"Tabstop" if in_title_content && paragraph_depth > 0 => {
                    if let Some(tab_stop) = parse_title_page_tab_stop(&reader, &event)? {
                        paragraph_tab_stops.push(tab_stop);
                    }
                }
                b"Text" if in_title_content && paragraph_depth > 0 => {
                    in_text = true;
                    text_styles = parse_style_names(optional_attr(&reader, &event, b"Style")?);
                    if let Some(adornment) = optional_attr(&reader, &event, b"AdornmentStyle")? {
                        adornment_styles.insert(adornment);
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(event)) => match event.name().as_ref() {
                b"Text" if in_title_content && paragraph_depth > 0 => {
                    let styles = parse_style_names(optional_attr(&reader, &event, b"Style")?);
                    if let Some(adornment) = optional_attr(&reader, &event, b"AdornmentStyle")? {
                        adornment_styles.insert(adornment);
                    }
                    text_chunks.push(TextChunk {
                        content: String::new(),
                        styles,
                    });
                }
                b"Tabstop" if in_title_content && paragraph_depth > 0 => {
                    if let Some(tab_stop) = parse_title_page_tab_stop(&reader, &event)? {
                        paragraph_tab_stops.push(tab_stop);
                    }
                }
                _ => {}
            },
            Ok(Event::Text(event)) if in_text => {
                let decoded = event
                    .decode()
                    .map_err(|err| FdxParseError(err.to_string()))?;
                text_chunks.push(TextChunk {
                    content: unescape(&decoded)
                        .map_err(|err| FdxParseError(err.to_string()))?
                        .into_owned(),
                    styles: text_styles.clone(),
                });
            }
            Ok(Event::GeneralRef(event)) if in_text => {
                let decoded = event
                    .decode()
                    .map_err(|err| FdxParseError(err.to_string()))?;
                let entity = format!("&{decoded};");
                text_chunks.push(TextChunk {
                    content: unescape(&entity)
                        .map_err(|err| FdxParseError(err.to_string()))?
                        .into_owned(),
                    styles: text_styles.clone(),
                });
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"Text" => in_text = false,
                b"Paragraph" if in_title_content && paragraph_depth > 0 => {
                    if paragraph_depth == 1 {
                        paragraphs.push(FdxTitlePageParagraph {
                            alignment: paragraph_alignment.take(),
                            left_indent: paragraph_left_indent.take(),
                            space_before: paragraph_space_before.take(),
                            starts_new_page: paragraph_starts_new_page,
                            tab_stops: std::mem::take(&mut paragraph_tab_stops),
                            text: collapse_text_chunks(std::mem::take(&mut text_chunks)),
                            adornment_styles: std::mem::take(&mut adornment_styles),
                        });
                    }
                    paragraph_depth -= 1;
                }
                b"Content" if in_title_content => in_title_content = false,
                b"TitlePage" if in_title_page => in_title_page = false,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => return Err(FdxParseError(err.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(paragraphs)
}

fn map_title_page_paragraphs_to_metadata(
    paragraphs: &[FdxTitlePageParagraph],
    page_count: usize,
) -> Metadata {
    let mut metadata = Metadata::new();
    if paragraphs.is_empty() {
        return metadata;
    }

    let page_ranges = title_page_page_ranges(paragraphs);
    let page_one_end = page_ranges
        .get(1)
        .map(|(start, _)| *start)
        .unwrap_or(paragraphs.len());
    let page_one = &paragraphs[..page_one_end];
    let center_groups = centered_title_page_groups(page_one);
    if let Some(group) = center_groups.first() {
        metadata.insert("title".into(), group.lines.clone());
    }
    if center_groups.len() > 1 {
        let remaining = &center_groups[1..];
        let source_index = remaining
            .iter()
            .rposition(|group| group.is_source_like)
            .or_else(|| (remaining.len() >= 3).then_some(remaining.len() - 1));
        let middle = match source_index {
            Some(index) => {
                metadata.insert("source".into(), remaining[index].lines.clone());
                &remaining[..index]
            }
            None => remaining,
        };

        match middle {
            [only] => {
                if let Some((credit, author_lines)) = split_credit_and_author_lines(&only.lines) {
                    metadata.insert("credit".into(), vec![credit]);
                    insert_author_metadata(&mut metadata, author_lines);
                } else {
                    insert_author_metadata(&mut metadata, only.lines.clone());
                }
            }
            [credit, authors @ ..] if !authors.is_empty() => {
                metadata.insert("credit".into(), credit.lines.clone());
                let author_lines = authors
                    .iter()
                    .flat_map(|group| group.lines.clone())
                    .collect::<Vec<_>>();
                insert_author_metadata(&mut metadata, author_lines);
            }
            _ => {}
        }
    }

    let bottom_start = page_one
        .iter()
        .rposition(|paragraph| paragraph.alignment.as_deref() == Some("Center"))
        .map(|index| index + 1)
        .unwrap_or(0);
    let mut contact_lines = Vec::new();
    let mut right_lines = Vec::new();
    let mut frontmatter_lines: Vec<ElementText> = Vec::new();
    for paragraph in &page_one[bottom_start..] {
        match paragraph.alignment.as_deref() {
            Some("Left") | Some("Full") => {
                // Paragraphs with action-width left indent (>= 1.50) that contain non-blank
                // text are frontmatter, not contact/draft info.
                let is_action_indent = paragraph
                    .left_indent
                    .map(|indent| indent >= 1.50 - f64::EPSILON)
                    .unwrap_or(false);
                if is_action_indent && !element_text_is_blank(&paragraph.text) {
                    frontmatter_lines.push(paragraph.text.clone());
                } else {
                    let (left, right) = split_title_page_bottom_columns(&paragraph.text);
                    if let Some(left) = left.filter(|value| !element_text_is_blank(value)) {
                        contact_lines.push(left);
                    }
                    if let Some(right) = right.filter(|value| !element_text_is_blank(value)) {
                        right_lines.push(right);
                    }
                }
            }
            Some("Right") => {
                if !element_text_is_blank(&paragraph.text) {
                    right_lines.push(paragraph.text.clone());
                }
            }
            _ => {}
        }
    }

    if page_count > 1 {
        if !frontmatter_lines.is_empty() {
            frontmatter_lines.push("===".into());
        }

        let mut first_overflow_page = true;
        for page in build_imported_title_page(paragraphs, ImportedTitlePageHeaderFooter::default())
            .into_iter()
            .flat_map(|title_page| title_page.pages.into_iter().enumerate())
        {
            let (page_index, page) = page;
            if page_index == 0 {
                continue;
            }
            if !first_overflow_page {
                frontmatter_lines.push("===".into());
            }
            first_overflow_page = false;
            for paragraph in page.paragraphs {
                let text = match paragraph.alignment {
                    ImportedTitlePageAlignment::Center => {
                        ElementText::Plain(format!("> {} <", paragraph.text.plain_text()))
                    }
                    _ => paragraph.text,
                };
                if !element_text_is_blank(&text) {
                    frontmatter_lines.push(text);
                }
            }
        }
    }

    if !contact_lines.is_empty() {
        metadata.insert("contact".into(), contact_lines);
    }
    if !right_lines.is_empty() {
        if right_lines.len() == 1 && looks_like_draft_date(&right_lines[0].plain_text()) {
            metadata.insert("draft date".into(), right_lines);
        } else if right_lines.len() > 1
            && looks_like_draft_date(&right_lines.last().unwrap().plain_text())
        {
            let draft_date = vec![right_lines.pop().unwrap()];
            metadata.insert("draft".into(), right_lines);
            metadata.insert("draft date".into(), draft_date);
        } else {
            metadata.insert("draft".into(), right_lines);
        }
    }
    if !frontmatter_lines.is_empty() {
        metadata.insert("frontmatter".into(), frontmatter_lines);
    }

    metadata
}

#[derive(Clone)]
struct TitlePageGroup {
    lines: Vec<ElementText>,
    is_source_like: bool,
}

fn split_credit_and_author_lines(lines: &[ElementText]) -> Option<(ElementText, Vec<ElementText>)> {
    let first = lines.first()?;
    if !is_credit_like(&first.plain_text()) || lines.len() < 2 {
        return None;
    }
    Some((first.clone(), lines[1..].to_vec()))
}

fn insert_author_metadata(metadata: &mut Metadata, author_lines: Vec<ElementText>) {
    if author_lines.is_empty() {
        return;
    }
    if author_lines.len() == 1 {
        metadata.insert("author".into(), author_lines);
    } else {
        metadata.insert("authors".into(), author_lines);
    }
}

fn is_credit_like(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "written by" | "by" | "screenplay by" | "story by" | "teleplay by" | "adapted by"
    )
}

fn centered_title_page_groups(paragraphs: &[FdxTitlePageParagraph]) -> Vec<TitlePageGroup> {
    let mut groups = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_source_like = false;

    for paragraph in paragraphs {
        if paragraph.alignment.as_deref() == Some("Center")
            && !element_text_is_blank(&paragraph.text)
        {
            current_source_like |= paragraph.adornment_styles.contains("-1");
            current_lines.push(paragraph.text.clone());
            continue;
        }

        if !current_lines.is_empty() {
            groups.push(TitlePageGroup {
                lines: std::mem::take(&mut current_lines),
                is_source_like: current_source_like,
            });
            current_source_like = false;
        }
    }

    if !current_lines.is_empty() {
        groups.push(TitlePageGroup {
            lines: current_lines,
            is_source_like: current_source_like,
        });
    }

    groups
}

fn build_imported_title_page(
    paragraphs: &[FdxTitlePageParagraph],
    header_footer: ImportedTitlePageHeaderFooter,
) -> Option<ImportedTitlePage> {
    if paragraphs.is_empty() {
        return None;
    }

    let pages = title_page_page_ranges(paragraphs)
        .into_iter()
        .map(|(start, end)| imported_title_page_page_from_paragraphs(&paragraphs[start..end]))
        .filter(|page| !page.paragraphs.is_empty())
        .collect::<Vec<_>>();

    (!pages.is_empty()).then_some(ImportedTitlePage { header_footer, pages })
}

fn imported_title_page_page_from_paragraphs(
    paragraphs: &[FdxTitlePageParagraph],
) -> ImportedTitlePagePage {
    ImportedTitlePagePage {
        paragraphs: paragraphs
            .iter()
            .map(|paragraph| ImportedTitlePageParagraph {
                text: paragraph.text.clone(),
                alignment: imported_title_page_alignment(paragraph.alignment.as_deref()),
                left_indent: paragraph.left_indent.map(|value| value as f32),
                space_before: paragraph.space_before.map(|value| value as f32),
                tab_stops: paragraph.tab_stops.clone(),
            })
            .collect(),
    }
}

fn title_page_page_ranges(paragraphs: &[FdxTitlePageParagraph]) -> Vec<(usize, usize)> {
    const TITLE_PAGE_LINES_PER_PAGE: usize = 54;

    if paragraphs.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut page_start = 0usize;
    let mut used_lines = 0usize;

    for (index, paragraph) in paragraphs.iter().enumerate() {
        let paragraph_lines = title_page_paragraph_total_lines(paragraph);
        if index > page_start && paragraph.starts_new_page {
            ranges.push((page_start, index));
            page_start = index;
            used_lines = 0;
        } else if index > page_start
            && used_lines > 0
            && used_lines + paragraph_lines > TITLE_PAGE_LINES_PER_PAGE
        {
            ranges.push((page_start, index));
            page_start = index;
            used_lines = 0;
        }
        used_lines += paragraph_lines;
    }

    ranges.push((page_start, paragraphs.len()));
    ranges
}

fn title_page_paragraph_total_lines(paragraph: &FdxTitlePageParagraph) -> usize {
    title_page_space_before_lines(paragraph.space_before)
        + title_page_wrapped_line_count(paragraph)
}

fn title_page_space_before_lines(space_before: Option<f64>) -> usize {
    (space_before.unwrap_or(0.0) / 12.0)
        .round()
        .max(0.0) as usize
}

fn title_page_wrapped_line_count(paragraph: &FdxTitlePageParagraph) -> usize {
    if element_text_is_blank(&paragraph.text) {
        return 1;
    }

    let left = paragraph.left_indent.unwrap_or(match paragraph.alignment.as_deref() {
        Some("Center") | Some("Right") | Some("Full") => 1.0,
        _ => 1.5,
    }) as f32;
    let width_chars = (((7.5 - left) * 72.0) / 7.0).floor().max(1.0) as usize;
    wrap_text_for_element(
        &paragraph.text.plain_text(),
        &WrapConfig::with_exact_width_chars(width_chars),
    )
    .len()
    .max(1)
}

fn parse_title_page_tab_stop(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<ImportedTitlePageTabStop>, FdxParseError> {
    let Some(position) = optional_attr(reader, event, b"Position")?
        .and_then(|value| value.parse::<f32>().ok())
    else {
        return Ok(None);
    };

    let kind = match optional_attr(reader, event, b"Type")?.as_deref() {
        Some("Center") => ImportedTitlePageTabStopKind::Center,
        Some("Right") => ImportedTitlePageTabStopKind::Right,
        _ => ImportedTitlePageTabStopKind::Left,
    };

    Ok(Some(ImportedTitlePageTabStop { position, kind }))
}

fn imported_title_page_alignment(alignment: Option<&str>) -> ImportedTitlePageAlignment {
    match alignment {
        Some("Center") => ImportedTitlePageAlignment::Center,
        Some("Right") => ImportedTitlePageAlignment::Right,
        Some("Full") => ImportedTitlePageAlignment::Full,
        _ => ImportedTitlePageAlignment::Left,
    }
}

fn element_text_is_blank(text: &ElementText) -> bool {
    text.plain_text().trim().is_empty()
}

fn split_title_page_bottom_columns(
    text: &ElementText,
) -> (Option<ElementText>, Option<ElementText>) {
    if !text.plain_text().contains('\t') {
        return (Some(text.clone()), None);
    }

    match text {
        ElementText::Plain(value) => {
            let mut parts = value.split('\t');
            let left = parts.next().unwrap_or_default().to_string();
            let right = parts
                .filter(|part| !part.is_empty())
                .next_back()
                .unwrap_or_default()
                .to_string();
            (
                (!left.is_empty()).then_some(ElementText::Plain(left)),
                (!right.is_empty()).then_some(ElementText::Plain(right)),
            )
        }
        ElementText::Styled(runs) => {
            let mut seen_tab = false;
            let mut left_runs = Vec::new();
            let mut right_runs = Vec::new();

            for run in runs {
                let mut left = String::new();
                let mut right = String::new();
                for ch in run.content.chars() {
                    if ch == '\t' {
                        seen_tab = true;
                        continue;
                    }
                    if seen_tab {
                        right.push(ch);
                    } else {
                        left.push(ch);
                    }
                }
                if !left.is_empty() {
                    left_runs.push(TextRun {
                        content: left,
                        text_style: run.text_style.clone(),
                    });
                }
                if !right.is_empty() {
                    right_runs.push(TextRun {
                        content: right,
                        text_style: run.text_style.clone(),
                    });
                }
            }

            (
                (!left_runs.is_empty()).then_some(ElementText::Styled(left_runs)),
                (!right_runs.is_empty()).then_some(ElementText::Styled(right_runs)),
            )
        }
    }
}

fn looks_like_draft_date(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }

    let has_digit = value.chars().any(|ch| ch.is_ascii_digit());
    let has_date_separator = value.contains('/') || value.contains('-') || value.contains(',');
    let has_month_name = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "sept", "oct", "nov", "dec",
    ]
    .into_iter()
    .any(|month| value.to_ascii_lowercase().contains(month));

    has_digit && (has_date_separator || has_month_name)
}

fn extract_import_settings(xml: &str) -> Result<ImportedFdxSettings, FdxParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut settings = ImportedFdxSettings::default();
    let mut current_element_settings_type = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"PageLayout" => apply_page_layout_attrs(&reader, &event, &mut settings)?,
                b"PageSize" => apply_page_size_attrs(&reader, &event, &mut settings)?,
                b"ElementSettings" => {
                    current_element_settings_type = required_attr(&reader, &event, b"Type")?;
                }
                b"FontSpec" => {
                    if let Some(type_name) = current_element_settings_type.as_deref() {
                        apply_font_spec_attrs(
                            &reader,
                            &event,
                            settings
                                .paragraph_styles
                                .entry(type_name.to_string())
                                .or_insert_with(|| ImportedParagraphStyle::default()),
                        )?;
                    }
                }
                b"ParagraphSpec" => {
                    if let Some(type_name) = current_element_settings_type.as_deref() {
                        let paragraph_spec = parse_paragraph_spec(&reader, &event)?;
                        merge_paragraph_style(
                            settings
                                .paragraph_styles
                                .entry(type_name.to_string())
                                .or_insert_with(ImportedParagraphStyle::default),
                            paragraph_spec,
                        );
                    }
                }
                b"DialogueBreaks" => {
                    apply_dialogue_break_attrs(
                        &reader,
                        &event,
                        &mut settings.mores_and_continueds.dialogue,
                    )?;
                }
                b"SceneBreaks" => {
                    apply_scene_break_attrs(
                        &reader,
                        &event,
                        &mut settings.mores_and_continueds.scene,
                    )?;
                }
                _ => {}
            },
            Ok(Event::Empty(event)) => match event.name().as_ref() {
                b"PageLayout" => apply_page_layout_attrs(&reader, &event, &mut settings)?,
                b"PageSize" => apply_page_size_attrs(&reader, &event, &mut settings)?,
                b"FontSpec" => {
                    if let Some(type_name) = current_element_settings_type.as_deref() {
                        apply_font_spec_attrs(
                            &reader,
                            &event,
                            settings
                                .paragraph_styles
                                .entry(type_name.to_string())
                                .or_insert_with(|| ImportedParagraphStyle::default()),
                        )?;
                    }
                }
                b"ParagraphSpec" => {
                    if let Some(type_name) = current_element_settings_type.as_deref() {
                        let paragraph_spec = parse_paragraph_spec(&reader, &event)?;
                        merge_paragraph_style(
                            settings
                                .paragraph_styles
                                .entry(type_name.to_string())
                                .or_insert_with(ImportedParagraphStyle::default),
                            paragraph_spec,
                        );
                    }
                }
                b"DialogueBreaks" => apply_dialogue_break_attrs(
                    &reader,
                    &event,
                    &mut settings.mores_and_continueds.dialogue,
                )?,
                b"SceneBreaks" => apply_scene_break_attrs(
                    &reader,
                    &event,
                    &mut settings.mores_and_continueds.scene,
                )?,
                _ => {}
            },
            Ok(Event::End(event)) => {
                if event.name().as_ref() == b"ElementSettings" {
                    current_element_settings_type = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(FdxParseError(err.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(settings)
}

fn apply_page_layout_attrs(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    settings: &mut ImportedFdxSettings,
) -> Result<(), FdxParseError> {
    settings.page_width = parse_attr_f32(reader, event, b"Width")?;
    settings.page_height = parse_attr_f32(reader, event, b"Height")?;
    settings.top_margin = parse_attr_f32(reader, event, b"TopMargin")?.map(|value| value / 72.0);
    settings.bottom_margin =
        parse_attr_f32(reader, event, b"BottomMargin")?.map(|value| value / 72.0);
    settings.header_margin =
        parse_attr_f32(reader, event, b"HeaderMargin")?.map(|value| value / 72.0);
    settings.footer_margin =
        parse_attr_f32(reader, event, b"FooterMargin")?.map(|value| value / 72.0);
    Ok(())
}

fn apply_page_size_attrs(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    settings: &mut ImportedFdxSettings,
) -> Result<(), FdxParseError> {
    settings.page_width = parse_attr_f32(reader, event, b"Width")?;
    settings.page_height = parse_attr_f32(reader, event, b"Height")?;
    Ok(())
}

fn parse_paragraph_spec(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<ImportedParagraphStyle, FdxParseError> {
    Ok(ImportedParagraphStyle {
        first_indent: parse_attr_f32(reader, event, b"FirstIndent")?,
        left_indent: parse_attr_f32(reader, event, b"LeftIndent")?,
        right_indent: parse_attr_f32(reader, event, b"RightIndent")?,
        space_before: parse_attr_f32(reader, event, b"SpaceBefore")?.map(spacing_lines_from_points),
        spacing: parse_attr_f32(reader, event, b"Spacing")?,
        alignment: parse_alignment(reader, event, b"Alignment")?,
        starts_new_page: parse_yes_no_attr(reader, event, b"StartsNewPage")?,
        underline: None,
        bold: None,
        italic: None,
    })
}

fn merge_paragraph_style(target: &mut ImportedParagraphStyle, parsed: ImportedParagraphStyle) {
    if parsed.first_indent.is_some() {
        target.first_indent = parsed.first_indent;
    }
    if parsed.left_indent.is_some() {
        target.left_indent = parsed.left_indent;
    }
    if parsed.right_indent.is_some() {
        target.right_indent = parsed.right_indent;
    }
    if parsed.space_before.is_some() {
        target.space_before = parsed.space_before;
    }
    if parsed.spacing.is_some() {
        target.spacing = parsed.spacing;
    }
    if parsed.alignment.is_some() {
        target.alignment = parsed.alignment;
    }
    if parsed.starts_new_page.is_some() {
        target.starts_new_page = parsed.starts_new_page;
    }
}

fn apply_font_spec_attrs(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    style: &mut ImportedParagraphStyle,
) -> Result<(), FdxParseError> {
    let font_styles = parse_style_names(optional_attr(reader, event, b"Style")?);
    style.underline = Some(font_styles.contains("Underline"));
    style.bold = Some(font_styles.contains("Bold"));
    style.italic = Some(font_styles.contains("Italic"));
    Ok(())
}

fn apply_dialogue_break_attrs(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    continueds: &mut ImportedDialogueContinueds,
) -> Result<(), FdxParseError> {
    continueds.automatic_character_continueds =
        parse_yes_no_attr(reader, event, b"AutomaticCharacterContinueds")?;
    continueds.bottom_of_page = parse_yes_no_attr(reader, event, b"BottomOfPage")?;
    continueds.dialogue_bottom = optional_attr(reader, event, b"DialogueBottom")?;
    continueds.dialogue_top = optional_attr(reader, event, b"DialogueTop")?;
    continueds.top_of_next = parse_yes_no_attr(reader, event, b"TopOfNext")?;
    Ok(())
}

fn apply_scene_break_attrs(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    continueds: &mut ImportedSceneContinueds,
) -> Result<(), FdxParseError> {
    continueds.continued_number = parse_yes_no_attr(reader, event, b"ContinuedNumber")?;
    continueds.scene_bottom = optional_attr(reader, event, b"SceneBottom")?;
    continueds.bottom_of_page = parse_yes_no_attr(reader, event, b"SceneBottomOfPage")?;
    continueds.scene_top = optional_attr(reader, event, b"SceneTop")?;
    continueds.top_of_next = parse_yes_no_attr(reader, event, b"SceneTopOfNext")?;
    Ok(())
}

fn parse_attr_f32(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<f32>, FdxParseError> {
    optional_attr(reader, event, name)?
        .map(|value| {
            value
                .parse::<f32>()
                .map_err(|err| FdxParseError(err.to_string()))
        })
        .transpose()
}

fn parse_yes_no_attr(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<bool>, FdxParseError> {
    optional_attr(reader, event, name)?
        .map(|value| match value.as_str() {
            "Yes" => Ok(true),
            "No" => Ok(false),
            _ => Err(FdxParseError(format!(
                "expected Yes/No attribute for {}",
                String::from_utf8_lossy(name)
            ))),
        })
        .transpose()
}

fn parse_alignment(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<ImportedAlignment>, FdxParseError> {
    optional_attr(reader, event, name)?
        .map(|value| match value.as_str() {
            "Left" => Ok(ImportedAlignment::Left),
            "Center" => Ok(ImportedAlignment::Center),
            "Right" => Ok(ImportedAlignment::Right),
            _ => Err(FdxParseError(format!("unsupported alignment: {value}"))),
        })
        .transpose()
}

fn normalize_settings_to_fmt(settings: &ImportedFdxSettings) -> String {
    let mut tokens = Vec::new();

    if matches_a4_page(settings) {
        tokens.push("a4".to_string());
    }
    if matches_multicam_profile(settings) {
        tokens.push("multicam".to_string());
    }

    if let Some(style) = settings.paragraph_styles.get("Scene Heading") {
        if style
            .space_before
            .is_some_and(|scene_heading_spacing_before| {
                approx_eq(scene_heading_spacing_before, 1.0)
            })
        {
            tokens.push("ssbsh".to_string());
        }
    }

    if let Some(style) = settings.paragraph_styles.get("Dialogue") {
        if style.spacing.is_some_and(|spacing| approx_eq(spacing, 2.0)) {
            tokens.push("dsd".to_string());
        }
        if let Some(left_indent) = style.left_indent.filter(|value| !approx_eq(*value, 2.5)) {
            tokens.push(format_numeric_token("dl", left_indent));
        }
        if let Some(right_indent) = style.right_indent.filter(|value| !approx_eq(*value, 6.0)) {
            tokens.push(format_numeric_token("dr", right_indent));
        }
    }

    if let Some(margin) = settings.top_margin.filter(|value| !approx_eq(*value, 1.0)) {
        tokens.push(format_numeric_token("tm", margin));
    }
    if let Some(margin) = settings
        .bottom_margin
        .filter(|value| !approx_eq(*value, 1.0))
    {
        tokens.push(format_numeric_token("bm", margin));
    }
    if let Some(margin) = settings
        .header_margin
        .filter(|value| !approx_eq(*value, 0.5))
    {
        tokens.push(format_numeric_token("hm", margin));
    }
    if let Some(margin) = settings
        .footer_margin
        .filter(|value| !approx_eq(*value, 0.5))
    {
        tokens.push(format_numeric_token("fm", margin));
    }

    tokens.join(" ")
}

fn matches_a4_page(settings: &ImportedFdxSettings) -> bool {
    matches!(
        (settings.page_width, settings.page_height),
        (Some(width), Some(height)) if approx_eq(width, 8.26) && approx_eq(height, 11.69)
    )
}

fn matches_multicam_profile(settings: &ImportedFdxSettings) -> bool {
    let Some(dialogue) = settings.paragraph_styles.get("Dialogue") else {
        return false;
    };
    let Some(character) = settings.paragraph_styles.get("Character") else {
        return false;
    };
    let Some(parenthetical) = settings.paragraph_styles.get("Parenthetical") else {
        return false;
    };
    let Some(transition) = settings.paragraph_styles.get("Transition") else {
        return false;
    };

    dialogue.spacing.is_some_and(|value| approx_eq(value, 2.0))
        && dialogue
            .left_indent
            .is_some_and(|value| approx_eq(value, 2.25))
        && character
            .right_indent
            .is_some_and(|value| approx_eq(value, 6.25))
        && parenthetical
            .left_indent
            .is_some_and(|value| approx_eq(value, 2.75))
        && transition
            .right_indent
            .is_some_and(|value| approx_eq(value, 7.25))
}

fn format_numeric_token(prefix: &str, value: f32) -> String {
    let mut formatted = format!("{value:.2}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    format!("{prefix}-{formatted}")
}

fn spacing_lines_from_points(space_before_points: f32) -> f32 {
    space_before_points / 12.0
}

fn imported_settings_to_layout_overrides(
    settings: &ImportedFdxSettings,
    used_paragraph_types: &HashSet<String>,
) -> Option<ImportedLayoutOverrides> {
    let mut element_styles = std::collections::BTreeMap::new();
    for (name, style) in &settings.paragraph_styles {
        if let Some(kind) = imported_element_kind(name) {
            let entry = ImportedElementStyle {
                first_indent: style.first_indent,
                left_indent: style.left_indent,
                right_indent: style.right_indent,
                spacing_before: style.space_before,
                line_spacing: style.spacing,
                alignment: style.alignment,
                starts_new_page: style.starts_new_page,
                underline: style.underline,
                bold: style.bold,
                italic: style.italic,
            };

            if let Some(existing_name) = element_styles.get(&kind).map(|(n, _)| n) {
                let new_is_used = used_paragraph_types.contains(name);
                let existing_is_used = used_paragraph_types.contains(*existing_name);

                if new_is_used && !existing_is_used {
                    element_styles.insert(kind, (name, entry));
                } else if !new_is_used && !existing_is_used {
                    element_styles.insert(kind, (name, entry));
                }
            } else {
                element_styles.insert(kind, (name, entry));
            }
        }
    }

    let imported_layout = ImportedLayoutOverrides {
        page: ImportedPageLayoutOverrides {
            page_width: settings.page_width,
            page_height: settings.page_height,
            top_margin: settings.top_margin,
            bottom_margin: settings.bottom_margin,
            header_margin: settings.header_margin,
            footer_margin: settings.footer_margin,
        },
        element_styles: element_styles
            .into_iter()
            .map(|(kind, (_, style))| (kind, style))
            .collect(),
        mores_and_continueds: settings.mores_and_continueds.clone(),
    };

    if imported_layout.is_empty() {
        None
    } else {
        Some(imported_layout)
    }
}

fn imported_element_kind(name: &str) -> Option<ImportedElementKind> {
    match name {
        "Action" => Some(ImportedElementKind::Action),
        "Scene Heading" => Some(ImportedElementKind::SceneHeading),
        "Character" => Some(ImportedElementKind::Character),
        "Dialogue" => Some(ImportedElementKind::Dialogue),
        "Parenthetical" => Some(ImportedElementKind::Parenthetical),
        "Transition" => Some(ImportedElementKind::Transition),
        "Lyric" => Some(ImportedElementKind::Lyric),
        "Cold Opening" => Some(ImportedElementKind::ColdOpening),
        "New Act" => Some(ImportedElementKind::NewAct),
        "End of Act" | "End Of Act" => Some(ImportedElementKind::EndOfAct),
        _ => None,
    }
}

fn approx_eq(left: f32, right: f32) -> bool {
    (left - right).abs() < 0.01
}

fn collapse_text_chunks(chunks: Vec<TextChunk>) -> ElementText {
    if chunks.is_empty() {
        return ElementText::Plain(String::new());
    }

    if chunks.iter().all(|chunk| chunk.styles.is_empty()) {
        return ElementText::Plain(chunks.into_iter().map(|chunk| chunk.content).collect());
    }

    let mut runs: Vec<TextRun> = Vec::new();
    for chunk in chunks {
        if let Some(last) = runs.last_mut() {
            if last.text_style == chunk.styles {
                last.content.push_str(&chunk.content);
                continue;
            }
        }

        runs.push(TextRun {
            content: chunk.content,
            text_style: chunk.styles,
        });
    }

    ElementText::Styled(runs)
}

fn paragraph_to_element(paragraph: FdxParagraph) -> Option<Element> {
    let mut attributes = blank_attributes();
    let is_centered_type = matches!(
        paragraph.paragraph_type.as_str(),
        "Action" | "Cold Opening" | "New Act" | "End of Act" | "End Of Act"
    );
    let is_inherently_centered = matches!(
        paragraph.paragraph_type.as_str(),
        "Cold Opening" | "New Act" | "End of Act" | "End Of Act"
    );
    if is_inherently_centered
        || (paragraph.alignment.as_deref() == Some("Center") && is_centered_type)
    {
        attributes.centered = true;
    }
    attributes.starts_new_page = paragraph.starts_new_page;
    attributes.scene_number = paragraph.number;

    let text_plain = paragraph.text.plain_text();

    match paragraph.paragraph_type.as_str() {
        "Scene Heading" => Some(Element::SceneHeading(paragraph.text, attributes)),
        "Action" => {
            if attributes.centered {
                if is_end_act(&text_plain) {
                    Some(Element::EndOfAct(paragraph.text, attributes))
                } else if is_cold_opening(&text_plain) {
                    Some(Element::ColdOpening(paragraph.text, attributes))
                } else if is_new_act(&text_plain) {
                    Some(Element::NewAct(paragraph.text, attributes))
                } else {
                    Some(Element::Action(paragraph.text, attributes))
                }
            } else {
                Some(Element::Action(paragraph.text, attributes))
            }
        }
        "Character" => Some(Element::Character(paragraph.text, attributes)),
        "Dialogue" => Some(Element::Dialogue(paragraph.text, attributes)),
        "Parenthetical" => Some(Element::Parenthetical(paragraph.text, attributes)),
        "Transition" => Some(Element::Transition(paragraph.text, attributes)),
        "Lyric" => Some(Element::Lyric(paragraph.text, attributes)),
        "Cold Opening" => Some(Element::ColdOpening(paragraph.text, attributes)),
        "New Act" => {
            if is_end_act(&text_plain) {
                Some(Element::EndOfAct(paragraph.text, attributes))
            } else if is_cold_opening(&text_plain) {
                Some(Element::ColdOpening(paragraph.text, attributes))
            } else {
                Some(Element::NewAct(paragraph.text, attributes))
            }
        }
        "End of Act" | "End Of Act" => Some(Element::EndOfAct(paragraph.text, attributes)),
        _ => None,
    }
}

fn group_dialogue_blocks(elements: Vec<Element>) -> Vec<Element> {
    let mut grouped = Vec::new();
    let mut index = 0;

    while index < elements.len() {
        if matches!(elements[index], Element::Character(_, _)) {
            let mut block = vec![elements[index].clone()];
            index += 1;

            while index < elements.len()
                && matches!(
                    elements[index],
                    Element::Parenthetical(_, _) | Element::Dialogue(_, _) | Element::Lyric(_, _)
                )
            {
                block.push(elements[index].clone());
                index += 1;
            }

            if block.len() > 1 {
                grouped.push(Element::DialogueBlock(block));
            } else {
                grouped.extend(block);
            }
        } else {
            grouped.push(elements[index].clone());
            index += 1;
        }
    }

    grouped
}
