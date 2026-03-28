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
    let mut lines = Vec::new();
    let max_width = config.exact_width_chars;

    if text.is_empty() {
        return lines;
    }

    for paragraph in text.lines() {
        let mut current_line = String::new();
        let words = tokenize_wrap_chunks(paragraph);

        for word in words {
            let combined = format!("{}{}", current_line, word);

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
                current_line.push_str(&word);
            } else if fits {
                current_line.push_str(&word);
            } else {
                // Line is full. Trim trailing spaces on the rendered visual line
                lines.push(current_line.trim_end().to_string());
                current_line = word;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}

fn tokenize_wrap_chunks(paragraph: &str) -> Vec<String> {
    let chars: Vec<char> = paragraph.chars().collect();
    let mut chunks = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if chars[index].is_whitespace() {
            let start = index;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            chunks.push(chars[start..index].iter().collect());
            continue;
        }

        let start = index;
        while index < chars.len() && !chars[index].is_whitespace() {
            index += 1;
        }

        let word: String = chars[start..index].iter().collect();
        let mut word_chunks = split_breakable_hyphen_chunks(&word);

        let ws_start = index;
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if ws_start < index {
            if let Some(last) = word_chunks.last_mut() {
                last.push_str(&chars[ws_start..index].iter().collect::<String>());
            }
        }

        chunks.extend(word_chunks);
    }

    chunks
}

fn split_breakable_hyphen_chunks(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for index in 0..chars.len() {
        let ch = chars[index];
        current.push(ch);

        if ch == '-'
            && index > 0
            && index + 1 < chars.len()
            && chars[index - 1].is_alphanumeric()
            && chars[index + 1].is_alphanumeric()
            && chars[index - 1] != '-'
            && chars[index + 1] != '-'
        {
            chunks.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}
