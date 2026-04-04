// tests/pagination_margin_test.rs
use jumpcut::pagination::margin::{calculate_element_width, dual_dialogue_character_left_indent};
use jumpcut::pagination::wrapping::ElementType;
use jumpcut::pagination::{Alignment, FdxExtractedSettings, FdxParagraphStyle, LayoutGeometry};
use std::collections::BTreeMap;

#[test]
fn action_margin_calculation_adds_inclusive_character_quirk() {
    let geometry = LayoutGeometry::default();
    // 1.5 to 7.5 at 10 CPI is mathematically 60 (6.0 * 10).
    // Action gets a +1 quirk to cleanly fit 61 characters.
    assert_eq!(calculate_element_width(&geometry, ElementType::Action), 61);
}

#[test]
fn scene_heading_margin_calculation_matches_action_width_quirk() {
    let geometry = LayoutGeometry::default();
    // Scene heading shares the full-width 1.5 to 7.5 margin family.
    assert_eq!(
        calculate_element_width(&geometry, ElementType::SceneHeading),
        61
    );
}

#[test]
fn dialogue_margin_calculation_is_exact_math() {
    let geometry = LayoutGeometry::default();
    // 2.5 to 6.0 at 10 CPI is mathematically 35 (3.5 * 10).
    // Dialogue does not receive the +1 quirk.
    assert_eq!(
        calculate_element_width(&geometry, ElementType::Dialogue),
        35
    );
}

#[test]
fn character_margin_calculation() {
    let geometry = LayoutGeometry::default();
    // 3.5 to 7.25 at 10 CPI is mathematically 37.5.
    // General policy uses ordinary rounding, with no special +1 quirk here.
    assert_eq!(
        calculate_element_width(&geometry, ElementType::Character),
        38
    );
}

#[test]
fn lyric_margin_calculation() {
    let geometry = LayoutGeometry::default();
    // 2.5 to 7.375 at 10 CPI is mathematically 48.75.
    // General policy uses ordinary rounding, with no special +1 quirk here.
    assert_eq!(calculate_element_width(&geometry, ElementType::Lyric), 49);
}

#[test]
fn dual_dialogue_margin_calculation_uses_normal_rounding_without_a_special_quirk() {
    let geometry = LayoutGeometry::default();

    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueLeft),
        29
    );
    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueRight),
        29
    );
    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueCharacterLeft),
        29
    );
    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueCharacterRight),
        29
    );
}

#[test]
fn dual_dialogue_character_left_indent_matches_final_draft_probe_points() {
    assert_eq!(dual_dialogue_character_left_indent("A", 1), 2.875);
    assert_eq!(dual_dialogue_character_left_indent("AB", 1), 2.8125);
    assert_eq!(dual_dialogue_character_left_indent("MARK", 1), 2.75);
    assert_eq!(dual_dialogue_character_left_indent("CHARACTER", 1), 2.5);
    assert_eq!(
        dual_dialogue_character_left_indent(&"X".repeat(26), 1),
        1.6875
    );
    assert_eq!(
        dual_dialogue_character_left_indent(&"X".repeat(29), 1),
        1.5625
    );
    assert_eq!(dual_dialogue_character_left_indent("A", 2), 6.0);
    assert_eq!(dual_dialogue_character_left_indent("MARK", 2), 5.875);
}

#[test]
fn layout_geometry_tracks_multicam_act_and_cold_opening_styles() {
    let mut styles = BTreeMap::new();
    styles.insert(
        "Cold Opening".into(),
        FdxParagraphStyle {
            left_indent: 1.0,
            right_indent: 7.5,
            space_before: 12.0,
            spacing: 1.0,
            alignment: Alignment::Center,
        },
    );
    styles.insert(
        "New Act".into(),
        FdxParagraphStyle {
            left_indent: 1.5,
            right_indent: 7.5,
            space_before: 0.0,
            spacing: 1.0,
            alignment: Alignment::Center,
        },
    );
    styles.insert(
        "End of Act".into(),
        FdxParagraphStyle {
            left_indent: 1.5,
            right_indent: 7.5,
            space_before: 24.0,
            spacing: 1.0,
            alignment: Alignment::Center,
        },
    );
    let settings = FdxExtractedSettings {
        paragraph_styles: styles,
    };

    let geometry = LayoutGeometry::from_fdx_settings(&settings);

    assert_eq!(geometry.cold_opening_left, 1.0);
    assert_eq!(geometry.cold_opening_right, 7.5);
    assert_eq!(geometry.cold_opening_spacing_before, 1.0);
    assert_eq!(geometry.cold_opening_alignment, Alignment::Center);

    assert_eq!(geometry.new_act_left, 1.5);
    assert_eq!(geometry.new_act_right, 7.5);
    assert_eq!(geometry.new_act_spacing_before, 0.0);
    assert_eq!(geometry.new_act_alignment, Alignment::Center);

    assert_eq!(geometry.end_of_act_left, 1.5);
    assert_eq!(geometry.end_of_act_right, 7.5);
    assert_eq!(geometry.end_of_act_spacing_before, 2.0);
    assert_eq!(geometry.end_of_act_alignment, Alignment::Center);
}
