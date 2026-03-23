use jumpcut::pagination::{
    normalize_screenplay, BlockPlacement, ContinuationMarker, NormalizedScreenplay,
    PageBreakFixture, PageKind, PaginatedScreenplay,
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
    assert_eq!(outgoing.continuation_markers, vec![ContinuationMarker::More]);

    let incoming = actual.pages[1]
        .items
        .iter()
        .find(|item| item.element_id == "el-00348")
        .unwrap();
    assert_eq!(incoming.continuation_markers, vec![ContinuationMarker::Continued]);

    let outgoing_block = actual.pages[0]
        .blocks
        .iter()
        .find(|block| block.id == "el-00348")
        .unwrap();
    assert_eq!(outgoing_block.item_ids, vec!["el-00348"]);
    assert_eq!(outgoing_block.continuation_markers, vec![ContinuationMarker::More]);

    let incoming_block = actual.pages[1]
        .blocks
        .iter()
        .find(|block| block.id == "el-00348")
        .unwrap();
    assert_eq!(incoming_block.continuation_markers, vec![ContinuationMarker::Continued]);
}

fn read_fixture<T: DeserializeOwned>(path: &str) -> T {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}
