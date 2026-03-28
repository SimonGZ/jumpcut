use crate::pagination::split_scoring::choose_best_scored_split;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowSplitDecision {
    pub top_line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlowSplitPolicy {
    prefer_sentence_boundaries: bool,
    prefer_fuller_top_fragment: bool,
}

impl Default for FlowSplitPolicy {
    fn default() -> Self {
        Self {
            prefer_sentence_boundaries: true,
            prefer_fuller_top_fragment: true,
        }
    }
}

pub fn choose_flow_split(
    wrapped_lines: &[String],
    max_top_lines: usize,
    min_top_lines: usize,
    min_bottom_lines: usize,
) -> Option<FlowSplitDecision> {
    let policy = FlowSplitPolicy::default();

    choose_best_scored_split(1..wrapped_lines.len(), |top_line_count| {
        if top_line_count > max_top_lines {
            return None;
        }

        let bottom_line_count = wrapped_lines.len() - top_line_count;
        if top_line_count < min_top_lines || bottom_line_count < min_bottom_lines {
            return None;
        }

        Some(FlowSplitScore {
            ends_sentence: policy.prefer_sentence_boundaries
                && line_ends_sentence(&wrapped_lines[top_line_count - 1]),
            fuller_top_fragment: if policy.prefer_fuller_top_fragment {
                top_line_count
            } else {
                0
            },
            balance_score: balance_score(top_line_count, bottom_line_count),
        })
    })
    .map(|top_line_count| FlowSplitDecision { top_line_count })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FlowSplitScore {
    ends_sentence: bool,
    fuller_top_fragment: usize,
    balance_score: usize,
}

fn line_ends_sentence(line: &str) -> bool {
    matches!(line.trim_end().chars().last(), Some('.') | Some('!') | Some('?'))
}

fn balance_score(top_lines: usize, bottom_lines: usize) -> usize {
    usize::MAX - top_lines.abs_diff(bottom_lines)
}
