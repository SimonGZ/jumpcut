use crate::{Metadata, TextRun};

pub(crate) fn sorted_style_names(run: &TextRun, preserve_case: bool) -> Vec<String> {
    let mut styles: Vec<String> = run.text_style.iter().cloned().collect();
    styles.sort_unstable();
    if preserve_case {
        styles
    } else {
        styles
            .into_iter()
            .map(|style| style.to_lowercase())
            .collect()
    }
}

pub(crate) fn join_metadata(metadata: &Metadata, key: &str, separator: &str) -> String {
    metadata
        .get(key)
        .map(|values| {
            values
                .iter()
                .map(|value| value.plain_text())
                .collect::<Vec<_>>()
                .join(separator)
        })
        .unwrap_or_default()
}

pub(crate) fn escape_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(feature = "html")]
pub(crate) fn escape_html(input: &str) -> String {
    escape_markup(input)
}

#[cfg(feature = "fdx")]
pub(crate) fn escape_xml_attr(input: &str) -> String {
    escape_markup(input)
}

#[cfg(feature = "fdx")]
pub(crate) fn escape_xml_text(input: &str) -> String {
    escape_markup(input)
}
