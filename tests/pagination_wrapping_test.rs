// tests/pagination_wrapping_test.rs
//
// These TDD tests explore the ideal API for the new Final Draft-parity greedy wrapping
// algorithm. They verify the constraints laid out in the pagination-spec.md.

// We hypothesize an ideal API that takes raw text, an exact width limit, and applies a configuration
// for space-preservation specific to screenplay pagination needs.
use jumpcut::pagination::wrapping::{wrap_text_for_element, ElementType, WrapConfig};

#[test]
fn action_width_fits_exactly_61_characters() {
    // 6-inch action width typically implies 60 characters (10 CPI),
    // but Final Draft is known to perfectly fit 61 characters cleanly.
    let config = WrapConfig::new(ElementType::Action);

    // Exactly 61 characters
    let text = "ABCDE FGHIJ KLMNO PQRST UVWXY ZABCD EFGHI JKLMN OPQRS TUVWXYZ";
    assert_eq!(text.len(), 61);

    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines.len(),
        1,
        "Expected 61-character string to not trigger an overflow wrap on Action block"
    );
    assert_eq!(lines[0], text);
}

#[test]
fn wrapping_respects_and_preserves_internal_spaces() {
    let config = WrapConfig::with_exact_width_chars(15);

    // "One sentence.  " exactly hits 15 characters, but the next text should push over.
    // The two spaces should count against the width limit but ideally not be completely stripped
    // if rendering depends on them, OR they simply push the next word strictly to the next line.
    let text = "One sentence.  Two sentence.";

    let lines = wrap_text_for_element(text, &config);
    assert_eq!(lines.len(), 2);
    // The exact trailing space behavior on visually split lines might vary,
    // but the critical wrap must occur.
    assert_eq!(lines[0].trim_end(), "One sentence.");
    assert_eq!(lines[1], "Two sentence.");
}

#[test]
fn trailing_spaces_at_the_end_of_a_visual_line_do_not_trigger_unnecessary_wraps() {
    let config = WrapConfig::with_exact_width_chars(10);

    // "1234567890" is 10 chars. Adding a space brings it to 11.
    // Final Draft does NOT push that extra space to a new blank line.
    let text = "1234567890 ";

    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines.len(),
        1,
        "Trailing space should be absorbed unconditionally"
    );
    assert_eq!(lines[0].trim_end(), "1234567890");
}

#[test]
fn multi_line_action_paragraph_splits_optimally() {
    let config = WrapConfig::new(ElementType::Action); // Assumed 61 char limit

    // 123 characters total.
    let text = "Edward Bloom, 61, lies asleep on the bed. Although he's not the vibrant man we've seen before, it's not as bad they feared.";

    let lines = wrap_text_for_element(text, &config);

    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0],
        "Edward Bloom, 61, lies asleep on the bed. Although he's not"
    );
    assert_eq!(
        lines[1],
        "the vibrant man we've seen before, it's not as bad they"
    );
    assert_eq!(lines[2], "feared.");
}

#[test]
fn action_text_wraps_jo_sits_scenario_correctly() {
    let config = WrapConfig::new(ElementType::Action);

    let text = "Jo sits, hands folded, trying to cover the ink stains. Mr. Dashwood reads her story with a pen in hand, gleefully crossing out and making notes, changes. Every time his pen scratches, Jo feels her heart breaking. She’s on the verge of tears when:";

    let lines = wrap_text_for_element(text, &config);

    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0], "Jo sits, hands folded, trying to cover the ink stains. Mr.");
    assert_eq!(lines[1], "Dashwood reads her story with a pen in hand, gleefully");
    assert_eq!(lines[2], "crossing out and making notes, changes. Every time his pen");
    assert_eq!(lines[3], "scratches, Jo feels her heart breaking. She’s on the verge of");
    assert_eq!(lines[4], "tears when:");
}
