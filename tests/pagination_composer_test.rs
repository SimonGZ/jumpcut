// tests/pagination_composer_test.rs
use jumpcut::pagination::composer::compose;
use jumpcut::pagination::{Cohesion, FlowKind, FlowUnit, LayoutGeometry, SemanticUnit};

fn mock_action(id: &str, text: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.to_string(),
        kind: FlowKind::Action,
        text: text.to_string(),
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    })
}

fn mock_scene_heading(id: &str, text: &str) -> SemanticUnit {
    SemanticUnit::Flow(FlowUnit {
        element_id: id.to_string(),
        kind: FlowKind::SceneHeading,
        text: text.to_string(),
        line_range: None,
        scene_number: None,
        cohesion: Cohesion {
            keep_together: true,
            keep_with_next: true,
            can_split: false,
        },
    })
}

#[test]
fn composer_determines_correct_non_additive_spacing_between_action_blocks() {
    let geometry = LayoutGeometry::default();
    let units = vec![
        mock_action(
            "el-1",
            "This action runs exactly one horizontal semantic line in width.",
        ),
        mock_action("el-2", "This connects."),
    ];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 2);

    // First element receives its intrinsic demanded spacing
    // because the Paginator (Phase 3) is responsible for stripping top-of-page padding.
    assert_eq!(
        blocks[0].spacing_above, 1.0,
        "Action requires 1 blank line above"
    );
    assert_eq!(blocks[0].content_lines, 2.0);

    assert_eq!(
        blocks[1].spacing_above, 1.0,
        "Action -> Action requires exactly 1 blank line (max of 1 and 1)"
    );
    assert_eq!(blocks[1].content_lines, 1.0);
}

#[test]
fn composer_handles_scene_heading_spacing_rules() {
    let geometry = LayoutGeometry::default();
    let units = vec![
        mock_action("el-1", "Start."),
        mock_scene_heading("el-2", "INT. HOUSE - DAY"),
        mock_action("el-3", "End."),
    ];

    let blocks = compose(&units, &geometry);

    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].content_lines, 1.0);

    // Scene Heading requires 2 lines above
    assert_eq!(
        blocks[1].spacing_above, 2.0,
        "Scene headings require 2 visual blank lines above"
    );
    assert_eq!(blocks[1].content_lines, 1.0);

    // Action requires 1 line above, Scene demands 1 below -> max(1, 1) = 1
    assert_eq!(blocks[2].spacing_above, 1.0);
    assert_eq!(blocks[2].content_lines, 1.0);
}

#[test]
fn composer_respects_custom_geometry() {
    let mut geometry = LayoutGeometry::default();
    // Default action is 1.5 to 7.5 (6.0 inches = 60+1 = 61 chars)
    // Make it much narrower: 1.5 to 3.5 (2.0 inches = 20+1 = 21 chars)
    geometry.action_right = 3.5;

    let text = "This is a long action line that should wrap more frequently.";
    // "This is a long action" -> 21 chars exactly.
    // " line that should wrap" -> 22 chars.
    // " more frequently."
    // Total 3 lines.

    let units = vec![mock_action("el-1", text)];
    let blocks = compose(&units, &geometry);

    assert_eq!(
        blocks[0].content_lines, 3.0,
        "Expected 3 lines with narrower custom geometry"
    );
}

#[test]
fn composer_respects_custom_vertical_spacing() {
    let mut geometry = LayoutGeometry::default();
    // Default action spacing is 1 above. Let's make it 3.
    geometry.action_spacing_before = 3.0;
    
    let units = vec![mock_action("el-1", "Starting action.")];
    let blocks = compose(&units, &geometry);
    
    assert_eq!(blocks[0].spacing_above, 3.0, "Expected 3 lines of spacing above due to custom geometry");
}

#[test]
fn composer_respects_custom_line_height() {
    let mut geometry = LayoutGeometry::default();
    // Default leading is 1.0. Let's make it 2.0 (Double Spaced).
    geometry.line_height = 2.0;
    
    // An action line that usually takes 2 lines of text.
    let text = "This is a sentence that is long enough to wrap onto a second line in the standard 6.0 inch action width.";
    // Check baseline (1.0 leading)
    let baseline_units = vec![mock_action("el-1", text)];
    let baseline_blocks = compose(&baseline_units, &LayoutGeometry::default());
    assert_eq!(baseline_blocks[0].content_lines, 2.0, "Baseline should be 2 lines");

    // Check custom leading (2.0 leading)
    let units = vec![mock_action("el-1", text)];
    let blocks = compose(&units, &geometry);
    
    // 2 text lines * 2.0 leading = 4.0 visual lines.
    assert_eq!(
        blocks[0].content_lines, 4.0, 
        "Expected 4 visual lines for double-spaced 2-line text"
    );
}

#[test]
fn composer_respects_1_5_line_height() {
    let mut geometry = LayoutGeometry::default();
    geometry.line_height = 1.5;
    
    // 1 text line -> 1.5 visual lines (NO MORE CEIL)
    let units_1 = vec![mock_action("el-1", "Short.")];
    let blocks_1 = compose(&units_1, &geometry);
    assert_eq!(blocks_1[0].content_lines, 1.5, "1 text line @ 1.5 -> 1.5 visual lines");

    // 2 text lines -> 3.0 visual lines
    let text_2 = "This is a sentence that is long enough to wrap onto a second line in the standard 6.0 inch action width.";
    let units_2 = vec![mock_action("el-2", text_2)];
    let blocks_2 = compose(&units_2, &geometry);
    assert_eq!(blocks_2[0].content_lines, 3.0, "2 text lines @ 1.5 -> 3.0 visual lines");

    // Multi-line text -> baseline lines * 1.5 = expected
    let mut geometry_narrow = geometry.clone();
    geometry_narrow.action_right = 3.5; // Narrower
    
    // Baseline check for narrow geometry @ 1.0 leading
    let mut narrow_1_0 = geometry_narrow.clone();
    narrow_1_0.line_height = 1.0;
    let units_baseline = vec![mock_action("el-baseline", text_2)];
    let blocks_baseline = compose(&units_baseline, &narrow_1_0);
    let baseline_lines = blocks_baseline[0].content_lines;
    
    let units_3 = vec![mock_action("el-3", text_2)];
    let blocks_3 = compose(&units_3, &geometry_narrow);
    
    // baseline_lines * 1.5 = expected (f32)
    let expected = baseline_lines * 1.5;
    assert_eq!(
        blocks_3[0].content_lines, expected, 
        "Expected {} visual lines for {} text lines @ 1.5", expected, baseline_lines
    );
}
