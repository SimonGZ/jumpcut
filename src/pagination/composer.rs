use crate::pagination::semantic::{SemanticUnit, FlowKind, DialoguePartKind};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig, ElementType};
use crate::pagination::LayoutGeometry;
use crate::pagination::fixtures::Fragment;

#[derive(Debug, PartialEq, Clone)]
pub struct LayoutBlock<'a> {
    pub unit: &'a SemanticUnit,
    pub fragment: Fragment,
    pub spacing_above: f32,
    pub content_lines: f32,
    pub keep_with_next: bool,
    pub can_split: bool,
    pub widow_penalty: f32,
}

pub fn compose<'a>(units: &'a [SemanticUnit], geometry: &LayoutGeometry) -> Vec<LayoutBlock<'a>> {
    let mut measured = Vec::new();

    for (_i, unit) in units.iter().enumerate() {
        let (content_lines, spacing_above) = match unit {
// ... (the match block stays as is)
            SemanticUnit::Flow(flow) => {
                let el_type = match flow.kind {
                    FlowKind::Action => ElementType::Action,
                    FlowKind::ColdOpening => ElementType::ColdOpening,
                    FlowKind::NewAct => ElementType::NewAct,
                    FlowKind::EndOfAct => ElementType::EndOfAct,
                    FlowKind::SceneHeading => ElementType::SceneHeading,
                    FlowKind::Transition => ElementType::Transition,
                    // Temporary defaults for unhandled FlowKinds
                    _ => ElementType::Action,
                };
                
                let config = WrapConfig::from_geometry(geometry, el_type);
                let lines = wrap_text_for_element(&flow.text, &config);
                
                let sp_above = match flow.kind {
                    FlowKind::SceneHeading => geometry.scene_heading_spacing_before,
                    FlowKind::ColdOpening => geometry.cold_opening_spacing_before,
                    FlowKind::NewAct => geometry.new_act_spacing_before,
                    FlowKind::EndOfAct => geometry.end_of_act_spacing_before,
                    FlowKind::Action => geometry.action_spacing_before,
                    FlowKind::Transition => geometry.transition_spacing_before,
                    _ => 1.0,
                };
                
                (lines.len(), sp_above)
            },
            SemanticUnit::Dialogue(dialogue) => {
                let lines = measure_dialogue_lines(dialogue.parts.iter(), geometry, |part| {
                    match part.kind {
                        DialoguePartKind::Character => ElementType::Character,
                        DialoguePartKind::Parenthetical => ElementType::Parenthetical,
                        DialoguePartKind::Dialogue => ElementType::Dialogue,
                        DialoguePartKind::Lyric => ElementType::Lyric,
                    }
                });
                (lines, geometry.character_spacing_before)
            },
            SemanticUnit::DualDialogue(dual) => {
                let mut max_lines = 0;
                for side in &dual.sides {
                    let side_element_type = match side.side {
                        1 => ElementType::DualDialogueLeft,
                        _ => ElementType::DualDialogueRight,
                    };
                    let side_lines =
                        measure_dialogue_lines(side.dialogue.parts.iter(), geometry, |_| {
                            side_element_type
                        });
                    if side_lines > max_lines { max_lines = side_lines; }
                }
                (max_lines, geometry.character_spacing_before)
            },
            SemanticUnit::Lyric(lyric) => {
                let config = WrapConfig::from_geometry(geometry, ElementType::Lyric);
                let lines = wrap_text_for_element(&lyric.text, &config).len();
                (lines, geometry.lyric_spacing_before)
            },
            SemanticUnit::PageStart(_) => (0, 0.0),
        };

        measured.push(LayoutBlock {
            unit,
            fragment: Fragment::Whole,
            spacing_above,
            content_lines: content_lines as f32 * geometry.line_height,
            keep_with_next: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.keep_with_next,
                _ => false,
            },
            can_split: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.can_split,
                _ => false,
            },
            widow_penalty: 0.0, // Dialogue will set this to 1.0 later
        });
    }

    measured
}

fn measure_dialogue_lines<'a>(
    parts: impl Iterator<Item = &'a crate::pagination::semantic::DialoguePart>,
    geometry: &LayoutGeometry,
    element_type_for_part: impl Fn(&crate::pagination::semantic::DialoguePart) -> ElementType,
) -> usize {
    let mut lines = 0;

    for part in parts {
        let config = WrapConfig::from_geometry(geometry, element_type_for_part(part));
        lines += wrap_text_for_element(&part.text, &config).len();
    }

    lines
}
