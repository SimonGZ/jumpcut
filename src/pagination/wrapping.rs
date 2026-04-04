use crate::pagination::margin::calculate_element_width;
use crate::pagination::LayoutGeometry;
use crate::styled_text::{StyledRun, StyledText};

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
    DualDialogueCharacterLeft,
    DualDialogueCharacterRight,
    DualDialogueParentheticalLeft,
    DualDialogueParentheticalRight,
}

impl ElementType {
    pub fn from_item_kind(kind: &str, dual_dialogue_side: Option<u8>) -> Self {
        if let Some(side) = dual_dialogue_side {
            return match (kind, side) {
                ("Character", 1) => Self::DualDialogueCharacterLeft,
                ("Character", _) => Self::DualDialogueCharacterRight,
                ("Parenthetical", 1) => Self::DualDialogueParentheticalLeft,
                ("Parenthetical", _) => Self::DualDialogueParentheticalRight,
                (_, 1) => Self::DualDialogueLeft,
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

    pub fn from_dual_dialogue_part_kind(
        kind: &crate::pagination::DialoguePartKind,
        side: u8,
    ) -> Self {
        match (kind, side) {
            (crate::pagination::DialoguePartKind::Character, 1) => Self::DualDialogueCharacterLeft,
            (crate::pagination::DialoguePartKind::Character, _) => Self::DualDialogueCharacterRight,
            (crate::pagination::DialoguePartKind::Parenthetical, 1) => {
                Self::DualDialogueParentheticalLeft
            }
            (crate::pagination::DialoguePartKind::Parenthetical, _) => {
                Self::DualDialogueParentheticalRight
            }
            (_, 1) => Self::DualDialogueLeft,
            (_, _) => Self::DualDialogueRight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptionDashWrap {
    FinalDraft,
    KeepTogether,
}

pub struct WrapConfig {
    pub exact_width_chars: usize,
    pub interruption_dash_wrap: InterruptionDashWrap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLine {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedStyledFragment {
    pub text: String,
    pub styles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedStyledLine {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub fragments: Vec<WrappedStyledFragment>,
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
        Self::from_geometry_with_mode(geometry, element_type, InterruptionDashWrap::FinalDraft)
    }

    pub fn from_geometry_with_mode(
        geometry: &LayoutGeometry,
        element_type: ElementType,
        interruption_dash_wrap: InterruptionDashWrap,
    ) -> Self {
        Self {
            exact_width_chars: calculate_element_width(geometry, element_type),
            interruption_dash_wrap,
        }
    }

    pub fn with_exact_width_chars(width: usize) -> Self {
        Self {
            exact_width_chars: width,
            interruption_dash_wrap: InterruptionDashWrap::FinalDraft,
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
        let mut current_line_start_offset = paragraph_offset;
        let mut current_line_end_offset = paragraph_offset;
        let words = tokenize_wrap_chunks(paragraph, paragraph_offset);

        let mut word_index = 0;
        while word_index < words.len() {
            let word = &words[word_index];
            let word_start_offset = line_start_offset_for_chunk(word);
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
                current_line_start_offset = word_start_offset;
                current_line_end_offset = word.end_offset;
            } else if fits {
                current_line.push_str(&word.text);
                current_line_end_offset = word.end_offset;
            } else if should_split_interruption_dash(
                &current_line,
                &word.text,
                &words[word_index + 1..],
                max_width,
                config.interruption_dash_wrap,
            ) {
                let split_offset = word_start_offset + 1;
                current_line.push('-');
                current_line_end_offset = split_offset;
                lines.push(WrappedLine {
                    text: current_line.trim_end().to_string(),
                    start_offset: current_line_start_offset,
                    end_offset: current_line_end_offset,
                });
                current_line = split_interruption_dash_chunk(&word.text);
                current_line_start_offset = split_offset;
                current_line_end_offset = word.end_offset;
            } else if should_split_trailing_double_hyphen_word(
                &current_line,
                &word.text,
                max_width,
                config.interruption_dash_wrap,
            ) {
                let top_fragment = top_fragment_for_trailing_double_hyphen(&word.text);
                let split_offset = word_start_offset + top_fragment.len();
                current_line.push_str(&top_fragment);
                current_line_end_offset = split_offset;
                lines.push(WrappedLine {
                    text: current_line.trim_end().to_string(),
                    start_offset: current_line_start_offset,
                    end_offset: current_line_end_offset,
                });
                current_line = bottom_fragment_for_trailing_double_hyphen(&word.text);
                current_line_start_offset = split_offset;
                current_line_end_offset = word.end_offset;
            } else {
                // Line is full. Trim trailing spaces on the rendered visual line
                lines.push(WrappedLine {
                    text: current_line.trim_end().to_string(),
                    start_offset: current_line_start_offset,
                    end_offset: current_line_end_offset,
                });
                current_line = word.text.clone();
                current_line_start_offset = word_start_offset;
                current_line_end_offset = word.end_offset;
            }
            word_index += 1;
        }

        if !current_line.is_empty() {
            lines.push(WrappedLine {
                text: current_line.trim_end().to_string(),
                start_offset: current_line_start_offset,
                end_offset: current_line_end_offset,
            });
        }

        paragraph_offset += paragraph.len() + 1;
    }

    lines
}

fn should_split_interruption_dash(
    current_line: &str,
    next_chunk: &str,
    remaining_chunks: &[WrapChunk],
    max_width: usize,
    mode: InterruptionDashWrap,
) -> bool {
    if mode != InterruptionDashWrap::FinalDraft {
        return false;
    }

    if current_line.trim_end().is_empty() {
        return false;
    }

    if next_chunk.trim() != "--" {
        return false;
    }

    let keep_together_next_line =
        wrap_single_line_from_chunks(next_chunk.to_string(), remaining_chunks, max_width);
    let split_next_line =
        wrap_single_line_from_chunks(split_interruption_dash_chunk(next_chunk), remaining_chunks, max_width);

    visible_alnum_count(&split_next_line) > visible_alnum_count(&keep_together_next_line)
}

fn split_interruption_dash_chunk(chunk: &str) -> String {
    let trailing_whitespace: String = chunk
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("-{trailing_whitespace}")
}

fn should_split_trailing_double_hyphen_word(
    current_line: &str,
    next_chunk: &str,
    max_width: usize,
    mode: InterruptionDashWrap,
) -> bool {
    if mode != InterruptionDashWrap::FinalDraft {
        return false;
    }

    let trimmed = next_chunk.trim_end();
    if !trimmed.ends_with("--") || trimmed.len() <= 2 {
        return false;
    }

    let stem = &trimmed[..trimmed.len() - 2];
    if !stem.chars().any(|ch| ch.is_alphanumeric()) {
        return false;
    }

    let combined = format!("{current_line}{}", top_fragment_for_trailing_double_hyphen(next_chunk));
    effective_line_len(&combined) <= max_width
}

fn top_fragment_for_trailing_double_hyphen(chunk: &str) -> String {
    let trimmed = chunk.trim_end();
    let trailing_whitespace: String = chunk[trimmed.len()..].to_string();
    let stem = &trimmed[..trimmed.len() - 2];
    format!("{stem}-{trailing_whitespace}")
}

fn bottom_fragment_for_trailing_double_hyphen(_chunk: &str) -> String {
    "-".to_string()
}

fn wrap_single_line_from_chunks(
    mut current_line: String,
    chunks: &[WrapChunk],
    max_width: usize,
) -> String {
    for chunk in chunks {
        let combined = format!("{current_line}{}", chunk.text);
        if current_line.is_empty() || effective_line_len(&combined) <= max_width {
            current_line.push_str(&chunk.text);
        } else {
            break;
        }
    }

    current_line.trim_end().to_string()
}

fn effective_line_len(text: &str) -> usize {
    let trimmed = text.trim_end_matches(' ');
    let mut effective_len = trimmed.chars().count();
    if trimmed.ends_with('-') {
        effective_len = effective_len.saturating_sub(1);
    }
    effective_len
}

fn visible_alnum_count(text: &str) -> usize {
    text.chars().filter(|ch| ch.is_alphanumeric()).count()
}

pub fn wrap_styled_text_for_element(
    text: &StyledText,
    config: &WrapConfig,
) -> Vec<WrappedStyledLine> {
    let wrapped_lines = wrap_text_for_element_with_offsets(&text.plain_text, config);
    let run_ranges = styled_run_ranges(text);

    wrapped_lines
        .into_iter()
        .map(|line| WrappedStyledLine {
            text: line.text,
            start_offset: line.start_offset,
            end_offset: line.end_offset,
            fragments: styled_fragments_for_range(&run_ranges, line.start_offset, line.end_offset),
        })
        .collect()
}

fn line_start_offset_for_chunk(chunk: &WrapChunk) -> usize {
    chunk.end_offset.saturating_sub(chunk.text.len())
}

fn styled_run_ranges(text: &StyledText) -> Vec<(usize, usize, &StyledRun)> {
    let mut ranges = Vec::with_capacity(text.runs.len());
    let mut start = 0usize;

    for run in &text.runs {
        let end = start + run.text.len();
        ranges.push((start, end, run));
        start = end;
    }

    ranges
}

fn styled_fragments_for_range(
    run_ranges: &[(usize, usize, &StyledRun)],
    start_offset: usize,
    end_offset: usize,
) -> Vec<WrappedStyledFragment> {
    run_ranges
        .iter()
        .filter_map(|(run_start, run_end, run)| {
            let slice_start = (*run_start).max(start_offset);
            let slice_end = (*run_end).min(end_offset);

            if slice_start >= slice_end {
                return None;
            }

            Some(WrappedStyledFragment {
                text: run.text[(slice_start - run_start)..(slice_end - run_start)].to_string(),
                styles: run.styles.clone(),
            })
        })
        .collect()
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
