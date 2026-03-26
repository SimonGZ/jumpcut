use crate::pagination::composer::MeasuredFlowUnit;

pub struct Page {
    pub blocks: Vec<MeasuredFlowUnit>,
}

/// An atomic group of blocks that absolutely cannot be split across a page boundary.
struct Chunk<'a> {
    blocks: Vec<&'a MeasuredFlowUnit>,
}

pub fn paginate(blocks: &[MeasuredFlowUnit], page_limit_lines: usize) -> Vec<Page> {
    // Phase 1: Group the raw measured units into indivisible Chunks.
    // If a block has `keep_with_next: true`, it binds to the block immediately following it,
    // forming a continuous chain until a block with `keep_with_next: false` terminates the chunk.
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut current_chunk: Vec<&MeasuredFlowUnit> = Vec::new();

    for block in blocks {
        current_chunk.push(block);
        if !block.keep_with_next {
            chunks.push(Chunk { blocks: current_chunk });
            current_chunk = Vec::new();
        }
    }
    
    // If the document ended on a keep_with_next block (which is technically invalid 
    // screenplay formatting, but we must handle it gracefully), flush it.
    if !current_chunk.is_empty() {
        chunks.push(Chunk { blocks: current_chunk });
    }

    // Phase 2: Distribute the Chunks across pages.
    let mut pages: Vec<Page> = Vec::new();
    let mut current_page_blocks = Vec::new();
    let mut current_page_lines = 0;

    for chunk in chunks {
        // Calculate the total height of this chunk if placed on the current page.
        // The *very first* block in the chunk might have its padding stripped if it lands 
        // at the top of the page. All subsequent blocks in the chunk retain their padding.
        
        let mut chunk_height = 0;
        let is_top_of_page = current_page_blocks.is_empty();
        
        for (i, block) in chunk.blocks.iter().enumerate() {
            let effective_spacing = if is_top_of_page && i == 0 {
                0
            } else {
                block.spacing_above
            };
            chunk_height += effective_spacing + block.content_lines;
        }

        if current_page_lines + chunk_height > page_limit_lines {
            // Check if we can salvage this by splitting cleanly without violating Orphan/Widow lines.
            // For now, we only split simple single-block chunks (like raw Action text).
            if chunk.blocks.len() == 1 && chunk.blocks[0].can_split {
                let block = chunk.blocks[0];
                let effective_spacing = if is_top_of_page { 0 } else { block.spacing_above };
                let available_lines = page_limit_lines.saturating_sub(current_page_lines);
                
                // We MUST have enough space to fulfill the Top Padding PLUS at least 2 Orphan lines
                if available_lines >= effective_spacing + 2 {
                    let lines_that_fit = available_lines - effective_spacing;
                    let lines_remaining = block.content_lines - lines_that_fit;
                    
                    // The Widow falling to the next page MUST also be at least 2 lines!
                    if lines_remaining >= 2 {
                        // Splinter the block!
                        current_page_blocks.push(MeasuredFlowUnit {
                            spacing_above: effective_spacing,
                            content_lines: lines_that_fit,
                            keep_with_next: false,
                            can_split: false,
                            widow_penalty: 0,
                        });
                        
                        pages.push(Page { blocks: current_page_blocks });
                        
                        // The splintered trailing block drops cleanly to the new page margin.
                        current_page_blocks = vec![MeasuredFlowUnit {
                            spacing_above: 0, 
                            // Penalty fulfilled and converted to geometric layout height!
                            content_lines: lines_remaining + block.widow_penalty, 
                            keep_with_next: block.keep_with_next,
                            can_split: block.can_split,
                            widow_penalty: 0, 
                        }];
                        current_page_lines = lines_remaining + block.widow_penalty;
                        continue;
                    }
                }
            }

            // The atomic Chunk overflows and cannot (or shouldn't) be split. We must push the page 
            // and place the entire chunk at the absolute top of the next page.
            if !current_page_blocks.is_empty() {
                pages.push(Page { blocks: current_page_blocks });
            }
            
            current_page_blocks = Vec::new();
            current_page_lines = 0;
            
            // Now that the chunk is at the top of a fresh page, its *first* block's spacing is stripped.
            for (i, block) in chunk.blocks.iter().enumerate() {
                let effective_spacing = if i == 0 { 0 } else { block.spacing_above };
                
                current_page_blocks.push(MeasuredFlowUnit {
                    spacing_above: effective_spacing,
                    content_lines: block.content_lines,
                    keep_with_next: block.keep_with_next,
                    can_split: block.can_split,
                    widow_penalty: block.widow_penalty, // Added widow_penalty here
                });
                
                current_page_lines += effective_spacing + block.content_lines;
            }
        } else {
            // The Chunk cleanly fits on the current page.
            for (i, block) in chunk.blocks.iter().enumerate() {
                let effective_spacing = if is_top_of_page && i == 0 { 0 } else { block.spacing_above };
                
                current_page_blocks.push(MeasuredFlowUnit {
                    spacing_above: effective_spacing,
                    content_lines: block.content_lines,
                    keep_with_next: block.keep_with_next,
                    can_split: block.can_split,
                    widow_penalty: block.widow_penalty,
                });
                
                current_page_lines += effective_spacing + block.content_lines;
            }
        }
    }

    // Flush any remaining blocks inside the final dangling page stream.
    if !current_page_blocks.is_empty() {
        pages.push(Page { blocks: current_page_blocks });
    }

    pages
}
