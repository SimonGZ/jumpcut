// tests/pagination_margin_test.rs
use jumpcut::pagination::margin::calculate_element_width;
use jumpcut::pagination::LayoutGeometry;
use jumpcut::pagination::wrapping::ElementType;

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
