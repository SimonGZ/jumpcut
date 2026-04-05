use std::collections::HashMap;

use jumpcut::pagination::{
    Alignment, InterruptionDashWrap, PaginationConfig, ScreenplayLayoutProfile, StyleProfile,
};
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
    assert_eq!(geometry.dual_dialogue_left_character_left, 2.5);
    assert_eq!(geometry.dual_dialogue_left_character_right, 4.875);
    assert_eq!(geometry.dual_dialogue_left_left, 1.5);
    assert_eq!(geometry.dual_dialogue_left_right, 4.375);
    assert_eq!(geometry.dual_dialogue_left_parenthetical_left, 1.75);
    assert_eq!(geometry.dual_dialogue_left_parenthetical_right, 4.125);
    assert_eq!(geometry.dual_dialogue_right_character_left, 5.875);
    assert_eq!(geometry.dual_dialogue_right_character_right, 7.5);
    assert_eq!(geometry.dual_dialogue_right_left, 4.625);
    assert_eq!(geometry.dual_dialogue_right_right, 7.5);
    assert_eq!(geometry.dual_dialogue_right_parenthetical_left, 4.875);
    assert_eq!(geometry.dual_dialogue_right_parenthetical_right, 7.25);
    assert_eq!(geometry.character_right, 6.25);
    assert_eq!(geometry.parenthetical_left, 2.75);
    assert_eq!(geometry.transition_right, 7.25);
    assert_eq!(geometry.scene_heading_spacing_before, 1.0);
    assert_eq!(geometry.new_act_alignment, Alignment::Center);
}

#[test]
fn pagination_config_can_be_built_from_screenplay_metadata_profile() {
    let screenplay = parse("Title: Demo\nFmt: multicam dr-5.75\n\nNEW ACT ONE\n\nINT. SET - DAY\n");

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
fn explicit_fmt_geometry_knobs_override_multicam_even_if_they_appear_first() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["dr-5.75 dl-3.0 ssbsh multicam".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);
    let geometry = profile.to_pagination_geometry();

    assert_eq!(profile.style_profile, StyleProfile::Multicam);
    assert_eq!(geometry.dialogue_left, 3.0);
    assert_eq!(geometry.dialogue_right, 5.75);
    assert_eq!(geometry.scene_heading_spacing_before, 1.0);
    assert_eq!(geometry.character_right, 6.25);
    assert_eq!(geometry.parenthetical_left, 2.75);
    assert_eq!(geometry.transition_right, 7.25);
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

#[test]
fn no_auto_act_breaks_fmt_disables_new_act_page_starts_without_changing_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["no-auto-act-breaks".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);
    let geometry = profile.to_pagination_geometry();

    assert!(!profile.styles.new_act.starts_new_page);
    assert_eq!(geometry.dialogue_left, 2.5);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.scene_heading_spacing_before, 2.0);
    assert_eq!(geometry.line_height, 1.0);
}

#[test]
fn no_act_underlines_fmt_disables_default_act_underlines_without_changing_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["no-act-underlines".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);
    let geometry = profile.to_pagination_geometry();

    assert!(!profile.styles.cold_opening.underline);
    assert!(!profile.styles.new_act.underline);
    assert!(!profile.styles.end_of_act.underline);
    assert_eq!(geometry.dialogue_left, 2.5);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.scene_heading_spacing_before, 2.0);
    assert_eq!(geometry.line_height, 1.0);
}

#[test]
fn clean_dashes_fmt_switches_the_wrap_policy_without_changing_geometry() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["clean-dashes".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);
    let geometry = profile.to_pagination_geometry();

    assert_eq!(profile.interruption_dash_wrap, InterruptionDashWrap::KeepTogether);
    assert_eq!(geometry.dialogue_left, 2.5);
    assert_eq!(geometry.dialogue_right, 6.0);
    assert_eq!(geometry.scene_heading_spacing_before, 2.0);
}

#[test]
fn balanced_fmt_switches_dash_behavior_and_disables_dual_dialogue_contds() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["balanced".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);

    assert_eq!(profile.interruption_dash_wrap, InterruptionDashWrap::KeepTogether);
    assert!(!profile.dual_dialogue_counts_for_contd);
    assert!(profile.closer_dual_dialogue_cues);
}

#[test]
fn no_dual_contds_fmt_only_disables_the_dual_dialogue_contd_rule() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["no-dual-contds".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);

    assert_eq!(profile.interruption_dash_wrap, InterruptionDashWrap::FinalDraft);
    assert!(!profile.dual_dialogue_counts_for_contd);
    assert!(!profile.closer_dual_dialogue_cues);
}

#[test]
fn closer_dual_dialogue_cues_fmt_only_enables_the_cleaner_dual_cue_spacing_rule() {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("fmt".into(), vec!["closer-dual-dialogue-cues".into()]);

    let profile = ScreenplayLayoutProfile::from_metadata(&metadata);

    assert_eq!(profile.interruption_dash_wrap, InterruptionDashWrap::FinalDraft);
    assert!(profile.dual_dialogue_counts_for_contd);
    assert!(profile.closer_dual_dialogue_cues);
}
