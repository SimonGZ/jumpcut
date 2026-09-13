use jumpcut::pagination::ScreenplayLayoutProfile;
use jumpcut::pagination::layout_profile::{ELEMENT_MARGIN_TAGS, parse_element_margin_option};
use jumpcut::{ImportedElementKind, parse, parse_fdx};

fn edges(profile: &ScreenplayLayoutProfile, kind: ImportedElementKind) -> (f32, f32) {
    use ImportedElementKind::*;
    let style = match kind {
        Action => &profile.styles.action,
        SceneHeading => &profile.styles.scene_heading,
        Character => &profile.styles.character,
        Dialogue => &profile.styles.dialogue,
        Parenthetical => &profile.styles.parenthetical,
        Transition => &profile.styles.transition,
        Lyric => &profile.styles.lyric,
        ColdOpening => &profile.styles.cold_opening,
        NewAct => &profile.styles.new_act,
        EndOfAct => &profile.styles.end_of_act,
    };
    (style.left_indent, style.right_indent)
}

#[test]
fn all_tags_are_independent_case_insensitive_and_last_valid_value_wins() {
    let defaults = ScreenplayLayoutProfile::from_screenplay(&parse("INT. ROOM - DAY"));
    for &(kind, left, right) in ELEMENT_MARGIN_TAGS {
        let source = format!(
            "Fmt: {left}-1.75 {}-2.125 {right}-6.875 {left}-NaN\n\nINT. ROOM - DAY",
            left.to_uppercase()
        );
        let screenplay = parse(&source);
        let layout = ScreenplayLayoutProfile::from_screenplay(&screenplay);
        assert_eq!(edges(&layout, kind), (2.125, 6.875));
        for &(other, _, _) in ELEMENT_MARGIN_TAGS {
            if other != kind {
                assert_eq!(edges(&layout, other), edges(&defaults, other));
            }
        }
        let single = ScreenplayLayoutProfile::from_screenplay(&parse(&format!(
            "Fmt: {left}-2\n\nINT. ROOM - DAY"
        )));
        assert_eq!(edges(&single, kind), (2.0, edges(&defaults, kind).1));
        let restored = ScreenplayLayoutProfile::from_screenplay(&parse(&screenplay.to_fountain()));
        assert_eq!(edges(&restored, kind), (2.125, 6.875));
    }
    for tag in ["al--1", "al-NaN", "al-inf", "al-1e99", "al-", "unknown-2"] {
        assert_eq!(parse_element_margin_option(tag), None, "{tag}");
    }
}

#[test]
fn imported_edges_survive_fdx_fountain_round_trips_without_duplicate_tags() {
    let fmt = ELEMENT_MARGIN_TAGS
        .iter()
        .map(|(_, l, r)| format!("{l}-2.25 {r}-6.75"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut script = parse(&format!(
        "Fmt: industry {fmt} tm-0.75 bm-0.8 hm-0.3 fm-0.4\n\nINT. ROOM - DAY\n\nAction."
    ));
    let fdx = script.to_fdx();
    let mut imported = parse_fdx(&fdx).unwrap();
    let imported_footer = ScreenplayLayoutProfile::from_screenplay(&imported).footer_margin;
    imported
        .metadata
        .insert("fmt".into(), vec!["industry al-1 AL-3 custom-flag".into()]);
    let fountain = imported.to_fountain();
    assert!(fountain.contains("custom-flag"));
    assert!(!fountain.contains("AL-3"));
    let parsed = parse(&fountain);
    assert_eq!(parsed.to_fountain(), fountain);
    let layout = ScreenplayLayoutProfile::from_screenplay(&parsed);
    for &(kind, left, right) in ELEMENT_MARGIN_TAGS {
        assert_eq!(edges(&layout, kind), (2.25, 6.75));
        assert_eq!(
            fountain
                .split_whitespace()
                .filter(|token| *token == format!("{left}-2.25"))
                .count(),
            1
        );
        assert_eq!(
            fountain
                .split_whitespace()
                .filter(|token| *token == format!("{right}-6.75"))
                .count(),
            1
        );
    }
    assert!((layout.top_margin - 0.75).abs() < 0.001);
    assert!((layout.footer_margin - imported_footer).abs() < 0.001);
}

#[test]
fn scene_heading_wrapping_and_geometry_are_independent_of_action() {
    let script = parse("Fmt: al-2 ar-6 shl-1.25 shr-7.25\n\nINT. ROOM - DAY\n\nAction.");
    let geometry = ScreenplayLayoutProfile::from_screenplay(&script).to_pagination_geometry();
    assert_eq!(geometry.action_left, 2.0);
    assert_eq!(geometry.scene_heading_left, 1.25);
    assert_eq!(geometry.scene_heading_right, 7.25);
    use jumpcut::pagination::wrapping::{ElementType, WrapConfig};
    assert!(
        WrapConfig::from_geometry_industry(&geometry, ElementType::SceneHeading).exact_width_chars
            > WrapConfig::from_geometry_industry(&geometry, ElementType::Action).exact_width_chars
    );
}

#[test]
fn fdx_settings_preserve_independent_scene_heading_geometry() {
    use jumpcut::pagination::margin::{FdxExtractedSettings, LayoutGeometry};
    let settings: FdxExtractedSettings = serde_json::from_str(r#"{"paragraph_styles":{"Action":{"left_indent":2,"right_indent":6,"space_before":12,"spacing":1},"Scene Heading":{"left_indent":1.25,"right_indent":6.875,"first_indent":0.25,"space_before":12,"spacing":1}}}"#).unwrap();
    let geometry = LayoutGeometry::from_fdx_settings(&settings);
    assert_eq!(
        (
            geometry.scene_heading_left,
            geometry.scene_heading_right,
            geometry.scene_heading_first_indent
        ),
        (1.25, 6.875, 0.25)
    );
    assert_eq!((geometry.action_left, geometry.action_right), (2.0, 6.0));
}

#[test]
fn fractional_margins_survive_fountain_fdx_fountain_conversion() {
    let tags = ELEMENT_MARGIN_TAGS
        .iter()
        .map(|(_, l, r)| format!("{l}-2.125 {r}-6.875"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut original = parse(&format!(
        "Fmt: {tags} tm-0.8125 bm-0.9375 hm-0.375 fm-0.4375\n\nINT. ROOM - DAY\n\nAction."
    ));
    let imported = parse_fdx(&original.to_fdx()).unwrap();
    for script in [&imported, &parse(&imported.to_fountain())] {
        let profile = ScreenplayLayoutProfile::from_screenplay(script);
        for &(kind, _, _) in ELEMENT_MARGIN_TAGS {
            assert_eq!(edges(&profile, kind), (2.125, 6.875));
        }
        assert_eq!(
            (
                profile.top_margin,
                profile.bottom_margin,
                profile.header_margin,
                profile.footer_margin
            ),
            (0.8125, 0.9375, 0.375, 0.4375)
        );
    }
}

#[cfg(feature = "html")]
#[test]
fn ordinary_html_changes_when_element_margins_change() {
    for (_, left, right) in ELEMENT_MARGIN_TAGS {
        let mut original = parse(
            "INT. ROOM - DAY\n\nAction.\n\nBOB\n(quietly)\nHello.\n\nCUT TO:\n\n~Song\n\n> COLD OPENING <\n\n> ACT TWO <\n\n> END OF ACT TWO <",
        );
        let mut custom = parse(&format!(
            "Fmt: {left}-2.125 {right}-6.875\n\n{}",
            original.to_fountain()
        ));
        assert!(
            original.to_html(true) != custom.to_html(true),
            "{left}/{right} should affect HTML"
        );
    }
}
