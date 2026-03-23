use jumpcut::pagination::{
    measure_dialogue_part_lines, measure_dialogue_unit_lines, measure_flow_unit_lines,
    measure_text_lines, Cohesion, DialoguePart, DialoguePartKind, DialogueUnit, FlowKind,
    FlowUnit, MeasurementConfig, PageKind, PaginatedScreenplay, PaginationConfig,
    PaginationScope, SemanticScreenplay, SemanticUnit,
};
use pretty_assertions::assert_eq;

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

    assert_eq!(measure_flow_unit_lines(&unit, &measurement), 2);
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
}

#[test]
fn paginator_uses_width_aware_measurement_for_page_placement() {
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
            lines_per_page: 2,
            measurement: narrow_measurement(),
        },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
    assert_eq!(actual.pages[0].metadata.kind, PageKind::Body);
    assert_eq!(
        actual.pages[0]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00001"]
    );
    assert_eq!(
        actual.pages[1]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00002"]
    );
}

fn narrow_measurement() -> MeasurementConfig {
    MeasurementConfig {
        chars_per_inch: 1.0,
        lines_per_inch: 6.0,
        action_left_indent_in: 0.0,
        action_right_indent_in: 10.0,
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
    }
}

fn splittable_cohesion() -> Cohesion {
    Cohesion {
        keep_together: false,
        keep_with_next: false,
        can_split: true,
    }
}
