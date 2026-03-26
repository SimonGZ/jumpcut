use crate::pagination::semantic::{SemanticUnit, FlowKind, DialoguePartKind};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig, ElementType};
use crate::pagination::fixtures::Fragment;
use std::cmp::max;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LayoutBlock<'a> {
    pub unit: &'a SemanticUnit,
    pub fragment: Fragment,
    pub spacing_above: usize,
    pub content_lines: usize,
    pub keep_with_next: bool,
    pub can_split: bool,
    pub widow_penalty: usize,
}

pub fn compose<'a>(units: &'a [SemanticUnit]) -> Vec<LayoutBlock<'a>> {
    let mut measured = Vec::new();
    let mut previous_spacing_below = 0;

    for (i, unit) in units.iter().enumerate() {
        let (content_lines, req_spacing_above, req_spacing_below) = match unit {
            SemanticUnit::Flow(flow) => {
                let el_type = match flow.kind {
                    FlowKind::Action => ElementType::Action,
                    FlowKind::SceneHeading => ElementType::SceneHeading,
                    FlowKind::Transition => ElementType::Transition,
                    // Temporary defaults for unhandled FlowKinds
                    _ => ElementType::Action,
                };
                
                let config = WrapConfig::new(el_type);
                let lines = wrap_text_for_element(&flow.text, &config);
                
                let (sp_above, sp_below) = match flow.kind {
                    FlowKind::SceneHeading => (2, 1), // 2 visual lines above, 1 below
                    FlowKind::Action => (1, 1),
                    FlowKind::Transition => (1, 1),
                    _ => (1, 1),
                };
                
                (lines.len(), sp_above, sp_below)
            },
            SemanticUnit::Dialogue(dialogue) => {
                let mut lines = 0;
                for part in &dialogue.parts {
                    let el_type = match part.kind {
                        DialoguePartKind::Character => ElementType::Character,
                        DialoguePartKind::Parenthetical => ElementType::Parenthetical,
                        DialoguePartKind::Dialogue => ElementType::Dialogue,
                        DialoguePartKind::Lyric => ElementType::Lyric,
                    };
                    let config = WrapConfig::new(el_type);
                    lines += wrap_text_for_element(&part.text, &config).len();
                }
                (lines, 1, 1)
            },
            SemanticUnit::DualDialogue(dual) => {
                let mut max_lines = 0;
                for side in &dual.sides {
                    let mut side_lines = 0;
                    for part in &side.dialogue.parts {
                        // Temp fast approximation for Dual Dialogue halves
                        side_lines += wrap_text_for_element(&part.text, &WrapConfig::new(ElementType::Action)).len();
                    }
                    if side_lines > max_lines { max_lines = side_lines; }
                }
                (max_lines, 1, 1)
            },
            SemanticUnit::Lyric(lyric) => {
                let config = WrapConfig::new(ElementType::Lyric);
                let lines = wrap_text_for_element(&lyric.text, &config).len();
                (lines, 1, 1)
            },
            SemanticUnit::PageStart(_) => (0, 0, 0),
        };

        // Resolution: non-additive padding logic
        let spacing_above = if i == 0 {
            req_spacing_above
        } else {
            max(previous_spacing_below, req_spacing_above)
        };

        measured.push(LayoutBlock {
            unit,
            fragment: Fragment::Whole,
            spacing_above,
            content_lines,
            keep_with_next: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.keep_with_next,
                _ => false,
            },
            can_split: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.can_split,
                _ => false,
            },
            widow_penalty: 0, // Dialogue will set this to 1 later
        });

        previous_spacing_below = req_spacing_below;
    }

    measured
}
