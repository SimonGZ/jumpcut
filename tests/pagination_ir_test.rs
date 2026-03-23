use jumpcut::pagination::{
    normalize_screenplay, BlockPlacement, ContinuationMarker, NormalizedElement,
    NormalizedScreenplay, PageBreakFixture, PageKind, PaginatedScreenplay, PaginationScope,
};
use jumpcut::parse;
use pretty_assertions::assert_eq;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

#[test]
fn normalized_screenplay_matches_brick_n_steel_fixture() {
    let fountain = fs::read_to_string("tests/fixtures/Brick-n-Steel.fountain").unwrap();
    let screenplay = parse(&fountain);
    let actual = normalize_screenplay("brick-n-steel", &screenplay);
    let expected: NormalizedScreenplay =
        read_fixture("tests/fixtures/pagination/brick-n-steel.normalized.json");

    assert_eq!(
        actual.elements[..expected.elements.len()],
        expected.elements[..],
    );
}

#[test]
fn paginated_ir_preserves_dual_dialogue_placement() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/brick-n-steel.page-breaks.json");
    let actual = PaginatedScreenplay::from_fixture(fixture);
    let page = &actual.pages[0];

    assert_eq!(page.metadata.number, 2);
    assert_eq!(page.metadata.kind, PageKind::Body);
    assert_eq!(page.metadata.body_page_number, Some(1));

    let dual_blocks: Vec<_> = page
        .blocks
        .iter()
        .filter(|block| matches!(block.placement, BlockPlacement::DualDialogue { .. }))
        .collect();

    assert_eq!(dual_blocks.len(), 2);
    assert_eq!(
        dual_blocks[0].placement,
        BlockPlacement::DualDialogue {
            group_id: "dual-00001".into(),
            side: 1,
        }
    );
    assert_eq!(dual_blocks[0].item_ids, vec!["el-00020", "el-00021"]);
    assert_eq!(
        dual_blocks[1].placement,
        BlockPlacement::DualDialogue {
            group_id: "dual-00001".into(),
            side: 2,
        }
    );
    assert_eq!(dual_blocks[1].item_ids, vec!["el-00022", "el-00023"]);
}

#[test]
fn paginated_ir_surfaces_split_continuation_markers() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let actual = PaginatedScreenplay::from_fixture(fixture);

    assert_eq!(actual.pages[0].metadata.body_page_number, Some(17));
    assert_eq!(actual.pages[1].metadata.body_page_number, Some(18));

    let outgoing = actual.pages[0]
        .items
        .iter()
        .find(|item| item.element_id == "el-00348")
        .unwrap();
    assert_eq!(
        outgoing.continuation_markers,
        vec![ContinuationMarker::More]
    );

    let incoming = actual.pages[1]
        .items
        .iter()
        .find(|item| item.element_id == "el-00348")
        .unwrap();
    assert_eq!(
        incoming.continuation_markers,
        vec![ContinuationMarker::Continued]
    );

    let outgoing_block = actual.pages[0]
        .blocks
        .iter()
        .find(|block| block.id == "el-00348")
        .unwrap();
    assert_eq!(outgoing_block.item_ids, vec!["el-00348"]);
    assert_eq!(
        outgoing_block.continuation_markers,
        vec![ContinuationMarker::More]
    );

    let incoming_block = actual.pages[1]
        .blocks
        .iter()
        .find(|block| block.id == "el-00348")
        .unwrap();
    assert_eq!(
        incoming_block.continuation_markers,
        vec![ContinuationMarker::Continued]
    );
}

#[test]
fn paginated_ir_from_normalized_honors_explicit_page_starts() {
    let normalized = NormalizedScreenplay {
        screenplay: "sample".into(),
        elements: vec![
            normalized_element("el-00001", "Scene Heading", false, None, None, None),
            normalized_element(
                "el-00002",
                "Character",
                false,
                Some("block-00001"),
                None,
                None,
            ),
            normalized_element(
                "el-00003",
                "Dialogue",
                false,
                Some("block-00001"),
                None,
                None,
            ),
            normalized_element("el-00004", "Action", true, None, None, None),
        ],
    };

    let actual = PaginatedScreenplay::from_normalized(
        normalized,
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );

    assert_eq!(actual.screenplay, "sample");
    assert_eq!(actual.style_profile, "standard");
    assert_eq!(actual.pages.len(), 2);

    assert_eq!(actual.pages[0].metadata.number, 2);
    assert_eq!(actual.pages[0].metadata.kind, PageKind::Body);
    assert_eq!(actual.pages[0].metadata.body_page_number, Some(1));
    assert_eq!(
        actual.pages[0]
            .items
            .iter()
            .map(|item| item.element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["el-00001", "el-00002", "el-00003"]
    );
    assert_eq!(actual.pages[0].blocks.len(), 2);
    assert_eq!(actual.pages[0].blocks[1].id, "block-00001");
    assert_eq!(
        actual.pages[0].blocks[1].item_ids,
        vec!["el-00002", "el-00003"]
    );

    assert_eq!(actual.pages[1].metadata.number, 3);
    assert_eq!(actual.pages[1].metadata.body_page_number, Some(2));
    assert_eq!(actual.pages[1].items.len(), 1);
    assert_eq!(actual.pages[1].items[0].element_id, "el-00004");
    assert_eq!(
        actual.pages[1].items[0].continuation_markers,
        Vec::<ContinuationMarker>::new()
    );
}

#[test]
fn paginated_ir_from_normalized_preserves_dual_dialogue_placement() {
    let normalized = NormalizedScreenplay {
        screenplay: "sample".into(),
        elements: vec![
            normalized_element(
                "el-00001",
                "Character",
                false,
                Some("block-00001"),
                Some("dual-00001"),
                Some(1),
            ),
            normalized_element(
                "el-00002",
                "Dialogue",
                false,
                Some("block-00001"),
                Some("dual-00001"),
                Some(1),
            ),
            normalized_element(
                "el-00003",
                "Character",
                false,
                Some("block-00002"),
                Some("dual-00001"),
                Some(2),
            ),
            normalized_element(
                "el-00004",
                "Dialogue",
                false,
                Some("block-00002"),
                Some("dual-00001"),
                Some(2),
            ),
        ],
    };

    let actual = PaginatedScreenplay::from_normalized(
        normalized,
        "standard",
        PaginationScope {
            title_page_count: Some(1),
            body_start_page: Some(2),
        },
    );
    let page = &actual.pages[0];

    assert_eq!(page.metadata.number, 2);
    assert_eq!(page.blocks.len(), 2);
    assert_eq!(
        page.blocks[0].placement,
        BlockPlacement::DualDialogue {
            group_id: "dual-00001".into(),
            side: 1,
        }
    );
    assert_eq!(page.blocks[0].item_ids, vec!["el-00001", "el-00002"]);
    assert_eq!(
        page.blocks[1].placement,
        BlockPlacement::DualDialogue {
            group_id: "dual-00001".into(),
            side: 2,
        }
    );
    assert_eq!(page.blocks[1].item_ids, vec!["el-00003", "el-00004"]);
}

fn read_fixture<T: DeserializeOwned>(path: &str) -> T {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn normalized_element(
    element_id: &str,
    kind: &str,
    starts_new_page: bool,
    block_id: Option<&str>,
    dual_dialogue_group: Option<&str>,
    dual_dialogue_side: Option<u8>,
) -> NormalizedElement {
    NormalizedElement {
        element_id: element_id.into(),
        kind: kind.into(),
        text: String::new(),
        starts_new_page,
        scene_number: None,
        block_kind: block_id.map(|_| "DialogueBlock".into()),
        block_id: block_id.map(str::to_string),
        dual_dialogue_group: dual_dialogue_group.map(str::to_string),
        dual_dialogue_side,
    }
}
