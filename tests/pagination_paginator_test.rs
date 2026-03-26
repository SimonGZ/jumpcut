// tests/pagination_paginator_test.rs
use jumpcut::pagination::composer::MeasuredFlowUnit;
use jumpcut::pagination::paginator::paginate;

fn mock_block(lines: usize, padding: usize) -> MeasuredFlowUnit {
    MeasuredFlowUnit {
        spacing_above: padding,
        content_lines: lines,
    }
}

#[test]
fn paginator_distributes_blocks_across_pages_when_they_exceed_limits() {
    // Standard screenplay page is 54 playable lines
    let page_limit = 54;
    
    let blocks = vec![
        mock_block(50, 0), // Page 1 takes 50 lines
        // Next block requires 10 lines + 1 padding = 11 lines.
        // 50 + 11 = 61. This exceeds 54, so it should roll ENTIRELY onto Page 2.
        mock_block(10, 1), 
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2, "Expected exactly two pages of output");
    assert_eq!(pages[0].blocks.len(), 1, "First 50-line block fills Page 1");
    // Ensure the spilled block lands on Page 2
    assert_eq!(pages[1].blocks.len(), 1, "Second block rolls cleanly to Page 2");
}

#[test]
fn paginator_strips_intrinsic_padding_from_elements_landing_at_the_top_of_a_page() {
    let page_limit = 54;
    
    let blocks = vec![
        mock_block(54, 0), // Perfectly fills Page 1 to the absolute brim
        mock_block(5, 2),  // A Scene Heading-like block demanding 2 lines of padding above
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    
    // Critical Spec Rule: Elements placed at the absolute top of a page 
    // disregard their intrinsic top visual spacing requirement.
    assert_eq!(pages[1].blocks[0].spacing_above, 0, "Top padding must be stripped to 0 at page boundaries");
    assert_eq!(pages[1].blocks[0].content_lines, 5);
}
