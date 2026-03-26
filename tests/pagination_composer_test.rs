// tests/pagination_composer_test.rs
use jumpcut::pagination::composer::compose;
use jumpcut::pagination::{Cohesion, FlowKind, FlowUnit, SemanticUnit};

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
    let units = vec![
        mock_action("el-1", "This action runs exactly one horizontal semantic line in width."),
        mock_action("el-2", "This connects."),
    ];

    let blocks = compose(&units);
    
    assert_eq!(blocks.len(), 2);
    
    // First element receives its intrinsic demanded spacing 
    // because the Paginator (Phase 3) is responsible for stripping top-of-page padding.
    assert_eq!(blocks[0].spacing_above, 1, "Action requires 1 blank line above");
    assert_eq!(blocks[0].content_lines, 2);
    
    assert_eq!(blocks[1].spacing_above, 1, "Action -> Action requires exactly 1 blank line (max of 1 and 1)");
    assert_eq!(blocks[1].content_lines, 1);
}

#[test]
fn composer_handles_scene_heading_spacing_rules() {
    let units = vec![
        mock_action("el-1", "Start."),
        mock_scene_heading("el-2", "INT. HOUSE - DAY"),
        mock_action("el-3", "End."),
    ];

    let blocks = compose(&units);
    
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].content_lines, 1); 
    
    // Scene Heading requires 2 lines above
    assert_eq!(blocks[1].spacing_above, 2, "Scene headings require 2 visual blank lines above");
    assert_eq!(blocks[1].content_lines, 1);
    
    // Action requires 1 line above, Scene demands 1 below -> max(1, 1) = 1
    assert_eq!(blocks[2].spacing_above, 1);
    assert_eq!(blocks[2].content_lines, 1);
}
