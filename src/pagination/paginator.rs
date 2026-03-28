use crate::pagination::composer::LayoutBlock;
use crate::pagination::dialogue_split::{
    plan_dialogue_split, plan_dialogue_split_parts, DialogueSplitPlan, DialogueTextPart,
};
use crate::pagination::flow_split::{choose_flow_split, FlowSplitPlan};
use crate::pagination::fixtures::Fragment;
use crate::pagination::wrapping::{wrap_text_for_element, ElementType, WrapConfig};
use crate::pagination::SemanticUnit;
use crate::pagination::LayoutGeometry;

pub struct Page<'a> {
    pub blocks: Vec<LayoutBlock<'a>>,
}

struct SplitDecision {
    top_lines: f32,
    bottom_lines: f32,
    dialogue_split: Option<DialogueSplitPlan>,
    flow_split: Option<FlowSplitPlan>,
}

/// An atomic group of blocks that absolutely cannot be split across a page boundary.
struct Chunk<'a> {
    blocks: Vec<&'a LayoutBlock<'a>>,
}

pub fn paginate<'a>(blocks: &'a [LayoutBlock<'a>], page_limit_lines: f32, geometry: &LayoutGeometry) -> Vec<Page<'a>> {
    let mut chunks: Vec<Chunk<'a>> = Vec::new();
    let mut current_chunk: Vec<&LayoutBlock<'a>> = Vec::new();

    for block in blocks {
        current_chunk.push(block);
        if !block.keep_with_next {
            chunks.push(Chunk { blocks: current_chunk });
            current_chunk = Vec::new();
        }
    }
    
    if !current_chunk.is_empty() {
        chunks.push(Chunk { blocks: current_chunk });
    }

    let mut pages: Vec<Page<'a>> = Vec::new();
    let mut current_page_blocks = Vec::new();
    let mut current_page_lines: f32 = 0.0;

    for chunk in chunks {
        if chunk.blocks.len() == 1 && matches!(chunk.blocks[0].unit, SemanticUnit::PageStart(_)) {
            if current_page_blocks.iter().any(block_has_visible_content) {
                pages.push(Page { blocks: current_page_blocks });
                current_page_blocks = Vec::new();
                current_page_lines = 0.0;
            }

                current_page_blocks.push(LayoutBlock {
                    unit: chunk.blocks[0].unit,
                    fragment: chunk.blocks[0].fragment.clone(),
                    spacing_above: 0.0,
                    content_lines: 0.0,
                    dialogue_split: None,
                    flow_split: None,
                    keep_with_next: false,
                    can_split: false,
                    widow_penalty: 0.0,
                });
            continue;
        }

        let mut chunk_height: f32 = 0.0;
        let mut page_has_visible_content =
            current_page_blocks.iter().any(block_has_visible_content);
        
        for block in &chunk.blocks {
            let effective_spacing = if !page_has_visible_content && block_has_visible_content(block) {
                0.0
            } else {
                block.spacing_above
            };
            chunk_height += effective_spacing + block.content_lines;
            if block_has_visible_content(block) {
                page_has_visible_content = true;
            }
        }

        if current_page_lines + chunk_height > page_limit_lines {
            if chunk.blocks.len() == 1 && chunk.blocks[0].can_split {
                let block = chunk.blocks[0];
                let effective_spacing = if current_page_blocks.iter().any(block_has_visible_content) {
                    block.spacing_above
                } else {
                    0.0
                };
                let available_lines = (page_limit_lines - current_page_lines).max(0.0);

                if let Some(split) =
                    choose_split_lines(block, available_lines, effective_spacing, geometry)
                {
                    current_page_blocks.push(LayoutBlock {
                        unit: block.unit,
                        fragment: Fragment::ContinuedToNext,
                        spacing_above: effective_spacing,
                        content_lines: split.top_lines,
                        dialogue_split: split.dialogue_split.clone(),
                        flow_split: split.flow_split.clone(),
                        keep_with_next: false,
                        can_split: false,
                        widow_penalty: 0.0,
                    });

                    pages.push(Page { blocks: current_page_blocks });

                    current_page_blocks = vec![LayoutBlock {
                        unit: block.unit,
                        fragment: Fragment::ContinuedFromPrev,
                        spacing_above: 0.0,
                        content_lines: split.bottom_lines + block.widow_penalty,
                        dialogue_split: split.dialogue_split,
                        flow_split: split.flow_split,
                        keep_with_next: block.keep_with_next,
                        can_split: block.can_split,
                        widow_penalty: 0.0,
                    }];
                    current_page_lines = split.bottom_lines + block.widow_penalty;
                    continue;
                }
            }

            if chunk_starts_with_transition(&chunk)
                && current_page_blocks.iter().any(block_has_visible_content)
            {
                if let Some(moved_block) = pull_last_visible_block_for_transition(&mut current_page_blocks) {
                    if !current_page_blocks.is_empty() {
                        pages.push(Page { blocks: current_page_blocks });
                    }

                    current_page_blocks = vec![LayoutBlock {
                        unit: moved_block.unit,
                        fragment: moved_block.fragment,
                        spacing_above: 0.0,
                        content_lines: moved_block.content_lines,
                        dialogue_split: moved_block.dialogue_split,
                        flow_split: moved_block.flow_split,
                        keep_with_next: moved_block.keep_with_next,
                        can_split: moved_block.can_split,
                        widow_penalty: moved_block.widow_penalty,
                    }];
                    current_page_lines = current_page_blocks[0].content_lines;

                    let mut page_has_visible_content = true;
                    for block in &chunk.blocks {
                        let effective_spacing = if !page_has_visible_content && block_has_visible_content(block) {
                            0.0
                        } else {
                            block.spacing_above
                        };

                        current_page_blocks.push(LayoutBlock {
                            unit: block.unit,
                            fragment: block.fragment.clone(),
                            spacing_above: effective_spacing,
                            content_lines: block.content_lines,
                            dialogue_split: block.dialogue_split.clone(),
                            flow_split: block.flow_split.clone(),
                            keep_with_next: block.keep_with_next,
                            can_split: block.can_split,
                            widow_penalty: block.widow_penalty,
                        });

                        current_page_lines += effective_spacing + block.content_lines;
                        if block_has_visible_content(block) {
                            page_has_visible_content = true;
                        }
                    }
                    continue;
                }
            }

            if !current_page_blocks.is_empty() {
                pages.push(Page { blocks: current_page_blocks });
            }
            
            current_page_blocks = Vec::new();
            current_page_lines = 0.0;
            
            let mut page_has_visible_content = false;

            for block in &chunk.blocks {
                let effective_spacing = if !page_has_visible_content && block_has_visible_content(block) {
                    0.0
                } else {
                    block.spacing_above
                };
                
                current_page_blocks.push(LayoutBlock {
                    unit: block.unit,
                    fragment: block.fragment.clone(),
                    spacing_above: effective_spacing,
                    content_lines: block.content_lines,
                    dialogue_split: block.dialogue_split.clone(),
                    flow_split: block.flow_split.clone(),
                    keep_with_next: block.keep_with_next,
                    can_split: block.can_split,
                    widow_penalty: block.widow_penalty,
                });
                
                current_page_lines += effective_spacing + block.content_lines;
                if block_has_visible_content(block) {
                    page_has_visible_content = true;
                }
            }
        } else {
            let mut page_has_visible_content =
                current_page_blocks.iter().any(block_has_visible_content);

            for block in &chunk.blocks {
                let effective_spacing = if !page_has_visible_content && block_has_visible_content(block) {
                    0.0
                } else {
                    block.spacing_above
                };
                
                current_page_blocks.push(LayoutBlock {
                    unit: block.unit,
                    fragment: block.fragment.clone(),
                    spacing_above: effective_spacing,
                    content_lines: block.content_lines,
                    dialogue_split: block.dialogue_split.clone(),
                    flow_split: block.flow_split.clone(),
                    keep_with_next: block.keep_with_next,
                    can_split: block.can_split,
                    widow_penalty: block.widow_penalty,
                });
                
                current_page_lines += effective_spacing + block.content_lines;
                if block_has_visible_content(block) {
                    page_has_visible_content = true;
                }
            }
        }
    }

    if !current_page_blocks.is_empty() {
        pages.push(Page { blocks: current_page_blocks });
    }

    pages
}

fn block_has_visible_content(block: &LayoutBlock<'_>) -> bool {
    !matches!(block.unit, SemanticUnit::PageStart(_)) || block.content_lines > 0.0
}

fn choose_split_lines(
    block: &LayoutBlock<'_>,
    available_lines: f32,
    effective_spacing: f32,
    geometry: &LayoutGeometry,
) -> Option<SplitDecision> {
    if available_lines < effective_spacing + geometry.orphan_limit as f32 {
        return None;
    }

    match block.unit {
        SemanticUnit::Dialogue(dialogue) => {
            let max_top_lines = ((available_lines - effective_spacing) / geometry.line_height)
                .floor() as usize;

            let plan = match (&block.fragment, block.dialogue_split.as_ref()) {
                (Fragment::ContinuedFromPrev, Some(previous_plan)) => {
                    let current_parts = dialogue
                        .parts
                        .iter()
                        .zip(previous_plan.parts.iter())
                        .map(|(part, split)| DialogueTextPart {
                            kind: part.kind.clone(),
                            text: split.bottom_text.clone(),
                        })
                        .collect::<Vec<_>>();

                    plan_dialogue_split_parts(
                        dialogue,
                        &current_parts,
                        geometry,
                        max_top_lines,
                        geometry.orphan_limit,
                        geometry.widow_limit,
                    )?
                }
                _ => plan_dialogue_split(
                    dialogue,
                    geometry,
                    max_top_lines,
                    geometry.orphan_limit,
                    geometry.widow_limit,
                )?,
            };
            let top_lines = plan.top_page_line_count() as f32 * geometry.line_height;
            let bottom_dialogue_lines = plan.bottom_line_count as f32 * geometry.line_height;
            let bottom_lines = bottom_dialogue_lines;
            (bottom_lines >= geometry.widow_limit as f32 * geometry.line_height)
                .then_some(SplitDecision {
                    top_lines,
                    bottom_lines,
                    dialogue_split: Some(plan),
                    flow_split: None,
                })
        }
        _ => {
            let SemanticUnit::Flow(flow) = block.unit else {
                return None;
            };
            let target_line_count = (block.content_lines / geometry.line_height).round() as usize;
            let max_top_lines = ((available_lines - effective_spacing) / geometry.line_height)
                .floor() as usize;
            let element_type = match flow.kind {
                crate::pagination::FlowKind::Action => ElementType::Action,
                crate::pagination::FlowKind::SceneHeading => ElementType::SceneHeading,
                crate::pagination::FlowKind::Transition => ElementType::Transition,
                crate::pagination::FlowKind::ColdOpening => ElementType::ColdOpening,
                crate::pagination::FlowKind::NewAct => ElementType::NewAct,
                crate::pagination::FlowKind::EndOfAct => ElementType::EndOfAct,
                crate::pagination::FlowKind::Section => ElementType::Action,
                crate::pagination::FlowKind::Synopsis => ElementType::Action,
            };
            let config = WrapConfig::from_geometry(geometry, element_type);
            let wrapped_lines = wrap_text_for_element(&flow.text, &config);
            if wrapped_lines.len() != target_line_count {
                let lines_that_fit = available_lines - effective_spacing;
                let lines_remaining = block.content_lines - lines_that_fit;

                if lines_remaining >= geometry.widow_limit as f32 {
                    return Some(SplitDecision {
                        top_lines: lines_that_fit,
                        bottom_lines: lines_remaining,
                        dialogue_split: None,
                        flow_split: None,
                    });
                }
                return None;
            }

            let plan = choose_flow_split(
                &flow.text,
                &config,
                max_top_lines,
                geometry.orphan_limit,
                geometry.widow_limit,
            )?;
            let top_lines = plan.top_line_count as f32 * geometry.line_height;
            let bottom_lines = plan.bottom_line_count as f32 * geometry.line_height;

            if bottom_lines >= geometry.widow_limit as f32 * geometry.line_height {
                Some(SplitDecision {
                    top_lines,
                    bottom_lines,
                    dialogue_split: None,
                    flow_split: Some(plan),
                })
            } else {
                None
            }
        }
    }
}

fn chunk_starts_with_transition(chunk: &Chunk<'_>) -> bool {
    chunk.blocks.iter().find(|block| block_has_visible_content(block)).is_some_and(|block| {
        matches!(
            block.unit,
            SemanticUnit::Flow(crate::pagination::FlowUnit {
                kind: crate::pagination::FlowKind::Transition,
                ..
            })
        )
    })
}

fn pull_last_visible_block_for_transition<'a>(
    current_page_blocks: &mut Vec<LayoutBlock<'a>>,
) -> Option<LayoutBlock<'a>> {
    let last_visible_index =
        current_page_blocks.iter().rposition(block_has_visible_content)?;
    let has_earlier_visible_content = current_page_blocks[..last_visible_index]
        .iter()
        .any(block_has_visible_content);

    if !has_earlier_visible_content {
        return None;
    }

    Some(current_page_blocks.remove(last_visible_index))
}
