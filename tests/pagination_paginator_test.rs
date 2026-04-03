// tests/pagination_paginator_test.rs
use jumpcut::pagination::composer::LayoutBlock;
use jumpcut::pagination::paginator::paginate;
use jumpcut::pagination::{
    Cohesion, FlowKind, FlowUnit, Fragment, LayoutGeometry, PageStartUnit, SemanticUnit,
};

fn mock_block<'a>(unit: &'a SemanticUnit, lines: f32, padding: f32, keep_with_next: bool, can_split: bool, widow_penalty: f32) -> LayoutBlock<'a> {
    LayoutBlock {
        unit,
        fragment: Fragment::Whole,
        spacing_above: padding,
        content_lines: lines,
        dialogue_split: None,
        flow_split: None,
        keep_with_next,
        can_split,
        widow_penalty,
    }
}

fn visible_unit(id: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.into(),
        kind: FlowKind::Action,
        text: format!("dummy-{id}"),
        inline_text: None,
        centered: false,
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

fn transition_unit(id: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.into(),
        kind: FlowKind::Transition,
        text: format!("CUT TO: {id}"),
        inline_text: None,
        centered: false,
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

#[test]
fn paginator_distributes_blocks_across_pages_when_they_exceed_limits() {
    let geometry = LayoutGeometry::default();
    // Standard screenplay page is 54 playable lines
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 50.0, 0.0, false, false, 0.0), // Page 1 takes 50 lines
        // Next block requires 10 lines + 1 padding = 11 lines.
        // 50 + 11 = 61. This exceeds 54, so it should roll ENTIRELY onto Page 2.
        mock_block(&unit2, 10.0, 1.0, false, false, 0.0), 
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2, "Expected exactly two pages of output");
    assert_eq!(pages[0].blocks.len(), 1, "First 50-line block fills Page 1");
    // Ensure the spilled block lands on Page 2
    assert_eq!(pages[1].blocks.len(), 1, "Second block rolls cleanly to Page 2");
}

#[test]
fn paginator_strips_intrinsic_padding_from_elements_landing_at_the_top_of_a_page() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 54.0, 0.0, false, false, 0.0), // Perfectly fills Page 1 to the absolute brim
        mock_block(&unit2, 5.0, 2.0, false, false, 0.0),  // A Scene Heading-like block demanding 2 lines of padding above
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2);
    
    // Critical Spec Rule: Elements placed at the absolute top of a page 
    // disregard their intrinsic top visual spacing requirement.
    assert_eq!(pages[1].blocks[0].spacing_above, 0.0, "Top padding must be stripped to 0 at page boundaries");
    assert_eq!(pages[1].blocks[0].content_lines, 5.0);
}

#[test]
fn paginator_ignores_page_start_markers_when_stripping_top_of_page_spacing() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let page_start = SemanticUnit::PageStart(PageStartUnit { source_element_id: "page-start".into() });
    let scene_heading = visible_unit("scene-heading");

    let blocks = vec![
        mock_block(&page_start, 0.0, 0.0, false, false, 0.0),
        mock_block(&scene_heading, 1.0, 2.0, false, false, 0.0),
    ];

    let pages = paginate(&blocks, page_limit, &geometry);

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].blocks.len(), 2);
    assert_eq!(
        pages[0].blocks[1].spacing_above,
        0.0,
        "the first visible block on a page should strip intrinsic spacing even if a PageStart marker precedes it",
    );
}

#[test]
fn paginator_starts_a_new_page_when_it_encounters_a_page_start_marker() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let before_break = visible_unit("before-break");
    let page_start = SemanticUnit::PageStart(PageStartUnit { source_element_id: "page-start".into() });
    let after_break = visible_unit("after-break");

    let blocks = vec![
        mock_block(&before_break, 2.0, 0.0, false, false, 0.0),
        mock_block(&page_start, 0.0, 0.0, false, false, 0.0),
        mock_block(&after_break, 1.0, 2.0, false, false, 0.0),
    ];

    let pages = paginate(&blocks, page_limit, &geometry);

    assert_eq!(pages.len(), 2, "PageStart should force a new page even when plenty of space remains");
    assert_eq!(pages[0].blocks.len(), 1, "The visible content before the PageStart should stay on page 1");
    assert_eq!(pages[1].blocks.len(), 2, "Page 2 should contain the PageStart marker and the following visible block");
    assert_eq!(pages[1].blocks[1].spacing_above, 0.0, "The first visible block after a forced page break should strip intrinsic spacing");
}

#[test]
fn paginator_prevents_stranding_blocks_that_require_keep_with_next() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    let unit3 = visible_unit("3");
    
    let blocks = vec![
        mock_block(&unit1, 50.0, 0.0, false, false, 0.0), // Fills lines 1-50
        
        // A scene heading needs 2 lines of padding and 1 line of content. 
        // 50 + 2 + 1 = 53 lines. This technically FITS snugly at the bottom of Page 1!
        mock_block(&unit2, 1.0, 2.0, true, false, 0.0), 
        
        // But the NEXT block demands 1 line padding + 2 lines content = 3 lines.
        // 53 + 3 = 56 lines. This forces the Action block onto Page 2.
        // Since the Scene Heading is marked `keep_with_next`, it must forfeit its 
        // spot on Page 1 and fall to Page 2 along with the Action block.
        mock_block(&unit3, 2.0, 1.0, false, false, 0.0), 
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 1, "Page 1 should only contain the first block because the Scene Heading was pushed");
    assert_eq!(pages[0].blocks[0].content_lines, 50.0);

    // The Scene Heading should have been pushed entirely to Page 2!
    assert_eq!(pages[1].blocks.len(), 2, "Page 2 should contain the Scene Heading and the Action block");
    
    // Top-of-page rule applies to the pushed Scene Heading!
    assert_eq!(pages[1].blocks[0].spacing_above, 0.0, "Scene Heading was pushed to the top of Page 2, so padding is stripped");
    assert_eq!(pages[1].blocks[0].content_lines, 1.0);
    
    // The Action block retains its spacing from the Scene Heading
    assert_eq!(pages[1].blocks[1].spacing_above, 1.0);
    assert_eq!(pages[1].blocks[1].content_lines, 2.0);
}

#[test]
fn paginator_keeps_scene_heading_and_splits_splittable_following_block() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let filler = visible_unit("filler");
    let scene_heading = SemanticUnit::Flow(FlowUnit {
        element_id: "scene-heading".into(),
        kind: FlowKind::SceneHeading,
        text: "INT. ROOM - DAY".into(),
        inline_text: None,
        centered: false,
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: true,
            keep_with_next: true,
            can_split: false,
        },
    });
    let action = SemanticUnit::Flow(FlowUnit {
        element_id: "action".into(),
        kind: FlowKind::Action,
        text: "Z ygeb ujazopoda ovepaj kequgarajar yvy uk ok uje Eroryheg ozejy. Udaxek, eb eky raxusore asazypo, useb ys z habu udybyzu rezas yvy uk OVEHO'U UJUQY RYGYVAKY. Hoz ryxu wyde qesy yvy zo rus ebeba qe qaryhyj: Ok uwokeby, qaba gywusyw, yzyv qoso, kud z kyvup qabex apokakeh.".into(),
        inline_text: None,
        centered: false,
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    });

    let blocks = vec![
        mock_block(&filler, 46.0, 0.0, false, false, 0.0),
        mock_block(&scene_heading, 1.0, 2.0, true, false, 0.0),
        mock_block(&action, 5.0, 1.0, false, true, 0.0),
    ];

    let pages = paginate(&blocks, page_limit, &geometry);

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].blocks.len(), 3);
    assert_eq!(pages[0].blocks[1].spacing_above, 2.0);
    assert_eq!(pages[0].blocks[1].content_lines, 1.0);
    assert_eq!(pages[0].blocks[2].fragment, Fragment::ContinuedToNext);
    assert_eq!(pages[0].blocks[2].content_lines, 3.0);

    assert_eq!(pages[1].blocks.len(), 1);
    assert_eq!(pages[1].blocks[0].fragment, Fragment::ContinuedFromPrev);
    assert_eq!(pages[1].blocks[0].spacing_above, 0.0);
    assert_eq!(pages[1].blocks[0].content_lines, 2.0);
}

#[test]
fn paginator_pulls_a_transition_off_the_top_of_the_next_page() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    let transition = transition_unit("cut");

    let blocks = vec![
        mock_block(&unit1, 49.0, 0.0, false, false, 0.0),
        mock_block(&unit2, 4.0, 0.0, false, false, 0.0),
        mock_block(&transition, 1.0, 1.0, false, false, 0.0),
    ];

    let pages = paginate(&blocks, page_limit, &geometry);

    assert_eq!(pages.len(), 2);
    assert_eq!(
        pages[0].blocks.len(),
        1,
        "The paginator should leave extra white space on page 1 instead of stranding a transition at the top of page 2",
    );
    assert_eq!(pages[1].blocks.len(), 2);
    assert!(matches!(pages[1].blocks[0].unit, SemanticUnit::Flow(FlowUnit { kind: FlowKind::Action, .. })));
    assert!(matches!(pages[1].blocks[1].unit, SemanticUnit::Flow(FlowUnit { kind: FlowKind::Transition, .. })));
    assert_eq!(pages[1].blocks[0].spacing_above, 0.0);
    assert_eq!(
        pages[1].blocks[1].spacing_above,
        1.0,
        "The transition should keep its intrinsic spacing once it is no longer the first visible block on the page",
    );
}

#[test]
fn paginator_splits_splittable_blocks_while_respecting_orphan_widow_limits() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 51.0, 0.0, false, false, 0.0), // Fills 51 lines (3 lines remaining on Page 1)
        mock_block(&unit2, 5.0, 1.0, false, true, 0.0),   // 5 line action block. Requires 1 padding. 
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 2, "Page 1 contains the 51-line block and the top half of the split block");
    assert_eq!(pages[0].blocks[1].content_lines, 2.0, "2 lines of the split block fit on Page 1 (Orphan limit respected)");
    
    assert_eq!(pages[1].blocks.len(), 1, "Page 2 gets the bottom half of the split block");
    assert_eq!(pages[1].blocks[0].spacing_above, 0.0, "Top padding stripped for the widow piece");
    assert_eq!(pages[1].blocks[0].content_lines, 3.0, "Remaining 3 lines push to Page 2");
}

#[test]
fn paginator_rejects_splits_that_violate_orphan_limits_and_pushes_entire_block() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 53.0, 0.0, false, false, 0.0), // Fills exactly 53 lines (1 line remaining)
        mock_block(&unit2, 4.0, 0.0, false, true, 0.0),   // 4 line block with 0 padding. can_split = true!
                                         // Only 1 line fits on Page 1.
                                         // But Orphan limit is 2 lines!
                                         // Split is REJECTED. The whole block goes to Page 2.
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].blocks.len(), 1, "Page 1 holds the 53-line block, leaves the 1 remaining line blank");
    
    assert_eq!(pages[1].blocks.len(), 1, "The entire 4-line block is pushed to Page 2 to avoid a 1-line orphan");
    assert_eq!(pages[1].blocks[0].content_lines, 4.0);
}

#[test]
fn paginator_accounts_for_additional_widow_penalty_lines_when_splitting_dialogue() {
    let geometry = LayoutGeometry::default();
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 50.0, 0.0, false, false, 0.0), // Fills 50 lines (4 lines remaining on Page 1)
        mock_block(&unit2, 7.0, 1.0, false, true, 1.0),   // 7 line dialogue block with 1 line padding above.
                                            // 4 lines remaining: 1 padding + 3 content lines fit on Page 1.
                                            // The remaining 4 content lines push to Page 2.
                                            // BUT this block has a widow_penalty of 1.0 (for the CONT'D character header).
                                            // So the trailing widow block on Page 2 should have 4 + 1 = 5 content lines!
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 2);
    
    assert_eq!(pages[0].blocks.len(), 2);
    assert_eq!(pages[0].blocks[1].content_lines, 3.0, "3 lines of the split dialogue fit on Page 1");
    
    // The widow block on Page 2 receives the structural penalty 
    assert_eq!(pages[1].blocks[0].content_lines, 5.0, "Remaining 4 lines + 1 penalty = 5 lines on Page 2");
}

#[test]
fn paginator_respects_custom_orphan_widow_limits() {
    let mut geometry = LayoutGeometry::default();
    geometry.orphan_limit = 4; // Much higher than default 2
    geometry.widow_limit = 4;
    
    let page_limit = 54.0;
    let unit1 = visible_unit("1");
    let unit2 = visible_unit("2");
    
    let blocks = vec![
        mock_block(&unit1, 50.0, 0.0, false, false, 0.0), // 4 lines remaining on Page 1
        mock_block(&unit2, 10.0, 0.0, false, true, 0.0),  // 10 line block. 
                                         // Technically 4 lines fit on Page 1.
                                         // Since orphan_limit is 4, it should SPLIT exactly at 4.
    ];

    let pages = paginate(&blocks, page_limit, &geometry);
    assert_eq!(pages[0].blocks.len(), 2, "Should split exactly at the 4-line orphan limit");

    // Case B: Only 3 lines available on Page 1. Orphan limit is 4. Should NOT split.
    let blocks2 = vec![
        mock_block(&unit1, 51.0, 0.0, false, false, 0.0), // 3 lines remaining
        mock_block(&unit2, 10.0, 0.0, false, true, 0.0),
    ];
    let pages2 = paginate(&blocks2, page_limit, &geometry);
    assert_eq!(pages2[0].blocks.len(), 1, "Should NOT split if available space (3.0) < orphan limit (4.0)");
}

#[test]
fn paginator_verifies_final_draft_parity_for_1_5_leading() {
    let geometry = LayoutGeometry::default(); // line_height handled in content_lines manually for mock
    let page_limit = 54.0;
    
    // 36 blocks of 1.5 lines each = 54.0 total.
    // They should fit EXACTLY on one 54.0-line page without a spill.
    let mut blocks = Vec::new();
    let units: Vec<SemanticUnit> = (0..36).map(|i| {
        visible_unit(&format!("el-{}", i))
    }).collect();

    for unit in &units {
        blocks.push(mock_block(unit, 1.5, 0.0, false, false, 0.0));
    }

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 1, "36 lines @ 1.5 leading (54.0 total) must fit on a single 54-line page");
    assert_eq!(pages[0].blocks.len(), 36, "All 36 blocks should be on Page 1");
}

#[test]
fn paginator_verifies_final_draft_parity_for_2_0_leading() {
    let geometry = LayoutGeometry::default(); // line_height handled in content_lines manually for mock
    let page_limit = 54.0;
    
    // 27 blocks of 2.0 lines each = 54.0 total.
    // They should fit EXACTLY on one 54.0-line page without a spill.
    let mut blocks = Vec::new();
    let units: Vec<SemanticUnit> = (0..27).map(|i| {
        visible_unit(&format!("el-{}", i))
    }).collect();

    for unit in &units {
        blocks.push(mock_block(unit, 2.0, 0.0, false, false, 0.0));
    }

    let pages = paginate(&blocks, page_limit, &geometry);
    
    assert_eq!(pages.len(), 1, "27 lines @ 2.0 leading (54.0 total) must fit on a single 54-line page");
    assert_eq!(pages[0].blocks.len(), 27, "All 27 blocks should be on Page 1");
}
