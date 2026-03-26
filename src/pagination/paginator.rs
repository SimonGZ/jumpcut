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
            // The atomic Chunk overflows the current page. We must push the page 
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
