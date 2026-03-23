use jumpcut::pagination::{
    build_semantic_screenplay, normalize_screenplay, Cohesion, DialoguePartKind, FlowKind,
    NormalizedElement, NormalizedScreenplay, SemanticUnit,
};
use jumpcut::parse;
use pretty_assertions::assert_eq;

#[test]
fn it_inserts_page_start_units_before_forced_new_pages() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Scene Heading", "INT. OFFICE - DAY")
                .with_starts_new_page(true),
            normalized_element("el-00002", "Action", "Papers everywhere."),
        ],
    });

    assert_eq!(semantic.units.len(), 3);

    match &semantic.units[0] {
        SemanticUnit::PageStart(unit) => assert_eq!(unit.source_element_id, "el-00001"),
        other => panic!("expected page start, got {other:?}"),
    }

    match &semantic.units[1] {
        SemanticUnit::Flow(unit) => {
            assert_eq!(unit.kind, FlowKind::SceneHeading);
            assert_eq!(
                unit.cohesion,
                Cohesion {
                    keep_together: true,
                    keep_with_next: true,
                    can_split: false,
                }
            );
        }
        other => panic!("expected flow unit, got {other:?}"),
    }
}

#[test]
fn it_groups_dialogue_and_lyric_blocks_into_dialogue_units() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Character", "SINGER")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
            normalized_element("el-00002", "Dialogue", "How does this sound?")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
            normalized_element("el-00003", "Lyric", "Loves Chocolate!")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock"),
        ],
    });

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::Dialogue(unit) => {
            assert_eq!(unit.block_id, "block-00001");
            assert_eq!(unit.parts.len(), 3);
            assert_eq!(unit.parts[0].kind, DialoguePartKind::Character);
            assert_eq!(unit.parts[1].kind, DialoguePartKind::Dialogue);
            assert_eq!(unit.parts[2].kind, DialoguePartKind::Lyric);
            assert_eq!(
                unit.cohesion,
                Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                }
            );
        }
        other => panic!("expected dialogue unit, got {other:?}"),
    }
}

#[test]
fn it_groups_dual_dialogue_into_a_single_dual_dialogue_unit() {
    let semantic = build_semantic_screenplay(NormalizedScreenplay {
        screenplay: "sample".into(),
        starting_page_number: None,
        elements: vec![
            normalized_element("el-00001", "Character", "LEFT")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 1),
            normalized_element("el-00002", "Dialogue", "First line.")
                .in_block("block-00001")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 1),
            normalized_element("el-00003", "Character", "RIGHT")
                .in_block("block-00002")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 2),
            normalized_element("el-00004", "Dialogue", "Second line.")
                .in_block("block-00002")
                .with_block_kind("DialogueBlock")
                .in_dual("dual-00001", 2),
        ],
    });

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::DualDialogue(unit) => {
            assert_eq!(unit.group_id, "dual-00001");
            assert_eq!(unit.sides.len(), 2);
            assert_eq!(unit.sides[0].side, 1);
            assert_eq!(unit.sides[0].dialogue.parts.len(), 2);
            assert_eq!(unit.sides[1].side, 2);
            assert_eq!(unit.sides[1].dialogue.parts.len(), 2);
        }
        other => panic!("expected dual dialogue unit, got {other:?}"),
    }
}

#[test]
fn it_creates_standalone_lyric_units_from_normalized_parser_output() {
    let screenplay = parse("~Willy Wonka! Willy Wonka!");
    let normalized = normalize_screenplay("lyric", &screenplay);
    let semantic = build_semantic_screenplay(normalized);

    assert_eq!(semantic.units.len(), 1);

    match &semantic.units[0] {
        SemanticUnit::Lyric(unit) => {
            assert_eq!(unit.element_id, "el-00001");
            assert_eq!(unit.text, "Willy Wonka! Willy Wonka!");
            assert_eq!(unit.cohesion.can_split, true);
        }
        other => panic!("expected lyric unit, got {other:?}"),
    }
}

fn normalized_element(element_id: &str, kind: &str, text: &str) -> NormalizedElement {
    NormalizedElement {
        element_id: element_id.into(),
        kind: kind.into(),
        text: text.into(),
        fragment: None,
        starts_new_page: false,
        scene_number: None,
        block_kind: None,
        block_id: None,
        dual_dialogue_group: None,
        dual_dialogue_side: None,
    }
}

trait NormalizedElementExt {
    fn with_starts_new_page(self, starts_new_page: bool) -> Self;
    fn with_block_kind(self, block_kind: &str) -> Self;
    fn in_block(self, block_id: &str) -> Self;
    fn in_dual(self, group_id: &str, side: u8) -> Self;
}

impl NormalizedElementExt for NormalizedElement {
    fn with_starts_new_page(mut self, starts_new_page: bool) -> Self {
        self.starts_new_page = starts_new_page;
        self
    }

    fn with_block_kind(mut self, block_kind: &str) -> Self {
        self.block_kind = Some(block_kind.into());
        self
    }

    fn in_block(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    fn in_dual(mut self, group_id: &str, side: u8) -> Self {
        self.dual_dialogue_group = Some(group_id.into());
        self.dual_dialogue_side = Some(side);
        self
    }
}
