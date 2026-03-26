// tests/pagination_paginator_test.rs
use jumpcut::pagination::composer::LayoutBlock;
use jumpcut::pagination::paginator::paginate;
use jumpcut::pagination::{SemanticUnit, PageStartUnit};
use jumpcut::pagination::Fragment;

fn mock_block<'a>(unit: &'a SemanticUnit, lines: usize, padding: usize, keep_with_next: bool, can_split: bool, widow_penalty: usize) -> LayoutBlock<'a> {
    LayoutBlock {
        unit,
        fragment: Fragment::Whole,
        spacing_above: padding,
        content_lines: lines,
        keep_with_next,
        can_split,
        widow_penalty,
    }
}

#[test]
fn paginator_distributes_blocks_across_pages_when_they_exceed_limits() {
    // Standard screenplay page is 54 playable lines
    let page_limit = 54;
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    
    let blocks = vec![
        mock_block(&unit1, 50, 0, false, false, 0), // Page 1 takes 50 lines
        // Next block requires 10 lines + 1 padding = 11 lines.
        // 50 + 11 = 61. This exceeds 54, so it should roll ENTIRELY onto Page 2.
        mock_block(&unit2, 10, 1, false, false, 0), 
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
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    
    let blocks = vec![
        mock_block(&unit1, 54, 0, false, false, 0), // Perfectly fills Page 1 to the absolute brim
        mock_block(&unit2, 5, 2, false, false, 0),  // A Scene Heading-like block demanding 2 lines of padding above
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
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    let unit3 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "3".into() });
    
    let blocks = vec![
        mock_block(&unit1, 50, 0, false, false, 0), // Fills lines 1-50
        
        // A scene heading needs 2 lines of padding and 1 line of content. 
        // 50 + 2 + 1 = 53 lines. This technically FITS snugly at the bottom of Page 1!
        mock_block(&unit2, 1, 2, true, false, 0), 
        
        // But the NEXT block demands 1 line padding + 2 lines content = 3 lines.
        // 53 + 3 = 56 lines. This forces the Action block onto Page 2.
        // Since the Scene Heading is marked `keep_with_next`, it must forfeit its 
        // spot on Page 1 and fall to Page 2 along with the Action block.
        mock_block(&unit3, 2, 1, false, false, 0), 
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

#[test]
fn paginator_splits_splittable_blocks_while_respecting_orphan_widow_limits() {
    let page_limit = 54;
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    
    let blocks = vec![
        mock_block(&unit1, 51, 0, false, false, 0), // Fills 51 lines (3 lines remaining on Page 1)
        mock_block(&unit2, 5, 1, false, true, 0),   // 5 line action block. Requires 1 padding. 
                                         // Remaining space = 54 - 51 = 3 lines.
                                         // Target takes 1 padding + 2 content = 3 lines on Page 1.
                                         // It pushes 3 content lines to Page 2.
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 2, "Page 1 contains the 51-line block and the top half of the split block");
    assert_eq!(pages[0].blocks[1].content_lines, 2, "2 lines of the split block fit on Page 1 (Orphan limit respected)");
    
    assert_eq!(pages[1].blocks.len(), 1, "Page 2 gets the bottom half of the split block");
    assert_eq!(pages[1].blocks[0].spacing_above, 0, "Top padding stripped for the widow piece");
    assert_eq!(pages[1].blocks[0].content_lines, 3, "Remaining 3 lines push to Page 2");
}

#[test]
fn paginator_rejects_splits_that_violate_orphan_limits_and_pushes_entire_block() {
    let page_limit = 54;
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    
    let blocks = vec![
        mock_block(&unit1, 53, 0, false, false, 0), // Fills exactly 53 lines (1 line remaining)
        mock_block(&unit2, 4, 0, false, true, 0),   // 4 line block with 0 padding. can_split = true!
                                         // Only 1 line fits on Page 1.
                                         // But Orphan limit is 2 lines!
                                         // Split is REJECTED. The whole block goes to Page 2.
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].blocks.len(), 1, "Page 1 holds the 53-line block, leaves the 1 remaining line blank");
    
    assert_eq!(pages[1].blocks.len(), 1, "The entire 4-line block is pushed to Page 2 to avoid a 1-line orphan");
    assert_eq!(pages[1].blocks[0].content_lines, 4);
}

#[test]
fn paginator_accounts_for_additional_widow_penalty_lines_when_splitting_dialogue() {
    let page_limit = 54;
    let unit1 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "1".into() });
    let unit2 = SemanticUnit::PageStart(PageStartUnit { source_element_id: "2".into() });
    
    let blocks = vec![
        mock_block(&unit1, 50, 0, false, false, 0), // Fills 50 lines (4 lines remaining on Page 1)
        mock_block(&unit2, 7, 1, false, true, 1),   // 7 line dialogue block with 1 line padding above.
                                            // 4 lines remaining: 1 padding + 3 content lines fit on Page 1.
                                            // The remaining 4 content lines push to Page 2.
                                            // BUT this block has a widow_penalty of 1 (for the CONT'D character header).
                                            // So the trailing widow block on Page 2 should have 4 + 1 = 5 content lines!
    ];

    let pages = paginate(&blocks, page_limit);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 2);
    assert_eq!(pages[0].blocks[1].content_lines, 3, "3 lines of the split dialogue fit on Page 1");
    
    // The widow block on Page 2 receives the structural penalty 
    assert_eq!(pages[1].blocks[0].content_lines, 5, "Remaining 4 lines + 1 penalty = 5 lines on Page 2");
}
