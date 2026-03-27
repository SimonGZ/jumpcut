use crate::pagination::composer::LayoutBlock;
use crate::pagination::fixtures::Fragment;
use crate::pagination::SemanticUnit;
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
                
                if available_lines >= effective_spacing + geometry.orphan_limit as f32 {
                    let lines_that_fit = available_lines - effective_spacing;
                    let lines_remaining = block.content_lines - lines_that_fit;
                    
                    if lines_remaining >= geometry.widow_limit as f32 {
                        current_page_blocks.push(LayoutBlock {
                            unit: block.unit,
                            fragment: Fragment::ContinuedToNext,
                            spacing_above: effective_spacing,
                            content_lines: lines_that_fit,
                            keep_with_next: false,
                            can_split: false,
                            widow_penalty: 0.0,
                        });
                        
                        pages.push(Page { blocks: current_page_blocks });
                        
                        current_page_blocks = vec![LayoutBlock {
                            unit: block.unit,
                            fragment: Fragment::ContinuedFromPrev,
                            spacing_above: 0.0, 
                            content_lines: lines_remaining + block.widow_penalty, 
                            keep_with_next: block.keep_with_next,
                            can_split: block.can_split,
                            widow_penalty: 0.0, 
                        }];
                        current_page_lines = lines_remaining + block.widow_penalty;
                        continue;
                    }
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
