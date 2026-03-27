use crate::pagination::margin::calculate_element_width;
use crate::pagination::LayoutGeometry;

#[derive(Debug, Clone, Copy)]
pub enum ElementType {
    Action,
    SceneHeading,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Lyric,
}

impl ElementType {
    pub fn from_flow_kind(kind: &crate::pagination::FlowKind) -> Self {
        match kind {
            crate::pagination::FlowKind::Action => Self::Action,
            crate::pagination::FlowKind::SceneHeading => Self::SceneHeading,
            crate::pagination::FlowKind::Transition => Self::Transition,
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
        // `split_inclusive` keeps the delimiter attached to the previous chunk,
        // allowing us to preserve internal spaces perfectly.
        let words: Vec<&str> = paragraph.split_inclusive(' ').collect();

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
                current_line.push_str(word);
            } else if fits {
                current_line.push_str(word);
            } else {
                // Line is full. Trim trailing spaces on the rendered visual line
                lines.push(current_line.trim_end().to_string());
                current_line = String::from(word);
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}
