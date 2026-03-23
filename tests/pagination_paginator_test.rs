use jumpcut::pagination::{
    BlockPlacement, Cohesion, ContinuationMarker, DialoguePart, DialoguePartKind, DialogueUnit,
    DualDialogueSide, DualDialogueUnit, FlowKind, FlowUnit, Fragment, LyricUnit, PageKind,
    PageStartUnit, PaginatedScreenplay, PaginationConfig, PaginationScope, SemanticScreenplay,
    SemanticUnit,
};
use pretty_assertions::assert_eq;

#[test]
fn it_places_whole_flow_lyric_and_dialogue_units_onto_pages() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.")),
            SemanticUnit::Lyric(lyric_unit("el-00002", "Sing!")),
            SemanticUnit::Dialogue(dialogue_unit(
                "block-00001",
                vec![
                    dialogue_part("el-00003", DialoguePartKind::Character, "MARCUS"),
                    dialogue_part("el-00004", DialoguePartKind::Dialogue, "Three."),
                ],
            )),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 3 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
    assert_eq!(actual.pages[0].metadata.number, 2);
    assert_eq!(actual.pages[0].metadata.kind, PageKind::Body);
    assert_eq!(
        actual.pages[0]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00001", "el-00002"]
    );
    assert_eq!(
        actual.pages[1]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00003", "el-00004"]
    );
    assert_eq!(actual.pages[1].blocks[0].id, "block-00001");
}

#[test]
fn it_honors_explicit_page_start_units() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.")),
            SemanticUnit::PageStart(PageStartUnit {
                source_element_id: "el-00002".into(),
            }),
            SemanticUnit::Flow(flow_unit("el-00002", FlowKind::Action, "Two.")),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 10 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
    assert_eq!(actual.pages[0].metadata.number, 2);
    assert_eq!(actual.pages[1].metadata.number, 3);
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

#[test]
fn it_keeps_scene_headings_with_the_following_unit_when_possible() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.\nThree.")),
            SemanticUnit::Flow(scene_heading_unit("el-00002", "INT. OFFICE - DAY")),
            SemanticUnit::Flow(flow_unit("el-00003", FlowKind::Action, "Four.")),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 4 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
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
        vec!["el-00002", "el-00003"]
    );
}

#[test]
fn it_places_dual_dialogue_units_whole_and_preserves_dual_blocks() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.")),
            SemanticUnit::DualDialogue(DualDialogueUnit {
                group_id: "dual-00001".into(),
                sides: vec![
                    DualDialogueSide {
                        side: 1,
                        dialogue: dialogue_unit(
                            "block-00001",
                            vec![
                                dialogue_part("el-00002", DialoguePartKind::Character, "LEFT"),
                                dialogue_part("el-00003", DialoguePartKind::Dialogue, "Alpha."),
                            ],
                        ),
                    },
                    DualDialogueSide {
                        side: 2,
                        dialogue: dialogue_unit(
                            "block-00002",
                            vec![
                                dialogue_part("el-00004", DialoguePartKind::Character, "RIGHT"),
                                dialogue_part("el-00005", DialoguePartKind::Dialogue, "Beta."),
                            ],
                        ),
                    },
                ],
                cohesion: splittable_cohesion(),
            }),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 3 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
    assert_eq!(
        actual.pages[1]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00002", "el-00003", "el-00004", "el-00005"]
    );
    let dual_blocks: Vec<_> = actual.pages[1]
        .blocks
        .iter()
        .filter(|block| matches!(block.placement, BlockPlacement::DualDialogue { .. }))
        .collect();
    assert_eq!(dual_blocks.len(), 2);
}

#[test]
fn it_splits_dialogue_units_at_part_boundaries_and_marks_continuations() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.")),
            SemanticUnit::Dialogue(dialogue_unit(
                "block-00001",
                vec![
                    dialogue_part("el-00002", DialoguePartKind::Character, "MARCUS"),
                    dialogue_part("el-00003", DialoguePartKind::Dialogue, "First bit."),
                    dialogue_part("el-00004", DialoguePartKind::Parenthetical, "(leaning in)"),
                    dialogue_part("el-00005", DialoguePartKind::Dialogue, "Second bit."),
                ],
            )),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 4 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
    assert_eq!(
        actual.pages[0]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00001", "el-00002", "el-00003"]
    );
    assert_eq!(
        actual.pages[1]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00004", "el-00005"]
    );

    let page_one_dialogue = actual.pages[0]
        .items
        .iter()
        .find(|item| item.element_id == "el-00003")
        .unwrap();
    assert_eq!(page_one_dialogue.fragment, Fragment::ContinuedToNext);
    assert_eq!(
        page_one_dialogue.continuation_markers,
        vec![ContinuationMarker::More]
    );

    let page_two_parenthetical = actual.pages[1]
        .items
        .iter()
        .find(|item| item.element_id == "el-00004")
        .unwrap();
    assert_eq!(page_two_parenthetical.fragment, Fragment::ContinuedFromPrev);
    assert_eq!(
        page_two_parenthetical.continuation_markers,
        vec![ContinuationMarker::Continued]
    );

    let page_one_block = actual.pages[0]
        .blocks
        .iter()
        .find(|block| block.id == "block-00001")
        .unwrap();
    assert_eq!(page_one_block.fragment, Fragment::ContinuedToNext);

    let page_two_block = actual.pages[1]
        .blocks
        .iter()
        .find(|block| block.id == "block-00001")
        .unwrap();
    assert_eq!(page_two_block.fragment, Fragment::ContinuedFromPrev);
}

#[test]
fn it_does_not_orphan_a_character_cue_when_dialogue_wont_fit() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.\nThree.")),
            SemanticUnit::Dialogue(dialogue_unit(
                "block-00001",
                vec![
                    dialogue_part("el-00002", DialoguePartKind::Character, "MARCUS"),
                    dialogue_part("el-00003", DialoguePartKind::Dialogue, "First bit."),
                ],
            )),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 4 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
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
        vec!["el-00002", "el-00003"]
    );
}

#[test]
fn it_keeps_parentheticals_with_some_dialogue_when_splitting() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![
            SemanticUnit::Flow(flow_unit("el-00001", FlowKind::Action, "One.\nTwo.\nThree.")),
            SemanticUnit::Dialogue(dialogue_unit(
                "block-00001",
                vec![
                    dialogue_part("el-00002", DialoguePartKind::Character, "MARCUS"),
                    dialogue_part("el-00003", DialoguePartKind::Parenthetical, "(quietly)"),
                    dialogue_part("el-00004", DialoguePartKind::Dialogue, "First bit."),
                ],
            )),
        ],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 5 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);
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
        vec!["el-00002", "el-00003", "el-00004"]
    );
}

#[test]
fn it_splits_flow_units_at_explicit_line_boundaries() {
    let semantic = SemanticScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        units: vec![SemanticUnit::Flow(flow_unit(
            "el-00001",
            FlowKind::Action,
            "One.\nTwo.\nThree.\nFour.",
        ))],
    };

    let actual = PaginatedScreenplay::paginate(
        semantic,
        PaginationConfig { lines_per_page: 2 },
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.pages.len(), 2);

    let first = &actual.pages[0].items[0];
    assert_eq!(first.element_id, "el-00001");
    assert_eq!(first.fragment, Fragment::ContinuedToNext);
    assert_eq!(first.line_range, Some((1, 2)));
    assert_eq!(first.continuation_markers, vec![ContinuationMarker::More]);

    let second = &actual.pages[1].items[0];
    assert_eq!(second.element_id, "el-00001");
    assert_eq!(second.fragment, Fragment::ContinuedFromPrev);
    assert_eq!(second.line_range, Some((3, 4)));
    assert_eq!(
        second.continuation_markers,
        vec![ContinuationMarker::Continued]
    );
}

fn flow_unit(element_id: &str, kind: FlowKind, text: &str) -> FlowUnit {
    let cohesion = match &kind {
        FlowKind::SceneHeading => Cohesion {
            keep_together: true,
            keep_with_next: true,
            can_split: false,
        },
        _ => Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    };

    FlowUnit {
        element_id: element_id.into(),
        kind,
        text: text.into(),
        line_range: None,
        scene_number: None,
        cohesion,
    }
}

fn scene_heading_unit(element_id: &str, text: &str) -> FlowUnit {
    flow_unit(element_id, FlowKind::SceneHeading, text)
}

fn lyric_unit(element_id: &str, text: &str) -> LyricUnit {
    LyricUnit {
        element_id: element_id.into(),
        text: text.into(),
        cohesion: splittable_cohesion(),
    }
}

fn dialogue_unit(block_id: &str, parts: Vec<DialoguePart>) -> DialogueUnit {
    DialogueUnit {
        block_id: block_id.into(),
        parts,
        cohesion: splittable_cohesion(),
    }
}

fn dialogue_part(element_id: &str, kind: DialoguePartKind, text: &str) -> DialoguePart {
    DialoguePart {
        element_id: element_id.into(),
        kind,
        text: text.into(),
    }
}

fn splittable_cohesion() -> Cohesion {
    Cohesion {
        keep_together: false,
        keep_with_next: false,
        can_split: true,
    }
}
