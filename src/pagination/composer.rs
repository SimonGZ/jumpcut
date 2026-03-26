use crate::pagination::semantic::{SemanticUnit, FlowKind};
use crate::pagination::wrapping::{wrap_text_for_element, WrapConfig, ElementType};
use std::cmp::max;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MeasuredFlowUnit {
    pub spacing_above: usize,
    pub content_lines: usize,
    pub keep_with_next: bool,
    pub can_split: bool,
}

pub fn compose(units: &[SemanticUnit]) -> Vec<MeasuredFlowUnit> {
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
            // Other elements like Dialogue blocks will need similar measurement logic
            _ => (0, 0, 0),
        };

        // Resolution: non-additive padding logic
        let spacing_above = if i == 0 {
            req_spacing_above
        } else {
            max(previous_spacing_below, req_spacing_above)
        };

        measured.push(MeasuredFlowUnit {
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
        });

        previous_spacing_below = req_spacing_below;
    }

    measured
}
