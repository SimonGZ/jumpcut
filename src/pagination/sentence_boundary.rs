pub(crate) fn sentence_boundary_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut offsets = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if !matches!(chars[index].1, '.' | '!' | '?') {
            index += 1;
            continue;
        }

        let mut next = index + 1;
        while next < chars.len() && is_sentence_closer(chars[next].1) {
            next += 1;
        }

        if next == chars.len() {
            offsets.push(text.len());
            index += 1;
            continue;
        }

        if chars[next].1.is_whitespace() {
            while next < chars.len() && chars[next].1.is_whitespace() {
                next += 1;
            }

            offsets.push(if next < chars.len() {
                chars[next].0
            } else {
                text.len()
            });
        }

        index += 1;
    }

    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

/// Sentence boundaries used by Final Draft 11's pagination correction pass.
///
/// This intentionally mirrors the small, English-centric recognizer recovered
/// from the macOS build rather than attempting general linguistic segmentation.
pub(crate) fn final_draft_sentence_boundary_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut offsets = Vec::new();

    for (index, (byte_offset, ch)) in chars.iter().copied().enumerate() {
        if !matches!(ch, '.' | '!' | '?' | ';') {
            continue;
        }
        if ch == '.' && is_final_draft_abbreviation(&text[..byte_offset]) {
            continue;
        }

        let mut next = index + 1;
        if next == chars.len() {
            offsets.push(text.len());
            continue;
        }
        if !chars[next].1.is_whitespace() {
            continue;
        }
        while next < chars.len() && chars[next].1.is_whitespace() {
            next += 1;
        }
        offsets.push(if next < chars.len() {
            chars[next].0
        } else {
            text.len()
        });
    }

    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

fn is_final_draft_abbreviation(prefix: &str) -> bool {
    const ABBREVIATIONS: &[&str] = &["DR", "MR", "MS", "ST", "MRS", "AVE", "BLVD"];

    let token_start = prefix
        .char_indices()
        .rev()
        .find_map(|(offset, ch)| (!ch.is_alphabetic()).then_some(offset + ch.len_utf8()))
        .unwrap_or(0);
    let token = &prefix[token_start..];
    let has_word_boundary = token_start == 0
        || prefix[..token_start]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);

    has_word_boundary
        && ABBREVIATIONS
            .iter()
            .any(|abbreviation| token.eq_ignore_ascii_case(abbreviation))
}

pub(crate) fn text_ends_sentence(text: &str) -> bool {
    text.trim_end_matches(char::is_whitespace)
        .trim_end_matches(is_sentence_closer)
        .chars()
        .last()
        .is_some_and(|ch| matches!(ch, '.' | '!' | '?'))
}

fn is_sentence_closer(ch: char) -> bool {
    matches!(ch, '"' | '\'' | ')' | ']' | '}')
}

#[cfg(test)]
mod tests {
    use super::final_draft_sentence_boundary_offsets;

    #[test]
    fn final_draft_boundaries_include_semicolons() {
        let text = "First clause; Second clause. Third.";
        let offsets = final_draft_sentence_boundary_offsets(text);

        assert_eq!(&text[offsets[0]..], "Second clause. Third.");
    }

    #[test]
    fn final_draft_boundaries_suppress_recovered_abbreviations() {
        for abbreviation in ["Dr.", "MR.", "ms.", "St.", "Mrs.", "Ave.", "Blvd."] {
            let text = format!("Ask {abbreviation} Smith. Continue.");
            let offsets = final_draft_sentence_boundary_offsets(&text);

            assert_eq!(offsets.len(), 2, "unexpected boundary for {abbreviation}");
            assert_eq!(&text[offsets[0]..], "Continue.");
        }
    }
}
