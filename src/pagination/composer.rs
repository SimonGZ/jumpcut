use crate::pagination::dialogue_split::DialogueSplitPlan;
use crate::pagination::flow_split::FlowSplitPlan;
use crate::pagination::margin::line_height_for_element_type;
use crate::pagination::semantic::{DialoguePartKind, FlowKind, SemanticUnit};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig, ElementType};
use crate::pagination::LayoutGeometry;
use crate::pagination::fixtures::Fragment;

#[derive(Debug, PartialEq, Clone)]
pub struct LayoutBlock<'a> {
    pub unit: &'a SemanticUnit,
    pub fragment: Fragment,
    pub spacing_above: f32,
    pub content_lines: f32,
    pub dialogue_split: Option<DialogueSplitPlan>,
    pub flow_split: Option<FlowSplitPlan>,
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
                
                (
                    measured_height_for_wrapped_lines(lines.len(), el_type, geometry),
                    sp_above,
                )
            },
            SemanticUnit::Dialogue(dialogue) => {
                let lines = measure_dialogue_height(dialogue.parts.iter(), geometry, |part| {
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
                let mut max_lines = 0.0;
                for side in &dual.sides {
                    let side_element_type = match side.side {
                        1 => ElementType::DualDialogueLeft,
                        _ => ElementType::DualDialogueRight,
                    };
                    let side_lines =
                        measure_dialogue_height(side.dialogue.parts.iter(), geometry, |_| {
                            side_element_type
                        });
                    if side_lines > max_lines { max_lines = side_lines; }
                }
                (max_lines, geometry.character_spacing_before)
            },
            SemanticUnit::Lyric(lyric) => {
                let config = WrapConfig::from_geometry(geometry, ElementType::Lyric);
                let lines = wrap_text_for_element(&lyric.text, &config).len();
                (
                    measured_height_for_wrapped_lines(lines, ElementType::Lyric, geometry),
                    geometry.lyric_spacing_before,
                )
            },
            SemanticUnit::PageStart(_) => (0.0, 0.0),
        };

        measured.push(LayoutBlock {
            unit,
            fragment: Fragment::Whole,
            spacing_above,
            content_lines,
            dialogue_split: None,
            flow_split: None,
            keep_with_next: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.keep_with_next,
                _ => false,
            },
            can_split: match unit {
                SemanticUnit::Flow(flow) => flow.cohesion.can_split,
                SemanticUnit::Dialogue(_) => true,
                _ => false,
            },
            widow_penalty: 0.0, // Dialogue will set this to 1.0 later
        });
    }

    measured
}

fn measure_dialogue_height<'a>(
    parts: impl Iterator<Item = &'a crate::pagination::semantic::DialoguePart>,
    geometry: &LayoutGeometry,
    element_type_for_part: impl Fn(&crate::pagination::semantic::DialoguePart) -> ElementType,
) -> f32 {
    let mut lines = 0.0;

    for part in parts {
        let element_type = element_type_for_part(part);
        let config = WrapConfig::from_geometry(geometry, element_type);
        lines += measured_height_for_wrapped_lines(
            wrap_text_for_element(&part.text, &config).len(),
            element_type,
            geometry,
        );
    }

    lines
}

fn measured_height_for_wrapped_lines(
    wrapped_line_count: usize,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> f32 {
    wrapped_line_count as f32 * line_height_for_element_type(geometry, element_type)
}
