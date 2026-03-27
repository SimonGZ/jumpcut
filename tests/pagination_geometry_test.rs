use jumpcut::pagination::LayoutGeometry;

#[test]
fn layout_geometry_defaults_match_spec() {
    let geometry = LayoutGeometry::default();
    
    // Asserting against defaults found in docs/pagination-spec.md
    assert_eq!(geometry.action_left, 1.5);
    assert_eq!(geometry.action_right, 7.5);
    assert_eq!(geometry.character_left, 3.5);
    assert_eq!(geometry.character_right, 7.25);
    assert_eq!(geometry.dialogue_left, 2.5);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.parenthetical_left, 3.0);
    assert_eq!(geometry.parenthetical_right, 5.5);
    assert_eq!(geometry.transition_left, 5.5);
    assert_eq!(geometry.transition_right, 7.1);
    assert_eq!(geometry.lyric_left, 2.5);
    // Vertical Spacing Defaults (Blank lines before)
    assert_eq!(geometry.action_spacing_before, 1);
    assert_eq!(geometry.scene_heading_spacing_before, 2);
    assert_eq!(geometry.character_spacing_before, 1);
    assert_eq!(geometry.transition_spacing_before, 1);
    assert_eq!(geometry.lyric_spacing_before, 1);
    
    // Limits
    assert_eq!(geometry.orphan_limit, 2);
    assert_eq!(geometry.widow_limit, 2);
    
    // Spacing
    assert_eq!(geometry.line_height, 1.0);
}

#[test]
fn calculate_element_width_uses_geometry_and_quirks() {
    let geometry = jumpcut::pagination::LayoutGeometry::default();
    use jumpcut::pagination::wrapping::ElementType;
    use jumpcut::pagination::margin::calculate_element_width;

    // Action: 1.5 to 7.5 at 10 CPI is 60. +1 quirk = 61.
    // This will fail to compile as the signature still expects 4 arguments
    assert_eq!(calculate_element_width(&geometry, ElementType::Action), 61);
    
    // Dialogue: 2.5 to 6.0 at 10 CPI is 35. No quirk.
    assert_eq!(calculate_element_width(&geometry, ElementType::Dialogue), 35);
}
