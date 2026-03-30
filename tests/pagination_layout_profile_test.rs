use std::collections::HashMap;

use jumpcut::pagination::{Alignment, PaginationConfig, ScreenplayLayoutProfile, StyleProfile};
use jumpcut::parse;
use jumpcut::Metadata;

#[test]
fn multicam_fmt_produces_a_shared_layout_profile_from_parser_metadata() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["multicam dr-5.75".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);

    assert_eq!(profile.style_profile, StyleProfile::Multicam);

    assert_eq!(profile.styles.dialogue.left_indent, 2.25);
    assert_eq!(profile.styles.dialogue.right_indent, 5.75);
    assert_eq!(profile.styles.dialogue.line_spacing, 2.0);
    assert_eq!(profile.styles.dialogue.alignment, Alignment::Left);

    assert_eq!(profile.styles.character.right_indent, 6.25);
    assert_eq!(profile.styles.parenthetical.left_indent, 2.75);
    assert_eq!(profile.styles.transition.right_indent, 7.25);

    assert_eq!(profile.styles.new_act.alignment, Alignment::Center);
    assert!(profile.styles.new_act.starts_new_page);
}

#[test]
fn shared_layout_profile_can_lower_into_pagination_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["multicam ssbsh dr-5.75".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);
    let geometry = profile.to_pagination_geometry();
    assert_eq!(geometry.dialogue_left, 2.25);
    assert_eq!(geometry.dialogue_right, 5.75);
    assert_eq!(geometry.character_right, 6.25);
    assert_eq!(geometry.parenthetical_left, 2.75);
    assert_eq!(geometry.transition_right, 7.25);
    assert_eq!(geometry.scene_heading_spacing_before, 1.0);
    assert_eq!(geometry.new_act_alignment, Alignment::Center);
}

#[test]
fn pagination_config_can_be_built_from_screenplay_metadata_profile() {
    let screenplay = parse(
        "Title: Demo\nFmt: multicam dr-5.75\n\nNEW ACT ONE\n\nINT. SET - DAY\n",
    );

    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);

    assert_eq!(config.lines_per_page, 54.0);
    assert_eq!(config.geometry.dialogue_left, 2.25);
    assert_eq!(config.geometry.dialogue_right, 5.75);
    assert_eq!(config.geometry.character_right, 6.25);
    assert_eq!(config.geometry.parenthetical_left, 2.75);
    assert_eq!(config.geometry.transition_right, 7.25);
}

#[test]
fn geometry_affecting_fmt_options_all_map_into_layout_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert(
        "fmt".into(),
        vec!["multicam ssbsh dsd dl-2.25 dr-6.00".into()],
    );

    let geometry = ScreenplayLayoutProfile::from_metadata(&metadata).to_pagination_geometry();

    assert_eq!(geometry.dialogue_left, 2.25);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.character_right, 6.25);
    assert_eq!(geometry.parenthetical_left, 2.75);
    assert_eq!(geometry.transition_right, 7.25);
    assert_eq!(geometry.scene_heading_spacing_before, 1.0);
    assert_eq!(geometry.line_height, 2.0);
}

#[test]
fn render_only_fmt_options_do_not_change_pagination_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["bsh ush acat cfd".into()]);

    let geometry = ScreenplayLayoutProfile::from_metadata(&metadata).to_pagination_geometry();

    assert_eq!(geometry.dialogue_left, 2.5);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.scene_heading_spacing_before, 2.0);
    assert_eq!(geometry.line_height, 1.0);
}
