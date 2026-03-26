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
        blocks[0].spacing_above, 1,
        "Action requires 1 blank line above"
    );
    assert_eq!(blocks[0].content_lines, 2);

    assert_eq!(
        blocks[1].spacing_above, 1,
        "Action -> Action requires exactly 1 blank line (max of 1 and 1)"
    );
    assert_eq!(blocks[1].content_lines, 1);
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
    assert_eq!(blocks[0].content_lines, 1);

    // Scene Heading requires 2 lines above
    assert_eq!(
        blocks[1].spacing_above, 2,
        "Scene headings require 2 visual blank lines above"
    );
    assert_eq!(blocks[1].content_lines, 1);

    // Action requires 1 line above, Scene demands 1 below -> max(1, 1) = 1
    assert_eq!(blocks[2].spacing_above, 1);
    assert_eq!(blocks[2].content_lines, 1);
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
        blocks[0].content_lines, 3,
        "Expected 3 lines with narrower custom geometry"
    );
}
