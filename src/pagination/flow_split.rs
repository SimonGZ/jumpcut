use crate::pagination::split_scoring::choose_best_scored_split;
use crate::pagination::sentence_boundary::{sentence_boundary_offsets, text_ends_sentence};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowSplitDecision {
    pub top_line_count: usize,
    pub bottom_line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSplitPlan {
    pub top_text: String,
    pub bottom_text: String,
    pub top_line_count: usize,
    pub bottom_line_count: usize,
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
    text: &str,
    config: &WrapConfig,
    max_top_lines: usize,
    min_top_lines: usize,
    min_bottom_lines: usize,
) -> Option<FlowSplitPlan> {
    let policy = FlowSplitPolicy::default();
    let candidate_offsets = sentence_boundary_offsets(text);

    choose_best_scored_split(candidate_offsets.into_iter(), |offset| {
        if offset == 0 || offset >= text.len() {
            return None;
        }

        let (top_text, bottom_text) = text.split_at(offset);
        let top_lines = wrap_fragment_lines(top_text, config);
        let bottom_lines = wrap_fragment_lines(bottom_text, config);
        let top_line_count = top_lines.len();
        let bottom_line_count = bottom_lines.len();

        if top_line_count < min_top_lines || bottom_line_count < min_bottom_lines {
            return None;
        }
        if top_line_count > max_top_lines {
            return None;
        }
        if has_discouraged_runt_top_line(&top_lines, top_text) {
            return None;
        }

        Some(FlowSplitScore {
            ends_sentence: policy.prefer_sentence_boundaries
                && text_ends_sentence(top_text),
            fuller_top_fragment: if policy.prefer_fuller_top_fragment {
                top_line_count
            } else {
                0
            },
            balance_score: balance_score(top_line_count, bottom_line_count),
        })
    })
    .map(|offset| {
        let (top_text, bottom_text) = text.split_at(offset);
        let top_line_count = wrap_fragment_lines(top_text, config).len();
        let bottom_line_count = wrap_fragment_lines(bottom_text, config).len();

        FlowSplitPlan {
            top_text: top_text.to_string(),
            bottom_text: bottom_text.to_string(),
            top_line_count,
            bottom_line_count,
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FlowSplitScore {
    ends_sentence: bool,
    fuller_top_fragment: usize,
    balance_score: usize,
}

fn wrap_fragment_lines(text: &str, config: &WrapConfig) -> Vec<String> {
    if text.is_empty() {
        Vec::new()
    } else {
        wrap_text_for_element(text, config)
    }
}

fn balance_score(top_lines: usize, bottom_lines: usize) -> usize {
    usize::MAX - top_lines.abs_diff(bottom_lines)
}

fn has_discouraged_runt_top_line(top_lines: &[String], _top_text: &str) -> bool {
    top_lines
        .last()
        .is_some_and(|line| visible_line_length(line) <= 12)
}

fn visible_line_length(line: &str) -> usize {
    line.trim().chars().count()
}
