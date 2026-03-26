use crate::pagination::semantic::{SemanticUnit, FlowKind, DialoguePartKind};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig, ElementType};
use crate::pagination::LayoutGeometry;
use crate::pagination::fixtures::Fragment;

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

pub fn compose<'a>(units: &'a [SemanticUnit], geometry: &LayoutGeometry) -> Vec<LayoutBlock<'a>> {
    let mut measured = Vec::new();

    for (i, unit) in units.iter().enumerate() {
        let (content_lines, spacing_above) = match unit {
// ... (the match block stays as is)
            SemanticUnit::Flow(flow) => {
                let el_type = match flow.kind {
                    FlowKind::Action => ElementType::Action,
                    FlowKind::SceneHeading => ElementType::SceneHeading,
                    FlowKind::Transition => ElementType::Transition,
                    // Temporary defaults for unhandled FlowKinds
                    _ => ElementType::Action,
                };
                
                let config = WrapConfig::from_geometry(geometry, el_type);
                let lines = wrap_text_for_element(&flow.text, &config);
                
                let sp_above = match flow.kind {
                    FlowKind::SceneHeading => geometry.scene_heading_spacing_before,
                    FlowKind::Action => geometry.action_spacing_before,
                    FlowKind::Transition => geometry.transition_spacing_before,
                    _ => 1,
                };
                
                (lines.len(), sp_above)
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
                    let config = WrapConfig::from_geometry(geometry, el_type);
                    lines += wrap_text_for_element(&part.text, &config).len();
                }
                (lines, geometry.character_spacing_before)
            },
            SemanticUnit::DualDialogue(dual) => {
                let mut max_lines = 0;
                for side in &dual.sides {
                    let mut side_lines = 0;
                    let config = WrapConfig::from_geometry(geometry, ElementType::Action);
                    for part in &side.dialogue.parts {
                        // Temp fast approximation for Dual Dialogue halves
                        side_lines += wrap_text_for_element(&part.text, &config).len();
                    }
                    if side_lines > max_lines { max_lines = side_lines; }
                }
                (max_lines, geometry.character_spacing_before)
            },
            SemanticUnit::Lyric(lyric) => {
                let config = WrapConfig::from_geometry(geometry, ElementType::Lyric);
                let lines = wrap_text_for_element(&lyric.text, &config).len();
                (lines, geometry.lyric_spacing_before)
            },
            SemanticUnit::PageStart(_) => (0, 0),
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
    }

    measured
}
