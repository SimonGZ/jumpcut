use crate::pagination::margin::calculate_element_width;
use crate::pagination::LayoutGeometry;

#[derive(Debug, Clone, Copy)]
pub enum ElementType {
    Action,
    ColdOpening,
    NewAct,
    EndOfAct,
    SceneHeading,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Lyric,
    DualDialogueLeft,
    DualDialogueRight,
}

impl ElementType {
    pub fn from_item_kind(kind: &str, dual_dialogue_side: Option<u8>) -> Self {
        if let Some(side) = dual_dialogue_side {
            return match side {
                1 => Self::DualDialogueLeft,
                _ => Self::DualDialogueRight,
            };
        }

        match kind {
            "Character" => Self::Character,
            "Dialogue" => Self::Dialogue,
            "Parenthetical" => Self::Parenthetical,
            "Lyric" => Self::Lyric,
            "Scene Heading" => Self::SceneHeading,
            "Transition" => Self::Transition,
            "Cold Opening" => Self::ColdOpening,
            "New Act" => Self::NewAct,
            "End of Act" => Self::EndOfAct,
            _ => Self::Action,
        }
    }

    pub fn from_flow_kind(kind: &crate::pagination::FlowKind) -> Self {
        match kind {
            crate::pagination::FlowKind::Action => Self::Action,
            crate::pagination::FlowKind::SceneHeading => Self::SceneHeading,
            crate::pagination::FlowKind::Transition => Self::Transition,
            crate::pagination::FlowKind::ColdOpening => Self::ColdOpening,
            crate::pagination::FlowKind::NewAct => Self::NewAct,
            crate::pagination::FlowKind::EndOfAct => Self::EndOfAct,
            _ => Self::Action,
        }
    }

    pub fn from_dialogue_part_kind(kind: &crate::pagination::DialoguePartKind) -> Self {
        match kind {
            crate::pagination::DialoguePartKind::Character => Self::Character,
            crate::pagination::DialoguePartKind::Dialogue => Self::Dialogue,
            crate::pagination::DialoguePartKind::Parenthetical => Self::Parenthetical,
            crate::pagination::DialoguePartKind::Lyric => Self::Lyric,
        }
    }
}

pub struct WrapConfig {
    pub exact_width_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLine {
    pub text: String,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WrapChunk {
    text: String,
    end_offset: usize,
}

impl WrapConfig {
    pub fn new(element_type: ElementType) -> Self {
        let geometry = LayoutGeometry::default();
        Self::from_geometry(&geometry, element_type)
    }

    pub fn from_geometry(geometry: &LayoutGeometry, element_type: ElementType) -> Self {
        Self {
            exact_width_chars: calculate_element_width(geometry, element_type),
        }
    }

    pub fn with_exact_width_chars(width: usize) -> Self {
        Self {
            exact_width_chars: width,
        }
    }
}

pub fn wrap_text_for_element(text: &str, config: &WrapConfig) -> Vec<String> {
    wrap_text_for_element_with_offsets(text, config)
        .into_iter()
        .map(|line| line.text)
        .collect()
}

pub fn wrap_text_for_element_with_offsets(text: &str, config: &WrapConfig) -> Vec<WrappedLine> {
    let mut lines = Vec::new();
    let max_width = config.exact_width_chars;

    if text.is_empty() {
        return lines;
    }

    let paragraphs: Vec<&str> = text.split('\n').collect();
    let mut paragraph_offset = 0;

    for paragraph in paragraphs {
        let mut current_line = String::new();
        let mut current_line_end_offset = paragraph_offset;
        let words = tokenize_wrap_chunks(paragraph, paragraph_offset);

        for word in words {
            let combined = format!("{}{}", current_line, word.text);

            // Final Draft explicitly discounts trailing whitespace and exactly
            // ONE single trailing hyphen from column width limits.
            let trimmed = combined.trim_end_matches(' ');
            let mut effective_combined_len = trimmed.chars().count();

            if trimmed.ends_with('-') {
                effective_combined_len = effective_combined_len.saturating_sub(1);
            }

            let fits = effective_combined_len <= max_width;

            if current_line.is_empty() {
                // Always push the first word of a line, even if it's too long
                current_line.push_str(&word.text);
                current_line_end_offset = word.end_offset;
            } else if fits {
                current_line.push_str(&word.text);
                current_line_end_offset = word.end_offset;
            } else {
                // Line is full. Trim trailing spaces on the rendered visual line
                lines.push(WrappedLine {
                    text: current_line.trim_end().to_string(),
                    end_offset: current_line_end_offset,
                });
                current_line = word.text;
                current_line_end_offset = word.end_offset;
            }
        }

        if !current_line.is_empty() {
            lines.push(WrappedLine {
                text: current_line.trim_end().to_string(),
                end_offset: current_line_end_offset,
            });
        }

        paragraph_offset += paragraph.len() + 1;
    }

    lines
}

fn tokenize_wrap_chunks(paragraph: &str, paragraph_offset: usize) -> Vec<WrapChunk> {
    let chars: Vec<char> = paragraph.chars().collect();
    let mut chunks = Vec::new();
    let mut index = 0;
    let mut byte_index = paragraph_offset;

    while index < chars.len() {
        if chars[index].is_whitespace() {
            let start = index;
            while index < chars.len() && chars[index].is_whitespace() {
                byte_index += chars[index].len_utf8();
                index += 1;
            }
            chunks.push(WrapChunk {
                text: chars[start..index].iter().collect(),
                end_offset: byte_index,
            });
            continue;
        }

        let start = index;
        let word_start_offset = byte_index;
        while index < chars.len() && !chars[index].is_whitespace() {
            byte_index += chars[index].len_utf8();
            index += 1;
        }

        let word: String = chars[start..index].iter().collect();
        let mut word_chunks = split_breakable_hyphen_chunks(&word, word_start_offset);

        let ws_start = index;
        while index < chars.len() && chars[index].is_whitespace() {
            byte_index += chars[index].len_utf8();
            index += 1;
        }
        if ws_start < index {
            let whitespace: String = chars[ws_start..index].iter().collect();
            if let Some(last) = word_chunks.last_mut() {
                last.text.push_str(&whitespace);
                last.end_offset = byte_index;
            }
        }

        chunks.extend(word_chunks);
    }

    chunks
}

fn split_breakable_hyphen_chunks(word: &str, start_offset: usize) -> Vec<WrapChunk> {
    let chars: Vec<char> = word.chars().collect();
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut consumed_bytes = 0;
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        current.push(ch);
        consumed_bytes += ch.len_utf8();

        if ch == '-' {
            let run_start = index;
            while index + 1 < chars.len() && chars[index + 1] == '-' {
                index += 1;
                current.push(chars[index]);
                consumed_bytes += chars[index].len_utf8();
            }

            let run_end = index;
            let has_alnum_neighbors = run_start > 0
                && run_end + 1 < chars.len()
                && chars[run_start - 1].is_alphanumeric()
                && chars[run_end + 1].is_alphanumeric();

            if has_alnum_neighbors {
                chunks.push(WrapChunk {
                    text: std::mem::take(&mut current),
                    end_offset: start_offset + consumed_bytes,
                });
            }
        }

        index += 1;
    }

    if !current.is_empty() {
        chunks.push(WrapChunk {
            text: current,
            end_offset: start_offset + consumed_bytes,
        });
    }

    chunks
}
