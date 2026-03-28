use crate::pagination::split_scoring::choose_best_scored_split;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueLineRole {
    Character,
    Parenthetical,
    Dialogue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueLine {
    pub role: DialogueLineRole,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogueSplitDecision {
    pub top_line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DialogueSplitPolicy {
    prefer_sentence_boundaries: bool,
    prefer_fuller_top_fragment: bool,
}

impl Default for DialogueSplitPolicy {
    fn default() -> Self {
        Self {
            prefer_sentence_boundaries: true,
            prefer_fuller_top_fragment: true,
        }
    }
}

pub fn choose_dialogue_split(
    lines: &[DialogueLine],
    max_top_lines: usize,
    min_top_dialogue_lines: usize,
    min_bottom_dialogue_lines: usize,
) -> Option<DialogueSplitDecision> {
    let policy = DialogueSplitPolicy::default();

    choose_best_scored_split(1..lines.len(), |top_line_count| {
        if top_line_count > max_top_lines {
            return None;
        }

        let top = &lines[..top_line_count];
        let bottom = &lines[top_line_count..];
        let top_dialogue_lines = count_dialogue_lines(top);
        let bottom_dialogue_lines = count_dialogue_lines(bottom);

        if top_dialogue_lines < min_top_dialogue_lines
            || bottom_dialogue_lines < min_bottom_dialogue_lines
        {
            return None;
        }

        Some(SplitScore {
            ends_sentence: policy.prefer_sentence_boundaries
                && line_ends_sentence(&lines[top_line_count - 1]),
            fuller_top_fragment: policy
                .prefer_fuller_top_fragment
                .then_some(top_line_count)
                .unwrap_or(0),
            balance_score: balance_score(top_dialogue_lines, bottom_dialogue_lines),
        })
    })
    .map(|top_line_count| DialogueSplitDecision { top_line_count })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SplitScore {
    ends_sentence: bool,
    fuller_top_fragment: usize,
    balance_score: usize,
}

fn count_dialogue_lines(lines: &[DialogueLine]) -> usize {
    lines.iter()
        .filter(|line| line.role == DialogueLineRole::Dialogue)
        .count()
}

fn line_ends_sentence(line: &DialogueLine) -> bool {
    matches!(line.role, DialogueLineRole::Dialogue)
        && matches!(
            line.text.trim_end().chars().last(),
            Some('.') | Some('!') | Some('?')
        )
}

fn balance_score(top_dialogue_lines: usize, bottom_dialogue_lines: usize) -> usize {
    usize::MAX - top_dialogue_lines.abs_diff(bottom_dialogue_lines)
}
