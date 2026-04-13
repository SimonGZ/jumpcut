use crate::title_page::plain_title_uses_all_caps;
use crate::{Attributes, Element, ElementText, Metadata, Screenplay, TextRun};

const TITLE_PAGE_KEYS_IN_ORDER: &[&str] = &[
    "title",
    "credit",
    "author",
    "authors",
    "source",
    "draft",
    "draft date",
    "contact",
];

pub fn render(screenplay: &Screenplay) -> String {
    let mut paragraphs = Vec::new();

    let metadata_block = render_metadata(&screenplay.metadata);
    if !metadata_block.is_empty() {
        paragraphs.push(metadata_block);
    }

    for element in &screenplay.elements {
        paragraphs.extend(render_element_with_page_breaks(element));
    }

    paragraphs.join("\n\n")
}

fn render_metadata(metadata: &Metadata) -> String {
    let mut lines = Vec::new();

    for key in TITLE_PAGE_KEYS_IN_ORDER {
        if let Some(values) = metadata.get(*key) {
            lines.push(render_metadata_entry(key, values, metadata));
        }
    }

    let mut remaining_keys = metadata
        .keys()
        .filter(|key| !TITLE_PAGE_KEYS_IN_ORDER.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    remaining_keys.sort();

    for key in remaining_keys {
        if let Some(values) = metadata.get(&key) {
            lines.push(render_metadata_entry(&key, values, metadata));
        }
    }

    lines.join("\n")
}

fn render_metadata_entry(key: &str, values: &[ElementText], metadata: &Metadata) -> String {
    let display_key = metadata_display_key(key);
    match values {
        [] => format!("{display_key}:"),
        [single] if !single.plain_text().contains('\n') => {
            format!(
                "{display_key}: {}",
                render_metadata_value(key, single, Some(metadata))
            )
        }
        _ => {
            let mut lines = vec![format!("{display_key}:")];
            lines.extend(values.iter().flat_map(|value| {
                render_metadata_value(key, value, Some(metadata))
                    .split('\n')
                    .map(|line| format!("    {line}"))
                    .collect::<Vec<_>>()
            }));
            lines.join("\n")
        }
    }
}

fn render_metadata_value(key: &str, value: &ElementText, metadata: Option<&Metadata>) -> String {
    if key == "title"
        && metadata.is_some_and(|metadata| title_uses_only_default_fountain_styling(value, metadata))
    {
        return escape_plain_text(&value.plain_text());
    }
    render_element_text(value)
}

fn title_uses_only_default_fountain_styling(value: &ElementText, metadata: &Metadata) -> bool {
    let ElementText::Styled(runs) = value else {
        return false;
    };
    let allow_all_caps = plain_title_uses_all_caps(metadata);
    runs.iter().all(|run| {
        let has_bold = run.text_style.contains("Bold");
        let has_underline = run.text_style.contains("Underline");
        let has_all_caps = run.text_style.contains("AllCaps");
        has_bold
            && has_underline
            && !run.text_style.contains("Italic")
            && run
                .text_style
                .iter()
                .all(|style| matches!(style.as_str(), "Bold" | "Underline" | "AllCaps"))
            && (!allow_all_caps || has_all_caps || run.content == run.content.to_uppercase())
    })
}

fn metadata_display_key(key: &str) -> String {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

fn render_element_with_page_breaks(element: &Element) -> Vec<String> {
    match element {
        Element::Action(text, attributes)
        | Element::Character(text, attributes)
        | Element::SceneHeading(text, attributes)
        | Element::Lyric(text, attributes)
        | Element::Parenthetical(text, attributes)
        | Element::Dialogue(text, attributes)
        | Element::Transition(text, attributes)
        | Element::ColdOpening(text, attributes)
        | Element::NewAct(text, attributes)
        | Element::EndOfAct(text, attributes) => {
            render_simple_element(element, text, attributes)
        }
        Element::DialogueBlock(elements) => render_page_started_block(
            elements
                .first()
                .and_then(block_attributes)
                .map(|attributes| attributes.starts_new_page)
                .unwrap_or(false),
            render_dialogue_block(elements, false),
        ),
        Element::DualDialogueBlock(blocks) => render_page_started_block(
            blocks
                .iter()
                .find_map(block_attributes)
                .map(|attributes| attributes.starts_new_page)
                .unwrap_or(false),
            render_dual_dialogue_block(blocks),
        ),
        Element::Section(text, attributes, level) => render_page_started_block(
            attributes.starts_new_page,
            format!("{} {}", "#".repeat((*level).into()), render_element_text(text)),
        ),
        Element::Synopsis(text) => vec![format!("= {}", render_element_text(text))],
        Element::PageBreak => vec!["===".to_string()],
    }
}

fn render_simple_element(
    element: &Element,
    text: &ElementText,
    attributes: &Attributes,
) -> Vec<String> {
    render_page_started_block(attributes.starts_new_page, match element {
        Element::Action(_, _) => render_action(text, attributes),
        Element::Character(_, _) => render_character(text, false),
        Element::SceneHeading(_, _) => render_scene_heading(text, attributes),
        Element::Lyric(_, _) => render_lyric(text),
        Element::Parenthetical(_, _) => render_parenthetical(text),
        Element::Dialogue(_, _) => render_text_with_notes(text, attributes),
        Element::Transition(_, _) => render_transition(text),
        Element::ColdOpening(_, _) | Element::NewAct(_, _) | Element::EndOfAct(_, _) => {
            render_centered(text, attributes)
        }
        _ => unreachable!(),
    })
}

fn render_page_started_block(starts_new_page: bool, block: String) -> Vec<String> {
    let mut paragraphs = Vec::new();
    if starts_new_page {
        paragraphs.push("===".to_string());
    }
    paragraphs.push(block);
    paragraphs
}

fn block_attributes(element: &Element) -> Option<&Attributes> {
    match element {
        Element::Action(_, attributes)
        | Element::Character(_, attributes)
        | Element::SceneHeading(_, attributes)
        | Element::Lyric(_, attributes)
        | Element::Parenthetical(_, attributes)
        | Element::Dialogue(_, attributes)
        | Element::Transition(_, attributes)
        | Element::ColdOpening(_, attributes)
        | Element::NewAct(_, attributes)
        | Element::EndOfAct(_, attributes)
        | Element::Section(_, attributes, _) => Some(attributes),
        _ => None,
    }
}

fn render_action(text: &ElementText, attributes: &Attributes) -> String {
    if attributes.centered {
        return render_centered(text, attributes);
    }

    let rendered = render_text_with_notes(text, attributes);
    if action_requires_force(&rendered) {
        format!("!{rendered}")
    } else {
        rendered
    }
}

fn render_scene_heading(text: &ElementText, attributes: &Attributes) -> String {
    let mut rendered = render_element_text(text);
    if let Some(scene_number) = &attributes.scene_number {
        rendered.push(' ');
        rendered.push('#');
        rendered.push_str(scene_number);
        rendered.push('#');
    }

    if scene_heading_requires_force(&rendered) {
        format!(".{rendered}")
    } else {
        rendered
    }
}

fn render_character(text: &ElementText, dual: bool) -> String {
    let rendered = render_element_text(text);
    let mut cue = if character_requires_force(&rendered) {
        format!("@{rendered}")
    } else {
        rendered
    };
    if dual {
        cue.push_str(" ^");
    }
    cue
}

fn render_parenthetical(text: &ElementText) -> String {
    render_element_text(text)
}

fn render_transition(text: &ElementText) -> String {
    let rendered = render_element_text(text);
    if rendered.to_ascii_uppercase().ends_with("TO:") {
        rendered
    } else {
        format!("> {rendered}")
    }
}

fn render_lyric(text: &ElementText) -> String {
    render_element_text(text)
        .split('\n')
        .map(|line| format!("~{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_centered(text: &ElementText, attributes: &Attributes) -> String {
    let rendered = render_text_with_notes(text, attributes);
    rendered
        .split('\n')
        .map(|line| format!("> {line} <"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_dialogue_block(elements: &[Element], dual: bool) -> String {
    elements
        .iter()
        .map(|element| match element {
            Element::Character(text, _) => render_character(text, dual),
            Element::Parenthetical(text, _) => render_parenthetical(text),
            Element::Dialogue(text, attributes) => render_text_with_notes(text, attributes),
            Element::Lyric(text, _) => render_lyric(text),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_dual_dialogue_block(blocks: &[Element]) -> String {
    blocks
        .iter()
        .enumerate()
        .filter_map(|(index, element)| match element {
            Element::DialogueBlock(elements) => Some(render_dialogue_block(elements, index > 0)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_text_with_notes(text: &ElementText, attributes: &Attributes) -> String {
    let mut rendered = render_element_text(text);
    if let Some(notes) = &attributes.notes {
        for note in notes {
            rendered.push_str("[[");
            rendered.push_str(note);
            rendered.push_str("]]");
        }
    }
    rendered
}

fn render_element_text(text: &ElementText) -> String {
    match text {
        ElementText::Plain(text) => escape_plain_text(text),
        ElementText::Styled(runs) => render_text_runs(runs),
    }
}

fn render_text_runs(runs: &[TextRun]) -> String {
    let mut rendered = String::new();
    for run in runs {
        rendered.push_str(&render_text_run(run));
    }
    rendered
}

fn render_text_run(run: &TextRun) -> String {
    if run.text_style.is_empty() {
        return escape_plain_text(&run.content);
    }

    let wrapped_segments = run
        .content
        .split('\n')
        .map(|segment| wrap_styles(&escape_plain_text(segment), run))
        .collect::<Vec<_>>();

    wrapped_segments.join("\n")
}

fn wrap_styles(content: &str, run: &TextRun) -> String {
    let bold = run.text_style.contains("Bold");
    let italic = run.text_style.contains("Italic");
    let underline = run.text_style.contains("Underline");

    let styled = match (bold, italic) {
        (true, true) => format!("***{content}***"),
        (true, false) => format!("**{content}**"),
        (false, true) => format!("*{content}*"),
        (false, false) => content.to_string(),
    };

    if underline {
        format!("_{styled}_")
    } else {
        styled
    }
}

fn escape_plain_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
}

fn scene_heading_requires_force(text: &str) -> bool {
    let trimmed = text.trim_start();
    let uppercase = trimmed.to_ascii_uppercase();
    !(uppercase.starts_with("INT")
        || uppercase.starts_with("EXT")
        || uppercase.starts_with("EST")
        || uppercase.starts_with("I/E")
        || uppercase.starts_with("INT./EXT")
        || uppercase.starts_with("EXT./INT"))
}

fn character_requires_force(text: &str) -> bool {
    !text.chars().any(|ch| ch.is_alphabetic()) || text != text.to_uppercase()
}

fn action_requires_force(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or_default();
    let trimmed = first_line.trim_start();
    (trimmed.starts_with('.') && !trimmed.starts_with("..."))
        || trimmed.starts_with('@')
        || trimmed.starts_with('>')
        || trimmed.starts_with('#')
        || trimmed.starts_with('=')
        || trimmed.starts_with('~')
        || trimmed.chars().all(|ch| ch == '=') && trimmed.len() >= 3
        || trimmed.to_ascii_uppercase().ends_with("TO:")
        || !scene_heading_requires_force(trimmed) && trimmed == first_line.trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blank_attributes, p, tr, ElementText::Styled};

    #[test]
    fn serializer_round_trips_metadata_and_core_body() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "title".into(),
            vec![
                Styled(vec![tr("BRICK & STEEL", vec!["Bold", "Underline"])]),
                "FULL RETIRED".into(),
            ],
        );
        metadata.insert("credit".into(), vec!["Written by".into()]);
        metadata.insert("author".into(), vec!["Stu Maschwitz".into()]);
        metadata.insert("fmt".into(), vec!["balanced allow-lowercase-title".into()]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            elements: vec![
                Element::SceneHeading(
                    p("INT. HOUSE - DAY"),
                    Attributes {
                        scene_number: Some("12".into()),
                        ..Default::default()
                    },
                ),
                Element::Action(p("John enters."), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("BRICK"), blank_attributes()),
                    Element::Parenthetical(p("(quietly)"), blank_attributes()),
                    Element::Dialogue(
                        Styled(vec![tr("Hello", vec!["Italic"]), tr(".", vec![])]),
                        blank_attributes(),
                    ),
                ]),
                Element::Transition(p("CUT TO:"), blank_attributes()),
            ],
        };

        let rendered = render(&screenplay);
        let reparsed = crate::parse(&rendered);

        assert_eq!(reparsed.elements, screenplay.elements);
        assert_eq!(
            reparsed.metadata.get("title"),
            Some(&vec!["BRICK & STEEL".into(), "FULL RETIRED".into()])
        );
        for key in ["credit", "author", "fmt"] {
            assert_eq!(reparsed.metadata.get(key), screenplay.metadata.get(key));
        }
    }

    #[test]
    fn serializer_forces_ambiguous_elements_to_preserve_type() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            elements: vec![
                Element::SceneHeading(p("inside the school bus"), blank_attributes()),
                Element::Action(p("INT. HOUSE - DAY"), blank_attributes()),
                Element::DialogueBlock(vec![
                    Element::Character(p("McGregor"), blank_attributes()),
                    Element::Dialogue(p("What the fuck!?"), blank_attributes()),
                ]),
                Element::Transition(p("Fade to black."), blank_attributes()),
            ],
        };

        let rendered = render(&screenplay);

        assert!(rendered.contains(".inside the school bus"));
        assert!(rendered.contains("!INT. HOUSE - DAY"));
        assert!(rendered.contains("@McGregor"));
        assert!(rendered.contains("> Fade to black."));
        assert_eq!(crate::parse(&rendered).elements, screenplay.elements);
    }

    #[test]
    fn serializer_round_trips_dual_dialogue_page_breaks_and_centered_markers() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            elements: vec![
                Element::Action(
                    p("THE END"),
                    Attributes {
                        centered: true,
                        ..Default::default()
                    },
                ),
                Element::DualDialogueBlock(vec![
                    Element::DialogueBlock(vec![
                        Element::Character(p("BRICK"), blank_attributes()),
                        Element::Dialogue(p("Left side."), blank_attributes()),
                    ]),
                    Element::DialogueBlock(vec![
                        Element::Character(p("STEEL"), blank_attributes()),
                        Element::Dialogue(p("Right side."), blank_attributes()),
                    ]),
                ]),
                Element::Action(
                    p("New page action."),
                    Attributes {
                        starts_new_page: true,
                        ..Default::default()
                    },
                ),
            ],
        };

        let rendered = render(&screenplay);

        assert!(rendered.contains("> THE END <"));
        assert!(rendered.contains("STEEL ^"));
        assert!(rendered.contains("===\n\nNew page action."));
        assert_eq!(crate::parse(&rendered).elements, screenplay.elements);
    }

    #[test]
    fn serializer_does_not_force_safe_action_lines_that_begin_with_ellipses() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            imported_layout: None,
            elements: vec![Element::Action(
                p("...come to find Edward making the shapes."),
                blank_attributes(),
            )],
        };

        let rendered = render(&screenplay);

        assert_eq!(rendered, "...come to find Edward making the shapes.");
        assert_eq!(crate::parse(&rendered).elements, screenplay.elements);
    }
}
