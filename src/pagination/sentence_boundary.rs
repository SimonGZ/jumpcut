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
