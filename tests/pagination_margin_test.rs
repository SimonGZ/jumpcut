// tests/pagination_margin_test.rs
use jumpcut::pagination::margin::calculate_element_width;
use jumpcut::pagination::wrapping::ElementType;

#[test]
fn action_margin_calculation_adds_inclusive_character_quirk() {
    // 1.5 to 7.5 at 10 CPI is mathematically 60 (6.0 * 10). 
    // Action gets a +1 quirk to cleanly fit 61 characters.
    assert_eq!(calculate_element_width(1.5, 7.5, 10.0, ElementType::Action), 61);
}

#[test]
fn dialogue_margin_calculation_is_exact_math() {
    // 2.5 to 6.0 at 10 CPI is mathematically 35 (3.5 * 10).
    // Dialogue does not receive the +1 quirk.
    assert_eq!(calculate_element_width(2.5, 6.0, 10.0, ElementType::Dialogue), 35);
}

#[test]
fn character_margin_calculation() {
    // 3.5 to 7.25 at 10 CPI is mathematically 37.5.
    // Final Draft implicitly requires a floor() round to reach the exact 37 character fit.
    assert_eq!(calculate_element_width(3.5, 7.25, 10.0, ElementType::Character), 37);
}

#[test]
fn lyric_margin_calculation() {
    // 2.5 to 7.375 at 10 CPI is mathematically 48.75.
    // Floored, we expect 48.
    assert_eq!(calculate_element_width(2.5, 7.375, 10.0, ElementType::Lyric), 48);
}
