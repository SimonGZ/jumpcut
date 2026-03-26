use crate::pagination::composer::MeasuredFlowUnit;

pub struct Page {
    pub blocks: Vec<MeasuredFlowUnit>,
}

/// Takes a sequence of globally measured/spaced FlowUnits and distributes them 
/// across fixed-height pages, gracefully handling boundary truncation.
pub fn paginate(blocks: &[MeasuredFlowUnit], page_limit_lines: usize) -> Vec<Page> {
    let mut pages: Vec<Page> = Vec::new();
    let mut current_page_blocks = Vec::new();
    let mut current_page_lines = 0;

    for block in blocks {
        let is_top_of_page = current_page_blocks.is_empty();
        
        // Critical Rule: Elements placed at the absolute top of a page 
        // disregard their intrinsic top visual spacing requirement.
        let effective_spacing = if is_top_of_page {
            0
        } else {
            block.spacing_above
        };

        let block_total_lines = effective_spacing + block.content_lines;

        if current_page_lines + block_total_lines > page_limit_lines {
            // This block visually overflows the current page. We push what we have.
            if !current_page_blocks.is_empty() {
                pages.push(Page { blocks: current_page_blocks });
            }
            
            // The block is now cleanly pushed to the absolute top of the newly minted page.
            // As such, we strip its intrinsic padding per the bounding rule.
            current_page_blocks = vec![MeasuredFlowUnit {
                spacing_above: 0,
                content_lines: block.content_lines,
            }];
            current_page_lines = block.content_lines; 
        } else {
            // The unit cleanly fits on the current page.
            current_page_blocks.push(MeasuredFlowUnit {
                spacing_above: effective_spacing,
                content_lines: block.content_lines,
            });
            current_page_lines += block_total_lines;
        }
    }

    // Flush any remaining blocks inside the final dangling page stream.
    if !current_page_blocks.is_empty() {
        pages.push(Page { blocks: current_page_blocks });
    }

    pages
}
