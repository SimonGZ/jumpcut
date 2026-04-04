// tests/pagination_wrapping_test.rs
//
// These TDD tests explore the ideal API for the new Final Draft-parity greedy wrapping
// algorithm. They verify the constraints laid out in the pagination-spec.md.

// We hypothesize an ideal API that takes raw text, an exact width limit, and applies a configuration
// for space-preservation specific to screenplay pagination needs.
use std::fs;

use jumpcut::pagination::wrapping::{
    wrap_styled_text_for_element, wrap_text_for_element, wrap_text_for_element_with_offsets,
    ElementType, InterruptionDashWrap, WrapConfig, WrappedStyledFragment,
};
use jumpcut::pagination::{FdxExtractedSettings, LayoutGeometry};
use jumpcut::styled_text::{StyledRun, StyledText};

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
    assert_eq!(
        lines[0],
        "Jo sits, hands folded, trying to cover the ink stains. Mr."
    );
    assert_eq!(
        lines[1],
        "Dashwood reads her story with a pen in hand, gleefully"
    );
    assert_eq!(
        lines[2],
        "crossing out and making notes, changes. Every time his pen"
    );
    assert_eq!(
        lines[3],
        "scratches, Jo feels her heart breaking. She’s on the verge of"
    );
    assert_eq!(lines[4], "tears when:");
}

#[test]
fn final_draft_discounts_exactly_one_trailing_dash_from_word_width() {
    let config = WrapConfig {
        exact_width_chars: 10,
        interruption_dash_wrap: InterruptionDashWrap::KeepTogether,
    };

    // SCENARIO 1: "A 12345678-" (length 11)
    // "A " is 2 visual characters conceptually (or 2 length on line).
    // Word is "12345678-" (length 9).
    // Discounting EXACTLY ONE dash gives an effective word length of 8.
    // Line width = 2 + 8 = 10. This FITS exactly within the 10-char limit!
    let lines_fit = wrap_text_for_element("A 12345678-", &config);
    assert_eq!(
        lines_fit.len(),
        1,
        "The word ending in a single dash should fit when one dash is discounted"
    );
    assert_eq!(lines_fit[0], "A 12345678-");

    // SCENARIO 2: "A 12345678--" (length 12)
    // "A " is 2 limit.
    // Word is "12345678--" (length 10).
    // Discounting EXACTLY ONE dash gives an effective word length of 9.
    // Line width = 2 + 9 = 11. This EXCEEDS the 10-char limit!
    // In keep-together mode, this should still wrap because only one dash is discounted.
    // Final Draft mode has a separate policy for splitting some trailing `--` endings.
    let lines_wrap = wrap_text_for_element("A 12345678--", &config);
    assert_eq!(
        lines_wrap.len(),
        2,
        "The word ending in two dashes should wrap because only one dash is discounted"
    );
    assert_eq!(lines_wrap[0], "A");
    assert_eq!(lines_wrap[1], "12345678--");
}

#[test]
fn final_draft_mode_can_split_a_standalone_double_dash_across_lines() {
    let config = WrapConfig {
        exact_width_chars: 35,
        interruption_dash_wrap: InterruptionDashWrap::FinalDraft,
    };

    let text = "everything you said was impossible -- everything! -- I felt like such a fool";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines,
        vec![
            "everything you said was impossible -",
            "- everything! -- I felt like such a",
            "fool",
        ]
    );
}

#[test]
fn clean_mode_keeps_a_standalone_double_dash_together() {
    let config = WrapConfig {
        exact_width_chars: 35,
        interruption_dash_wrap: InterruptionDashWrap::KeepTogether,
    };

    let text = "everything you said was impossible -- everything! -- I felt like such a fool";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines,
        vec![
            "everything you said was impossible",
            "-- everything! -- I felt like such",
            "a fool",
        ]
    );
}

#[test]
fn final_draft_mode_can_split_a_word_ending_in_double_hyphens() {
    let config = WrapConfig {
        exact_width_chars: 38,
        interruption_dash_wrap: InterruptionDashWrap::FinalDraft,
    };

    let text = "Qaxu, uda owy’ox eqyxu. Y ugyvoju’e ga--";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines,
        vec!["Qaxu, uda owy’ox eqyxu. Y ugyvoju’e ga-", "-"]
    );
}

#[test]
fn clean_mode_keeps_a_word_ending_in_double_hyphens_together() {
    let config = WrapConfig {
        exact_width_chars: 38,
        interruption_dash_wrap: InterruptionDashWrap::KeepTogether,
    };

    let text = "Qaxu, uda owy’ox eqyxu. Y ugyvoju’e ga--";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(lines, vec!["Qaxu, uda owy’ox eqyxu. Y ugyvoju’e", "ga--"]);
}

#[test]
fn final_draft_discounts_all_trailing_spaces_from_width() {
    let config = WrapConfig::with_exact_width_chars(10);

    // "1234567890" is exactly 10 characters.
    // Adding 8 spaces makes it 18 characters: "1234567890        "
    // Since Final Draft discounts ALL trailing spaces, this should evaluate as 10 characters,
    // and thus FIT on the single 10-character line perfectly.
    let text = "1234567890        ";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines.len(),
        1,
        "A 10-character word followed by 8 spaces should not wrap on a 10-char limit!"
    );
    assert_eq!(lines[0], "1234567890");

    // But if there is a visible character AFTER the spaces, it must wrap!
    // "1234567890        A"
    // The spaces are no longer trailing the line, they are internal to the wrap context!
    // (It wraps after the 10th char)
    let text2 = "1234567890        A";
    let lines2 = wrap_text_for_element(text2, &config);
    assert_eq!(
        lines2.len(),
        2,
        "If a visible character follows the spaces, it must wrap."
    );
}

#[test]
fn final_draft_allows_hyphenated_compounds_to_break_after_a_trailing_hyphen() {
    let config = WrapConfig::new(ElementType::Dialogue);

    let text = "Did I want to deprive my soon-to-be-born son the chance to catch a fish like this of his own?  This lady fish and I, well, we had the same destiny.";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines,
        vec![
            "Did I want to deprive my soon-to-be-",
            "born son the chance to catch a fish",
            "like this of his own?  This lady",
            "fish and I, well, we had the same",
            "destiny.",
        ]
    );
}

#[test]
fn mostly_genius_action_wrap_allows_the_final_draft_hyphen_split_for_el_00363() {
    let settings: FdxExtractedSettings = serde_json::from_str(
        &fs::read_to_string(
            "tests/fixtures/corpus/public/mostly-genius/extracted/fdx-settings.json",
        )
        .unwrap(),
    )
    .unwrap();
    let geometry = LayoutGeometry::from_fdx_settings(&settings);
    let config = WrapConfig::from_geometry(&geometry, ElementType::Action);

    let text = "EWYKO GYJYG WYSOHA, AWUBY. RUS ROGO OV KUQYPAXYDA QAGAGO--YZOVAPYPY REVOQU EQOQ, WUDAWEPAW AKUR HOG GERAB, UKEROJA OSOPOWU YVY. YSUREZU YPUJEJ.";
    let lines = wrap_text_for_element(text, &config);

    assert_eq!(
        lines,
        vec![
            "EWYKO GYJYG WYSOHA, AWUBY. RUS ROGO OV KUQYPAXYDA QAGAGO--",
            "YZOVAPYPY REVOQU EQOQ, WUDAWEPAW AKUR HOG GERAB, UKEROJA",
            "OSOPOWU YVY. YSUREZU YPUJEJ.",
        ]
    );
}

#[test]
fn wrap_config_can_be_created_from_custom_geometry() {
    use jumpcut::pagination::LayoutGeometry;

    let mut geometry = LayoutGeometry::default();
    // Default dialogue is 2.5 to 6.0 (3.5 inches = 35 chars)
    // Let's make it narrower: 2.5 to 5.0 (2.5 inches = 25 chars)
    geometry.dialogue_right = 5.0;

    // This constructor doesn't exist yet
    let config = WrapConfig::from_geometry(&geometry, ElementType::Dialogue);

    assert_eq!(config.exact_width_chars, 25);

    let text = "1234567890123456789012345 6"; // Space at 26th char. "1234567890123456789012345 " is 26 chars, trimmed is 25.
    let lines = wrap_text_for_element(text, &config);
    assert_eq!(
        lines.len(),
        2,
        "Should wrap at the space after 25 characters"
    );
}

#[test]
fn wrapped_lines_report_start_and_end_offsets() {
    let config = WrapConfig::with_exact_width_chars(10);
    let text = "One two three";

    let lines = wrap_text_for_element_with_offsets(text, &config);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, "One two");
    assert_eq!(lines[0].start_offset, 0);
    assert_eq!(lines[0].end_offset, 8);
    assert_eq!(lines[1].text, "three");
    assert_eq!(lines[1].start_offset, 8);
    assert_eq!(lines[1].end_offset, text.len());
}

#[test]
fn wrapped_line_offsets_follow_hyphen_and_space_chunk_boundaries() {
    let config = WrapConfig::new(ElementType::Dialogue);
    let text = "Did I want to deprive my soon-to-be-born son";

    let lines = wrap_text_for_element_with_offsets(text, &config);

    assert_eq!(lines[0].text, "Did I want to deprive my soon-to-be-");
    assert_eq!(lines[0].start_offset, 0);
    assert_eq!(lines[0].end_offset, 36);
    assert_eq!(lines[1].text, "born son");
    assert_eq!(lines[1].start_offset, 36);
    assert_eq!(lines[1].end_offset, text.len());
}

#[test]
fn styled_wrapping_preserves_multiple_style_fragments_on_one_line() {
    let config = WrapConfig::with_exact_width_chars(20);
    let text = StyledText {
        plain_text: "BOLD WORDS".into(),
        runs: vec![
            StyledRun {
                text: "BOLD ".into(),
                styles: vec!["Bold".into()],
            },
            StyledRun {
                text: "WORDS".into(),
                styles: vec!["Italic".into()],
            },
        ],
    };

    let lines = wrap_styled_text_for_element(&text, &config);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, "BOLD WORDS");
    assert_eq!(
        lines[0].fragments,
        vec![
            WrappedStyledFragment {
                text: "BOLD ".into(),
                styles: vec!["Bold".into()],
            },
            WrappedStyledFragment {
                text: "WORDS".into(),
                styles: vec!["Italic".into()],
            },
        ]
    );
}

#[test]
fn styled_wrapping_slices_a_single_styled_run_across_wrapped_lines() {
    let config = WrapConfig::with_exact_width_chars(10);
    let text = StyledText {
        plain_text: "Bold words here".into(),
        runs: vec![StyledRun {
            text: "Bold words here".into(),
            styles: vec!["Bold".into()],
        }],
    };

    let lines = wrap_styled_text_for_element(&text, &config);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, "Bold words");
    assert_eq!(lines[0].start_offset, 0);
    assert_eq!(lines[0].end_offset, 11);
    assert_eq!(
        lines[0].fragments,
        vec![WrappedStyledFragment {
            text: "Bold words ".into(),
            styles: vec!["Bold".into()],
        }]
    );

    assert_eq!(lines[1].text, "here");
    assert_eq!(lines[1].start_offset, 11);
    assert_eq!(lines[1].end_offset, text.plain_text.len());
    assert_eq!(
        lines[1].fragments,
        vec![WrappedStyledFragment {
            text: "here".into(),
            styles: vec!["Bold".into()],
        }]
    );
}
