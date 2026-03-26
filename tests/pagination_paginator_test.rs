// tests/pagination_paginator_test.rs
use jumpcut::pagination::composer::MeasuredFlowUnit;
use jumpcut::pagination::paginator::paginate;

fn mock_block(lines: usize, padding: usize, keep_with_next: bool) -> MeasuredFlowUnit {
    MeasuredFlowUnit {
        spacing_above: padding,
        content_lines: lines,
        keep_with_next,
    }
}

#[test]
fn paginator_distributes_blocks_across_pages_when_they_exceed_limits() {
    // Standard screenplay page is 54 playable lines
    let page_limit = 54;
    
    let blocks = vec![
        mock_block(50, 0, false), // Page 1 takes 50 lines
        // Next block requires 10 lines + 1 padding = 11 lines.
        // 50 + 11 = 61. This exceeds 54, so it should roll ENTIRELY onto Page 2.
        mock_block(10, 1, false), 
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
        mock_block(54, 0, false), // Perfectly fills Page 1 to the absolute brim
        mock_block(5, 2, false),  // A Scene Heading-like block demanding 2 lines of padding above
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    
    // Critical Spec Rule: Elements placed at the absolute top of a page 
    // disregard their intrinsic top visual spacing requirement.
    assert_eq!(pages[1].blocks[0].spacing_above, 0, "Top padding must be stripped to 0 at page boundaries");
    assert_eq!(pages[1].blocks[0].content_lines, 5);
}

#[test]
fn paginator_prevents_stranding_blocks_that_require_keep_with_next() {
    let page_limit = 54;
    
    let blocks = vec![
        mock_block(50, 0, false), // Fills lines 1-50
        
        // A scene heading needs 2 lines of padding and 1 line of content. 
        // 50 + 2 + 1 = 53 lines. This technically FITS snugly at the bottom of Page 1!
        mock_block(1, 2, true), 
        
        // But the NEXT block demands 1 line padding + 2 lines content = 3 lines.
        // 53 + 3 = 56 lines. This forces the Action block onto Page 2.
        // Since the Scene Heading is marked `keep_with_next`, it must forfeit its 
        // spot on Page 1 and fall to Page 2 along with the Action block.
        mock_block(2, 1, false), 
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 1, "Page 1 should only contain the first block because the Scene Heading was pushed");
    assert_eq!(pages[0].blocks[0].content_lines, 50);

    // The Scene Heading should have been pushed entirely to Page 2!
    assert_eq!(pages[1].blocks.len(), 2, "Page 2 should contain the Scene Heading and the Action block");
    
    // Top-of-page rule applies to the pushed Scene Heading!
    assert_eq!(pages[1].blocks[0].spacing_above, 0, "Scene Heading was pushed to the top of Page 2, so padding is stripped");
    assert_eq!(pages[1].blocks[0].content_lines, 1);
    
    // The Action block retains its spacing from the Scene Heading
    assert_eq!(pages[1].blocks[1].spacing_above, 1);
    assert_eq!(pages[1].blocks[1].content_lines, 2);
}
