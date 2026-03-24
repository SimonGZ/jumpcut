use jumpcut::pagination::{
    boundary_spacing_lines, measure_dialogue_part_lines, measure_dialogue_unit,
    measure_dialogue_unit_lines, measure_flow_unit, measure_flow_unit_lines, measure_text_lines,
    wrap_text_lines_with_policy, Cohesion, DialoguePart, DialoguePartKind, DialogueUnit,
    FlowKind, FlowUnit, FdxExtractedSettings, MeasurementConfig, PageKind,
    PaginatedScreenplay, PaginationConfig, PaginationScope, SemanticScreenplay, SemanticUnit,
    UnitMeasurement,
};
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;

#[test]
fn it_wraps_flow_units_to_the_configured_action_width() {
    let measurement = narrow_measurement();
    let unit = FlowUnit {
        element_id: "el-00001".into(),
        kind: FlowKind::Action,
        text: "ALPHA BETA GAMMA".into(),
        line_range: None,
        scene_number: None,
        cohesion: splittable_cohesion(),
    };

    let measured = measure_flow_unit(&unit, &measurement);

    assert_eq!(measure_flow_unit_lines(&unit, &measurement), 2);
    assert_eq!(measured.top_spacing_lines, 1);
    assert_eq!(measured.bottom_spacing_lines, 1);
}

#[test]
fn it_uses_distinct_widths_for_dialogue_parts() {
    let measurement = narrow_measurement();
    let unit = DialogueUnit {
        block_id: "block-00001".into(),
        parts: vec![
            DialoguePart {
                element_id: "el-00001".into(),
                kind: DialoguePartKind::Character,
                text: "MARCUS PRIME".into(),
            },
            DialoguePart {
                element_id: "el-00002".into(),
                kind: DialoguePartKind::Parenthetical,
                text: "(VERY SOFTLY)".into(),
            },
            DialoguePart {
                element_id: "el-00003".into(),
                kind: DialoguePartKind::Dialogue,
                text: "ALPHA BETA GAMMA".into(),
            },
        ],
        cohesion: splittable_cohesion(),
    };

    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Character,
            "MARCUS PRIME",
            &measurement,
        ),
        2
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Parenthetical,
            "(VERY SOFTLY)",
            &measurement,
        ),
        2
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "ALPHA BETA GAMMA",
            &measurement,
        ),
        2
    );
    assert_eq!(measure_dialogue_unit_lines(&unit, &measurement), 6);
}

#[test]
fn it_counts_explicit_line_breaks_even_when_each_line_fits() {
    assert_eq!(measure_text_lines("ALPHA BETA\nGAMMA DELTA", 40), 2);
}

#[test]
fn dialogue_wrapping_preserves_repeated_internal_spaces() {
    assert_eq!(
        wrap_text_lines_with_policy("ALPHA  BETA", 10, false),
        vec!["ALPHA BETA"]
    );
    assert_eq!(
        wrap_text_lines_with_policy("ALPHA  BETA", 10, true),
        vec!["ALPHA", "BETA"]
    );
}

#[test]
fn screenplay_default_measures_big_fish_edward_contd_example_as_seven_lines() {
    let measurement = MeasurementConfig::screenplay_default();
    let unit = DialogueUnit {
        block_id: "block-00001".into(),
        parts: vec![
            DialoguePart {
                element_id: "el-00001".into(),
                kind: DialoguePartKind::Character,
                text: "EDWARD (CONT'D)".into(),
            },
            DialoguePart {
                element_id: "el-00002".into(),
                kind: DialoguePartKind::Dialogue,
                text: "I mean, on one hand, if dying was all you thought about, it could kind of screw you up. But it could kind of help you, couldn't it?".into(),
            },
        ],
        cohesion: splittable_cohesion(),
    };

    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Character,
            "EDWARD (CONT'D)",
            &measurement,
        ),
        1
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "I mean, on one hand, if dying was all you thought about, it could kind of screw you up. But it could kind of help you, couldn't it?",
            &measurement,
        ),
        6
    );
    assert_eq!(measure_dialogue_unit_lines(&unit, &measurement), 7);
}

#[test]
fn screenplay_default_measures_exact_little_women_dialogue_examples() {
    let measurement = MeasurementConfig::screenplay_default();

    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "The country just went through a  war. People want to be amused, not  preached at. Morals don’t sell  nowadays.  ",
            &measurement,
        ),
        5
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "You can have it. Make the edits.  ",
            &measurement,
        ),
        2
    );
}

#[test]
fn screenplay_default_measures_exact_big_fish_dialogue_example() {
    let measurement = MeasurementConfig::screenplay_default();

    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "I was thinking about death and all.  About seeing how you're gonna die.",
            &measurement,
        ),
        3
    );
}

#[test]
fn little_women_and_big_fish_examples_conflict_under_one_dialogue_width() {
    let big_fish_short =
        "I was thinking about death and all.  About seeing how you're gonna die.";
    let little_women_long =
        "The country just went through a  war. People want to be amused, not  preached at. Morals don’t sell  nowadays.  ";
    let little_women_short = "You can have it. Make the edits.  ";
    let big_fish =
        "I mean, on one hand, if dying was all you thought about, it could kind of screw you up. But it could kind of help you, couldn't it?";

    assert_eq!(measure_text_lines(big_fish_short, 28), 3);
    assert_eq!(measure_text_lines(big_fish_short, 35), 2);
    assert_eq!(measure_text_lines(little_women_long, 28), 5);
    assert_eq!(measure_text_lines(little_women_long, 29), 4);
    assert_eq!(measure_text_lines(little_women_short, 31), 2);
    assert_eq!(measure_text_lines(little_women_short, 32), 1);
    assert_eq!(measure_text_lines(big_fish, 28), 6);
    assert_eq!(measure_text_lines(big_fish, 32), 5);
}

#[test]
fn screenplay_default_exposes_narrower_dialogue_columns_than_action() {
    let measurement = MeasurementConfig::screenplay_default();

    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::Action), 60);
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Character),
        20
    );
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Dialogue),
        28
    );
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Parenthetical),
        20
    );
    assert_eq!(measurement.action_top_spacing_lines, 0);
    assert_eq!(measurement.scene_heading_bottom_spacing_lines, 0);
    assert_eq!(measurement.dialogue_top_spacing_lines, 0);
}

#[test]
fn fdx_derived_geometry_uses_real_dialogue_width_for_public_corpus() {
    let measurement = public_corpus_measurement("big-fish");

    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::Action), 60);
    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::SceneHeading), 60);
    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::ColdOpening), 65);
    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::NewAct), 60);
    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::EndOfAct), 60);
    assert_eq!(measurement.width_chars_for_flow_kind(&FlowKind::Transition), 15);
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Character),
        37
    );
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Dialogue),
        35
    );
    assert_eq!(
        measurement.width_chars_for_dialogue_part(&DialoguePartKind::Parenthetical),
        25
    );
}

#[test]
fn fdx_derived_geometry_matches_concrete_public_dialogue_examples_better() {
    let measurement = public_corpus_measurement("big-fish");

    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "I was thinking about death and all.  About seeing how you're gonna die.",
            &measurement,
        ),
        2
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "The country just went through a  war. People want to be amused, not  preached at. Morals don’t sell  nowadays.  ",
            &measurement,
        ),
        4
    );
    assert_eq!(
        measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            "You can have it. Make the edits.  ",
            &measurement,
        ),
        1
    );
}

#[test]
fn fdx_derived_spacing_uses_space_before_as_top_spacing_without_double_bottoms() {
    let measurement = public_corpus_measurement("big-fish");

    assert_eq!(measurement.action_top_spacing_lines, 1);
    assert_eq!(measurement.action_bottom_spacing_lines, 0);
    assert_eq!(measurement.scene_heading_top_spacing_lines, 2);
    assert_eq!(measurement.scene_heading_bottom_spacing_lines, 0);
    assert_eq!(measurement.cold_opening_top_spacing_lines, 1);
    assert_eq!(measurement.new_act_top_spacing_lines, 0);
    assert_eq!(measurement.end_of_act_top_spacing_lines, 2);
    assert_eq!(measurement.transition_top_spacing_lines, 1);
    assert_eq!(measurement.dialogue_top_spacing_lines, 1);
    assert_eq!(measurement.dialogue_bottom_spacing_lines, 0);
}

#[test]
fn fdx_derived_action_width_still_overestimates_big_fish_el_00787() {
    let measurement = public_corpus_measurement("big-fish");
    let unit = FlowUnit {
        element_id: "el-00787".into(),
        kind: FlowKind::Action,
        text: "Edward tosses the sign and forges ahead, into the spiderwebs.".into(),
        line_range: None,
        scene_number: None,
        cohesion: splittable_cohesion(),
    };

    assert_eq!(measure_flow_unit_lines(&unit, &measurement), 2);
}

#[test]
fn it_uses_shared_boundary_spacing_instead_of_double_counting_blank_lines() {
    let previous = UnitMeasurement {
        content_lines: 2,
        top_spacing_lines: 0,
        bottom_spacing_lines: 1,
    };
    let current = UnitMeasurement {
        content_lines: 3,
        top_spacing_lines: 1,
        bottom_spacing_lines: 0,
    };

    assert_eq!(boundary_spacing_lines(Some(&previous), Some(&current)), 1);
    assert_eq!(current.placement_lines_with_prev(Some(&previous)), 4);
}

#[test]
fn screenplay_default_adds_vertical_spacing_to_dialogue_units() {
    let measurement = MeasurementConfig::screenplay_default();
    let unit = DialogueUnit {
        block_id: "block-00001".into(),
        parts: vec![
            DialoguePart {
                element_id: "el-00001".into(),
                kind: DialoguePartKind::Character,
                text: "EDWARD (CONT'D)".into(),
            },
            DialoguePart {
                element_id: "el-00002".into(),
                kind: DialoguePartKind::Dialogue,
                text: "I mean, on one hand, if dying was all you thought about, it could kind of screw you up. But it could kind of help you, couldn't it?".into(),
            },
        ],
        cohesion: splittable_cohesion(),
    };

    let measured = measure_dialogue_unit(&unit, &measurement);

    assert_eq!(measured.content_lines, 7);
    assert_eq!(measured.top_spacing_lines, 0);
    assert_eq!(measured.bottom_spacing_lines, 0);
}

#[test]
fn paginator_uses_width_aware_measurement_and_shared_spacing_for_page_placement() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(FlowUnit {
                element_id: "el-00001".into(),
                kind: FlowKind::Action,
                text: "Short.".into(),
                line_range: None,
                scene_number: None,
                cohesion: splittable_cohesion(),
            }),
            SemanticUnit::Flow(FlowUnit {
                element_id: "el-00002".into(),
                kind: FlowKind::Action,
                text: "ALPHA BETA GAMMA".into(),
                line_range: None,
                scene_number: None,
                cohesion: splittable_cohesion(),
            }),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig {
            lines_per_page: 4,
            measurement: narrow_measurement(),
        },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 1);
    assert_eq!(actual.pages[0].metadata.kind, PageKind::Body);
    assert_eq!(
        actual.pages[0]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00001", "el-00002"]
    );
}

fn narrow_measurement() -> MeasurementConfig {
    MeasurementConfig {
        chars_per_inch: 1.0,
        lines_per_inch: 6.0,
        action_left_indent_in: 0.0,
        action_right_indent_in: 10.0,
        scene_heading_left_indent_in: 0.0,
        scene_heading_right_indent_in: 10.0,
        cold_opening_left_indent_in: 0.0,
        cold_opening_right_indent_in: 10.0,
        new_act_left_indent_in: 0.0,
        new_act_right_indent_in: 10.0,
        end_of_act_left_indent_in: 0.0,
        end_of_act_right_indent_in: 10.0,
        dialogue_left_indent_in: 0.0,
        dialogue_right_indent_in: 10.0,
        character_left_indent_in: 0.0,
        character_right_indent_in: 6.0,
        parenthetical_left_indent_in: 0.0,
        parenthetical_right_indent_in: 8.0,
        lyric_left_indent_in: 0.0,
        lyric_right_indent_in: 10.0,
        transition_left_indent_in: 0.0,
        transition_right_indent_in: 7.0,
        action_top_spacing_lines: 1,
        action_bottom_spacing_lines: 1,
        scene_heading_top_spacing_lines: 1,
        scene_heading_bottom_spacing_lines: 1,
        cold_opening_top_spacing_lines: 1,
        cold_opening_bottom_spacing_lines: 1,
        new_act_top_spacing_lines: 1,
        new_act_bottom_spacing_lines: 1,
        end_of_act_top_spacing_lines: 1,
        end_of_act_bottom_spacing_lines: 1,
        transition_top_spacing_lines: 1,
        transition_bottom_spacing_lines: 1,
        dialogue_top_spacing_lines: 1,
        dialogue_bottom_spacing_lines: 0,
        lyric_top_spacing_lines: 1,
        lyric_bottom_spacing_lines: 0,
    }
}

fn splittable_cohesion() -> Cohesion {
    Cohesion {
        keep_together: false,
        keep_with_next: false,
        can_split: true,
    }
}

fn public_corpus_measurement(screenplay: &str) -> MeasurementConfig {
    let path = Path::new("/ductor/workspace/jumpcut-layout-corpus/corpus/public")
        .join(screenplay)
        .join("extracted/fdx-settings.json");
    let settings: FdxExtractedSettings =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    MeasurementConfig::from_fdx_settings(&settings)
}
