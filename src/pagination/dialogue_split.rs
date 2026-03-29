use std::collections::BTreeMap;

use crate::pagination::split_scoring::choose_best_scored_split;
use crate::pagination::sentence_boundary::{sentence_boundary_offsets, text_ends_sentence};
use crate::pagination::wrapping::{
    wrap_text_for_element, ElementType, WrapConfig,
};
use crate::pagination::{DialoguePartKind, DialogueUnit, LayoutGeometry};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialoguePartSplitLines {
    pub top_text: String,
    pub bottom_text: String,
    pub top_lines: Vec<String>,
    pub bottom_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueSplitPlan {
    pub top_line_count: usize,
    pub bottom_line_count: usize,
    pub ends_sentence: bool,
    pub parts: Vec<DialoguePartSplitLines>,
}

impl DialogueSplitPlan {
    pub fn top_page_line_count(&self) -> usize {
        self.top_line_count + more_line_cost()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueTextPart {
    pub kind: DialoguePartKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DialogueSplitPolicy {
    prefer_sentence_boundaries: bool,
    prefer_fuller_top_fragment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DialogueSplitBoundary {
    part_index: usize,
    offset: usize,
    ends_sentence: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DialogueSplitCandidate {
    plan: DialogueSplitPlan,
    top_dialogue_lines: usize,
    bottom_dialogue_lines: usize,
    ends_sentence: bool,
    top_content_bytes: usize,
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
                .then_some(top_line_count + more_line_cost())
                .unwrap_or(0),
            balance_score: balance_score(top_dialogue_lines, bottom_dialogue_lines),
            top_content_bytes: 0,
        })
    })
    .map(|top_line_count| DialogueSplitDecision { top_line_count })
}

pub fn plan_dialogue_split(
    dialogue: &DialogueUnit,
    geometry: &LayoutGeometry,
    max_top_lines: usize,
    min_top_dialogue_lines: usize,
    min_bottom_dialogue_lines: usize,
) -> Option<DialogueSplitPlan> {
    let parts = dialogue
        .parts
        .iter()
        .map(|part| DialogueTextPart {
            kind: part.kind.clone(),
            text: part.text.clone(),
        })
        .collect::<Vec<_>>();
    plan_dialogue_split_parts(
        dialogue,
        &parts,
        geometry,
        max_top_lines,
        min_top_dialogue_lines,
        min_bottom_dialogue_lines,
    )
}

pub fn plan_dialogue_split_parts(
    _dialogue: &DialogueUnit,
    parts: &[DialogueTextPart],
    geometry: &LayoutGeometry,
    max_top_lines: usize,
    min_top_dialogue_lines: usize,
    min_bottom_dialogue_lines: usize,
) -> Option<DialogueSplitPlan> {
    let policy = DialogueSplitPolicy::default();
    let candidates = generate_dialogue_split_candidates(parts, geometry);

    let winner = choose_best_scored_split(0..candidates.len(), |candidate_index| {
        let candidate = &candidates[candidate_index];
        if candidate.plan.top_line_count > max_top_lines {
            return None;
        }

        if candidate.top_dialogue_lines < min_top_dialogue_lines
            || candidate.bottom_dialogue_lines < min_bottom_dialogue_lines
        {
            return None;
        }

        Some(SplitScore {
            ends_sentence: policy.prefer_sentence_boundaries && candidate.ends_sentence,
            fuller_top_fragment: policy
                .prefer_fuller_top_fragment
                .then_some(candidate.plan.top_page_line_count())
                .unwrap_or(0),
            balance_score: balance_score(
                candidate.top_dialogue_lines,
                candidate.bottom_dialogue_lines,
            ),
            top_content_bytes: candidate.top_content_bytes,
        })
    });

    winner.map(|candidate_index| candidates[candidate_index].plan.clone())
}

fn generate_dialogue_split_candidates(
    parts: &[DialogueTextPart],
    geometry: &LayoutGeometry,
) -> Vec<DialogueSplitCandidate> {
    let mut boundaries: BTreeMap<(usize, usize), bool> = BTreeMap::new();

    for (part_index, part) in parts.iter().enumerate() {
        // Part-end boundaries are always candidates but never score as sentence endings.
        boundaries
            .entry((part_index, part.text.len()))
            .or_insert(false);

        if !matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric) {
            continue;
        }

        let config =
            WrapConfig::from_geometry(geometry, element_type_for_part_kind(part.kind.clone()));
        let total_wrapped_lines = wrap_text_for_element(&part.text, &config).len();

        // Only allow mid-text sentence splits for parts with at least 3 wrapped lines.
        // A 2-line part is too compact to split cleanly.
        if total_wrapped_lines < 3 {
            continue;
        }

        // FD only splits dialogue at sentence boundaries, never at arbitrary
        // wrapped-line breaks.
        for offset in sentence_boundary_offsets(&part.text) {
            boundaries
                .entry((part_index, offset))
                .and_modify(|ends_sentence| *ends_sentence = true)
                .or_insert(true);
        }
    }

    boundaries
        .into_iter()
        .filter_map(|((part_index, offset), ends_sentence)| {
            build_candidate(
                parts,
                geometry,
                DialogueSplitBoundary {
                    part_index,
                    offset,
                    ends_sentence,
                },
            )
        })
        .collect()
}

fn build_candidate(
    parts: &[DialogueTextPart],
    geometry: &LayoutGeometry,
    boundary: DialogueSplitBoundary,
) -> Option<DialogueSplitCandidate> {
    let mut top_line_count = 0;
    let mut bottom_line_count = 0;
    let mut top_dialogue_lines = 0;
    let mut bottom_dialogue_lines = 0;
    let mut split_parts = Vec::with_capacity(parts.len());

    for (part_index, part) in parts.iter().enumerate() {
        let (top_text, bottom_text) = split_part_text(&part.text, part_index, boundary);
        let config =
            WrapConfig::from_geometry(geometry, element_type_for_part_kind(part.kind.clone()));
        let top_lines = wrap_fragment_lines(top_text, &config);
        let bottom_lines = wrap_fragment_lines(bottom_text, &config);

        top_line_count += top_lines.len();
        bottom_line_count += bottom_lines.len();

        if matches!(part.kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric | DialoguePartKind::Parenthetical) {
            top_dialogue_lines += top_lines.len();
            bottom_dialogue_lines += bottom_lines.len();
        }

        split_parts.push(DialoguePartSplitLines {
            top_text: top_text.to_string(),
            bottom_text: bottom_text.to_string(),
            top_lines,
            bottom_lines,
        });
    }

    if top_line_count == 0 || bottom_line_count == 0 {
        return None;
    }

    let top_content_bytes: usize = split_parts.iter().map(|p| p.top_text.len()).sum();

    Some(DialogueSplitCandidate {
        plan: DialogueSplitPlan {
            top_line_count,
            bottom_line_count,
            ends_sentence: boundary.ends_sentence,
            parts: split_parts,
        },
        top_dialogue_lines,
        bottom_dialogue_lines,
        ends_sentence: boundary.ends_sentence,
        top_content_bytes,
    })
}

fn split_part_text<'a>(
    text: &'a str,
    part_index: usize,
    boundary: DialogueSplitBoundary,
) -> (&'a str, &'a str) {
    if part_index < boundary.part_index {
        return (text, "");
    }

    if part_index > boundary.part_index {
        return ("", text);
    }

    text.split_at(boundary.offset)
}

fn wrap_fragment_lines(text: &str, config: &WrapConfig) -> Vec<String> {
    if text.is_empty() {
        Vec::new()
    } else {
        wrap_text_for_element(text, config)
    }
}

fn element_type_for_part_kind(kind: DialoguePartKind) -> ElementType {
    match kind {
        DialoguePartKind::Character => ElementType::Character,
        DialoguePartKind::Parenthetical => ElementType::Parenthetical,
        DialoguePartKind::Dialogue => ElementType::Dialogue,
        DialoguePartKind::Lyric => ElementType::Lyric,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SplitScore {
    ends_sentence: bool,
    fuller_top_fragment: usize,
    balance_score: usize,
    top_content_bytes: usize,
}

fn count_dialogue_lines(lines: &[DialogueLine]) -> usize {
    lines.iter()
        .filter(|line| matches!(line.role, DialogueLineRole::Dialogue | DialogueLineRole::Parenthetical))
        .count()
}

fn line_ends_sentence(line: &DialogueLine) -> bool {
    matches!(line.role, DialogueLineRole::Dialogue) && text_ends_sentence(&line.text)
}

fn balance_score(top_dialogue_lines: usize, bottom_dialogue_lines: usize) -> usize {
    usize::MAX - top_dialogue_lines.abs_diff(bottom_dialogue_lines)
}

fn more_line_cost() -> usize {
    1
}
