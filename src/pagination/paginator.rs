use crate::pagination::composer::LayoutBlock;
use crate::pagination::dialogue_split::{
    choose_dialogue_split, DialogueLine, DialogueLineRole,
};
use crate::pagination::flow_split::choose_flow_split;
use crate::pagination::fixtures::Fragment;
use crate::pagination::wrapping::{wrap_text_for_element, ElementType, WrapConfig};
use crate::pagination::{DialoguePartKind, SemanticUnit};
use crate::pagination::LayoutGeometry;

pub struct Page<'a> {
    pub blocks: Vec<LayoutBlock<'a>>,
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

                if let Some((top_lines, bottom_lines)) =
                    choose_split_lines(block, available_lines, effective_spacing, geometry)
                {
                    current_page_blocks.push(LayoutBlock {
                        unit: block.unit,
                        fragment: Fragment::ContinuedToNext,
                        spacing_above: effective_spacing,
                        content_lines: top_lines,
                        keep_with_next: false,
                        can_split: false,
                        widow_penalty: 0.0,
                    });

                    pages.push(Page { blocks: current_page_blocks });

                    current_page_blocks = vec![LayoutBlock {
                        unit: block.unit,
                        fragment: Fragment::ContinuedFromPrev,
                        spacing_above: 0.0,
                        content_lines: bottom_lines + block.widow_penalty,
                        keep_with_next: block.keep_with_next,
                        can_split: block.can_split,
                        widow_penalty: 0.0,
                    }];
                    current_page_lines = bottom_lines + block.widow_penalty;
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
) -> Option<(f32, f32)> {
    if available_lines < effective_spacing + geometry.orphan_limit as f32 {
        return None;
    }

    match block.unit {
        SemanticUnit::Dialogue(dialogue) => {
            let max_top_lines = ((available_lines - effective_spacing) / geometry.line_height)
                .floor() as usize;
            let dialogue_lines = dialogue
                .parts
                .iter()
                .flat_map(|part| {
                    let element_type = match part.kind {
                        DialoguePartKind::Character => ElementType::Character,
                        DialoguePartKind::Parenthetical => ElementType::Parenthetical,
                        DialoguePartKind::Dialogue => ElementType::Dialogue,
                        DialoguePartKind::Lyric => ElementType::Lyric,
                    };
                    let config = WrapConfig::from_geometry(geometry, element_type);
                    wrap_text_for_element(&part.text, &config)
                        .into_iter()
                        .map(move |line| DialogueLine {
                            role: match part.kind {
                                DialoguePartKind::Character => DialogueLineRole::Character,
                                DialoguePartKind::Parenthetical => DialogueLineRole::Parenthetical,
                                DialoguePartKind::Dialogue | DialoguePartKind::Lyric => DialogueLineRole::Dialogue,
                            },
                            text: line,
                        })
                })
                .collect::<Vec<_>>();
            let continuation_prefix_lines =
                dialogue_continuation_prefix_line_count(dialogue, geometry);

            let decision = choose_dialogue_split(
                &dialogue_lines,
                max_top_lines,
                geometry.orphan_limit,
                geometry.widow_limit,
            )?;
            let top_lines = decision.top_line_count as f32 * geometry.line_height;
            let bottom_dialogue_lines = block.content_lines - top_lines;
            let bottom_lines = bottom_dialogue_lines
                + continuation_prefix_lines as f32 * geometry.line_height;
            (bottom_lines >= geometry.widow_limit as f32 * geometry.line_height)
                .then_some((top_lines, bottom_lines))
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
                    return Some((lines_that_fit, lines_remaining));
                }
                return None;
            }

            let decision = choose_flow_split(
                &wrapped_lines,
                max_top_lines,
                geometry.orphan_limit,
                geometry.widow_limit,
            )?;
            let top_lines = decision.top_line_count as f32 * geometry.line_height;
            let bottom_lines = block.content_lines - top_lines;

            if bottom_lines >= geometry.widow_limit as f32 * geometry.line_height {
                Some((top_lines, bottom_lines))
            } else {
                None
            }
        }
    }
}

fn dialogue_continuation_prefix_line_count(
    dialogue: &crate::pagination::DialogueUnit,
    geometry: &LayoutGeometry,
) -> usize {
    dialogue
        .parts
        .iter()
        .take_while(|part| matches!(part.kind, DialoguePartKind::Character))
        .map(|part| {
            let config = WrapConfig::from_geometry(geometry, ElementType::Character);
            wrap_text_for_element(&part.text, &config).len()
        })
        .sum()
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
