use super::shared::{escape_html, join_metadata, sorted_style_names};
use crate::html_output::HtmlRenderOptions;
use crate::visual_lines::{display_page_number, render_paginated_visual_pages, render_unpaginated_visual_lines, VisualLine};
use crate::pagination::{ScreenplayLayoutProfile, StyleProfile};
use crate::{Attributes, Element, ElementText, Screenplay};
use std::fmt::Write;

const HTML_STYLE: &str = include_str!("../templates/html_style.css");

pub(crate) fn render_document(screenplay: &Screenplay, options: HtmlRenderOptions) -> String {
    let layout_profile = ScreenplayLayoutProfile::from_metadata(&screenplay.metadata);
    let mut out = String::with_capacity(32 * 1024);
    if options.head {
        out.push_str("<!doctype html>\n\n<html>\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\n  <title>");
        out.push_str(&escape_html(&join_metadata(
            &screenplay.metadata,
            "title",
            " ",
        )));
        out.push_str("</title>\n\n  <style type=\"text/css\" media=\"screen\">\n   ");
        out.push_str(HTML_STYLE);
        out.push_str("\n  </style>\n</head>\n\n<body");
        if options.paginated {
            out.push_str(" class=\"paginatedHtmlView\"");
        }
        out.push_str(">\n");
    }

    write!(out, "<section class=\"{}\">\n", root_class_name(&layout_profile, options)).unwrap();
    render_body(&mut out, screenplay, options);
    out.push_str("</section>\n");

    if options.head {
        out.push_str("</body>\n</html>\n");
    }

    out
}

fn root_class_name(layout_profile: &ScreenplayLayoutProfile, options: HtmlRenderOptions) -> String {
    let mut classes = match layout_profile.style_profile {
        StyleProfile::Screenplay => vec!["screenplay"],
        StyleProfile::Multicam => vec!["screenplay", "multicam"],
    };

    if options.exact_wraps || options.paginated {
        classes.push("exactWraps");
    }
    if options.paginated {
        classes.push("paginatedHtml");
    }

    classes.join(" ")
}

fn render_body(out: &mut String, screenplay: &Screenplay, options: HtmlRenderOptions) {
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

    if options.paginated {
        render_paginated_body(out, screenplay);
        return;
    }

    if options.exact_wraps {
        render_exact_wrap_body(out, screenplay);
        return;
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

fn render_exact_wrap_body(out: &mut String, screenplay: &Screenplay) {
    out.push_str("        <section class=\"body exactWrapBody\">\n");
    for line in render_unpaginated_visual_lines(screenplay) {
        render_visual_line(out, &line);
    }
    out.push_str("        </section>\n");
}

fn render_paginated_body(out: &mut String, screenplay: &Screenplay) {
    out.push_str("        <section class=\"body paginatedBody\">\n");

    for page in render_paginated_visual_pages(screenplay) {
        write!(
            out,
            "            <section class=\"page{}\" data-page-number=\"{}\">\n",
            if page.page.metadata.index == 0 {
                " firstPage"
            } else {
                ""
            },
            page.page.metadata.number
        )
        .unwrap();

        out.push_str("                <div class=\"pageHeader\">");
        if let Some(display_number) = display_page_number(&page.page) {
            write!(out, "<span class=\"pageNumber\">{}.</span>", display_number).unwrap();
        }
        out.push_str("</div>\n");
        out.push_str("                <div class=\"pageBody\">\n");
        for line in page.lines {
            render_visual_line(out, &line);
        }
        out.push_str("                </div>\n");
        out.push_str("            </section>\n");
    }

    out.push_str("        </section>\n");
}

fn render_visual_line(out: &mut String, line: &VisualLine) {
    let mut classes = vec!["visualLine"];
    if line.text.is_empty() {
        classes.push("blankLine");
    }
    if !line.counted {
        classes.push("uncountedLine");
    }
    if line.centered {
        classes.push("centeredLine");
    }

    write!(out, "                    <div class=\"{}\">", classes.join(" ")).unwrap();
    if line.text.is_empty() {
        out.push_str("&nbsp;");
    } else {
        render_visual_fragments(out, &line.fragments);
    }
    out.push_str("</div>\n");
}

fn render_visual_fragments(out: &mut String, fragments: &[crate::visual_lines::VisualFragment]) {
    for fragment in fragments {
        if fragment.styles.is_empty() {
            out.push_str(&escape_html(&fragment.text));
        } else {
            let classes = fragment
                .styles
                .iter()
                .map(|style| style.to_lowercase())
                .collect::<Vec<_>>();
            write!(out, "<span class=\"{}\">", classes.join(" ")).unwrap();
            out.push_str(&escape_html(&fragment.text));
            out.push_str("</span>");
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{blank_attributes, p, tr, Attributes, Element, ElementText, Metadata};

    #[test]
    fn exact_wrap_html_renders_visual_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                p("THIS IS A LONG ACTION LINE THAT SHOULD WRAP WHEN EXACT HTML WRAPS ARE ENABLED"),
                blank_attributes(),
            )],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: true,
                paginated: false,
            },
        );

        assert!(output.contains("exactWraps"));
        assert!(output.contains("visualLine"));
        assert!(output.contains("exactWrapBody"));
        assert!(!output.contains("<p class=\"action"));
    }

    #[test]
    fn exact_wrap_html_preserves_styled_spans_for_unsplit_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                ElementText::Styled(vec![
                    tr("BOLD", vec!["Bold"]),
                    tr(" plain", vec![]),
                    tr(" ITALIC", vec!["Italic"]),
                ]),
                blank_attributes(),
            )],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: true,
                paginated: false,
            },
        );

        assert!(output.contains("<span class=\"bold\">BOLD</span> plain<span class=\"italic\"> ITALIC</span>"));
    }

    #[test]
    fn exact_wrap_html_marks_centered_lines() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                p("THE END"),
                Attributes {
                    centered: true,
                    ..blank_attributes()
                },
            )],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: true,
                paginated: false,
            },
        );

        assert!(output.contains("visualLine centeredLine"));
        assert!(output.contains(">THE END</div>"));
    }

    #[test]
    fn paginated_html_renders_page_containers_and_hides_first_page_number() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![
                Element::Action(p("FIRST PAGE"), blank_attributes()),
                Element::Action(
                    p("SECOND PAGE"),
                    Attributes {
                        starts_new_page: true,
                        ..blank_attributes()
                    },
                ),
            ],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: false,
                paginated: true,
            },
        );

        assert!(output.contains("paginatedHtml"));
        assert!(output.contains("class=\"page firstPage\""));
        assert!(output.contains("data-page-number=\"2\""));
        assert!(output.contains("<span class=\"pageNumber\">2.</span>"));
        assert!(!output.contains("<span class=\"pageNumber\">1.</span>"));
    }

    #[test]
    fn paginated_html_preserves_styled_spans_for_split_flow_fragments() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::Action(
                ElementText::Styled(vec![tr(&"BOLD SENTENCE. ".repeat(500), vec!["Bold"])]),
                blank_attributes(),
            )],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: false,
                paginated: true,
            },
        );

        let second_page = output
            .split("data-page-number=\"2\"")
            .nth(1)
            .expect("expected a second paginated page");

        assert!(second_page.contains("<span class=\"bold\">"));
    }

    #[test]
    fn paginated_html_preserves_styled_spans_for_dual_dialogue() {
        let screenplay = Screenplay {
            metadata: Metadata::new(),
            elements: vec![Element::DualDialogueBlock(vec![
                Element::DialogueBlock(vec![
                    Element::Character(
                        ElementText::Styled(vec![tr("BRICK", vec!["Bold"])]),
                        blank_attributes(),
                    ),
                    Element::Dialogue(
                        ElementText::Styled(vec![tr("Left side.", vec!["Italic"])]),
                        blank_attributes(),
                    ),
                ]),
                Element::DialogueBlock(vec![
                    Element::Character(
                        ElementText::Styled(vec![tr("STEEL", vec!["Underline"])]),
                        blank_attributes(),
                    ),
                    Element::Dialogue(
                        ElementText::Styled(vec![tr("Right side.", vec!["Bold"])]),
                        blank_attributes(),
                    ),
                ]),
            ])],
        };

        let output = render_document(
            &screenplay,
            HtmlRenderOptions {
                head: false,
                exact_wraps: false,
                paginated: true,
            },
        );

        assert!(output.contains("<span class=\"bold\">BRICK</span>"));
        assert!(output.contains("<span class=\"italic\">Left side.</span>"));
        assert!(output.contains("<span class=\"underline\">STEEL</span>"));
        assert!(output.contains("<span class=\"bold\">Right side.</span>"));
    }
}
