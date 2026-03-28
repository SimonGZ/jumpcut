// tests/pagination_margin_test.rs
use jumpcut::pagination::margin::calculate_element_width;
use jumpcut::pagination::{Alignment, FdxExtractedSettings, FdxParagraphStyle, LayoutGeometry};
use jumpcut::pagination::wrapping::ElementType;
use std::collections::BTreeMap;

#[test]
fn action_margin_calculation_adds_inclusive_character_quirk() {
    let geometry = LayoutGeometry::default();
    // 1.5 to 7.5 at 10 CPI is mathematically 60 (6.0 * 10). 
    // Action gets a +1 quirk to cleanly fit 61 characters.
    assert_eq!(calculate_element_width(&geometry, ElementType::Action), 61);
}

#[test]
fn dialogue_margin_calculation_is_exact_math() {
    let geometry = LayoutGeometry::default();
    // 2.5 to 6.0 at 10 CPI is mathematically 35 (3.5 * 10).
    // Dialogue does not receive the +1 quirk.
    assert_eq!(calculate_element_width(&geometry, ElementType::Dialogue), 35);
}

#[test]
fn character_margin_calculation() {
    let geometry = LayoutGeometry::default();
    // 3.5 to 7.25 at 10 CPI is mathematically 37.5.
    // Final Draft implicitly requires a floor() round to reach the exact 37 character fit.
    assert_eq!(calculate_element_width(&geometry, ElementType::Character), 37);
}

#[test]
fn lyric_margin_calculation() {
    let geometry = LayoutGeometry::default();
    // 2.5 to 7.375 at 10 CPI is mathematically 48.75.
    // Floored, we expect 48.
    assert_eq!(calculate_element_width(&geometry, ElementType::Lyric), 48);
}

#[test]
fn dual_dialogue_margin_calculation_uses_special_29_character_width() {
    let geometry = LayoutGeometry::default();

    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueLeft),
        29
    );
    assert_eq!(
        calculate_element_width(&geometry, ElementType::DualDialogueRight),
        29
    );
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
