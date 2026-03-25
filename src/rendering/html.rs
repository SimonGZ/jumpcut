use super::shared::{escape_html, join_metadata, sorted_style_names};
use crate::{Attributes, Element, ElementText, Screenplay};
use std::fmt::Write;

const HTML_STYLE: &str = include_str!("../templates/html_style.css");

pub(crate) fn render_document(screenplay: &Screenplay, head: bool) -> String {
    let mut out = String::with_capacity(32 * 1024);
    if head {
        out.push_str("<!doctype html>\n\n<html>\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\n  <title>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "title",
            " ",
        )));
        out.push_str("</title>\n\n  <style type=\"text/css\" media=\"screen\">\n   ");
        out.push_str(HTML_STYLE);
        out.push_str("\n  </style>\n</head>\n\n<body>\n");
    }

    out.push_str("<section class=\"screenplay\">\n");
    render_body(&mut out, screenplay);
    out.push_str("</section>\n");

    if head {
        out.push_str("</body>\n</html>\n");
    }

    out
}

fn render_body(out: &mut String, screenplay: &Screenplay) {
    if screenplay.metadata.get("title").is_some() {
        out.push_str(
            "    <section class=\"title-page\">\n        <div class=\"title\">\n            <h1>",
        );
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "title",
            "",
        )));
        out.push_str("</h1>\n            <p>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "credit",
            "",
        )));
        out.push_str("</p>\n            <p>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "author",
            "",
        )));
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "authors",
            "",
        )));
        out.push_str("</p>\n        </div>\n    </section>\n");
    }

    out.push_str("        <section class=\"body\">\n");
    for element in &screenplay.elements {
        match element {
            Element::DialogueBlock(block) => {
                out.push_str("    <div class=\"dialogueBlock\">\n");
                for child in block {
                    render_paragraph(out, child);
                }
                out.push_str("                </div>\n");
            }
            Element::DualDialogueBlock(blocks) => {
                out.push_str("                <div class=\"dualDialogueBlock\">\n");
                for block in blocks {
                    out.push_str("                    <div class=\"dialogueBlock\">\n");
                    if let Element::DialogueBlock(dialogue_block) = block {
                        for child in dialogue_block {
                            render_paragraph(out, child);
                        }
                    }
                    out.push_str("                    </div>\n");
                }
                out.push_str("                </div>\n");
            }
            _ => render_paragraph(out, element),
        }
    }
    out.push_str("        </section>\n");
}

fn render_paragraph(out: &mut String, element: &Element) {
    let (type_name, text, attributes) = match element {
        Element::Action(text, attributes)
        | Element::Character(text, attributes)
        | Element::SceneHeading(text, attributes)
        | Element::Lyric(text, attributes)
        | Element::Parenthetical(text, attributes)
        | Element::Dialogue(text, attributes)
        | Element::Transition(text, attributes)
        | Element::ColdOpening(text, attributes)
        | Element::NewAct(text, attributes)
        | Element::EndOfAct(text, attributes) => (element.name(), text, attributes),
        Element::Section(text, attributes, _) => ("Section", text, attributes),
        Element::Synopsis(text) => ("Synopsis", text, &Attributes::default()),
        Element::DialogueBlock(_) | Element::DualDialogueBlock(_) | Element::PageBreak => return,
    };

    write!(
        out,
        "                <p class=\"{}{}{}\">",
        class_name(type_name),
        if attributes.starts_new_page {
            " startsNewPage"
        } else {
            ""
        },
        if attributes.centered { " centered" } else { "" }
    )
    .unwrap();
    render_text(out, text);
    out.push_str("</p>\n");
}

fn render_text(out: &mut String, text: &ElementText) {
    match text {
        ElementText::Plain(text) => out.push_str(&escape_html(text)),
        ElementText::Styled(runs) => {
            for run in runs {
                let classes = sorted_style_names(run, false);
                if classes.is_empty() {
                    out.push_str(&escape_html(&run.content));
                } else {
                    write!(out, "<span class=\"{}\">", classes.join(" ")).unwrap();
                    out.push_str(&escape_html(&run.content));
                    out.push_str("</span>");
                }
            }
        }
    }
}

fn class_name(type_name: &str) -> &'static str {
    match type_name {
        "Scene Heading" => "sceneHeading",
        "Action" => "action",
        "Character" => "character",
        "Dialogue" => "dialogue",
        "Parenthetical" => "parenthetical",
        "Transition" => "transition",
        "Lyric" => "lyric",
        "Section" => "section",
        "Synopsis" => "synopsis",
        "Cold Opening" => "coldOpening",
        "New Act" => "newAct",
        "End of Act" => "endOfAct",
        _ => "unknown",
    }
}
