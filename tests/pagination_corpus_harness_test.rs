use jumpcut::pagination::{
    build_semantic_screenplay, compare_paginated_to_fixture,
    normalize_screenplay, wrapping::ElementType, ComparisonIssueKind,
    DialoguePartKind, FlowKind, Fragment, LayoutGeometry, LineRange, NormalizedElement,
    NormalizedScreenplay, PageBreakFixture, PaginatedScreenplay, PaginationConfig,
};
use jumpcut::parse;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// This harness mixes two kinds of checks:
// - page-break parity tests, which run the main parser -> semantic -> composer -> paginator pipeline
// - line-break parity diagnostics, which compare PDF-extracted lines against direct wrapping of
//   normalized items using production wrapping helpers, but do not run the full pagination engine

#[test]
// #[ignore = "Temporarily disabled"]
fn comparison_reports_no_issues_for_fixture_round_trip() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let actual = PaginatedScreenplay::from_fixture(fixture.clone());
    let report = compare_paginated_to_fixture(&actual, &fixture);

    assert_eq!(report.expected_page_count, fixture.pages.len());
    assert_eq!(report.actual_page_count, fixture.pages.len());
    assert!(report.issues.is_empty());
}

#[test]
// #[ignore = "Temporarily disabled"]
fn selected_big_fish_window_fixtures_round_trip() {
    for path in [
        "tests/fixtures/pagination/big-fish.p38-40.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p42-44.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p55-57.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p77-79.page-breaks.json",
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let actual = PaginatedScreenplay::from_fixture(fixture.clone());
        let report = compare_paginated_to_fixture(&actual, &fixture);

        assert_eq!(report.expected_page_count, fixture.pages.len(), "{path}");
        assert_eq!(report.actual_page_count, fixture.pages.len(), "{path}");
        assert!(report.issues.is_empty(), "{path}: {:?}", report.issues);
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
fn selected_public_window_fixtures_round_trip() {
    for path in [
        "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
        "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
        "tests/fixtures/pagination/little-women.p13-14.page-breaks.json",
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let actual = PaginatedScreenplay::from_fixture(fixture.clone());
        let report = compare_paginated_to_fixture(&actual, &fixture);

        assert_eq!(report.expected_page_count, fixture.pages.len(), "{path}");
        assert_eq!(report.actual_page_count, fixture.pages.len(), "{path}");
        assert!(report.issues.is_empty(), "{path}: {:?}", report.issues);
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
fn selected_public_windows_have_useful_exact_unique_pdf_line_matches() {
    for (path, screenplay_id, fountain_path, min_exact_unique) in [
        (
            "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
            "brick-n-steel",
            "tests/fixtures/corpus/public/brick-n-steel/source/source.fountain",
            50,
        ),
        (
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "little-women",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
            45,
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_window_from_fountain(screenplay_id, fountain_path, &fixture);
        let debug = canonical_pdf_line_count_debug(screenplay_id, &fixture, &normalized);

        assert_eq!(
            debug.supported_items + debug.unsupported_items,
            debug.items.len(),
            "{path}"
        );
        assert!(
            debug.exact_unique_items >= min_exact_unique,
            "{path}: only {} exact-unique PDF matches",
            debug.exact_unique_items
        );
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
fn pdf_line_count_diagnostic_confirms_big_fish_el_00787_is_one_line() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.p38-40.page-breaks.json");
    let normalized =
        normalized_window_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
    let debug = canonical_pdf_line_count_debug("big-fish", &fixture, &normalized);
    let item = debug
        .items
        .iter()
        .find(|item| item.element_id == "el-00787")
        .unwrap();

    assert_eq!(item.match_kind, "exact_unique");
    assert_eq!(item.pdf_line_count, Some(1));
    assert_eq!(item.line_span, Some((10, 10)));
}

#[test]
// #[ignore = "Temporarily disabled"]
fn big_fish_public_slice_stays_at_or_better_than_width_measurement_baseline() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized =
        normalized_window_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
    let semantic = build_semantic_screenplay(normalized);

    let report = run_window_parity_check(&fixture, &semantic, geometry_for_screenplay("big-fish"));

    assert!(
        report.total_issues() == 0,
        "expected zero issues, got {}: {:?}",
        report.total_issues(),
        report.issues
    );
    assert!(
        report.issue_count(ComparisonIssueKind::WrongPage) == 0,
        "expected zero wrong-page issues, got {}: {:?}",
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issues
    );
    assert!(
        report.issue_count(ComparisonIssueKind::WrongFragment) == 0,
        "expected zero wrong-fragment issues, got {}: {:?}",
        report.issue_count(ComparisonIssueKind::WrongFragment),
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .all(|issue| issue.text_preview.is_some()),
        "expected all issues to carry text previews: {:?}",
        report.issues
    );
}

#[test]
fn big_fish_pages_34_35_split_mayor_speech_at_sentence_boundary() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 34 | 35));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let report = run_window_parity_check(&fixture, &semantic, geometry_for_screenplay("big-fish"));

    assert!(
        report.issues.is_empty(),
        "expected the page 34/35 window to match canonical split behavior for block-00213, got {:?}",
        report.issues
    );
}

#[test]
fn big_fish_pages_38_39_split_beamen_action_at_sentence_boundary() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 38 | 39));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry_for_screenplay("big-fish"),
    };
    let semantic = build_semantic_screenplay(normalized);
    let blocks = jumpcut::pagination::composer::compose(&semantic.units, &config.geometry);
    let pages = jumpcut::pagination::paginator::paginate(&blocks, config.lines_per_page, &config.geometry);

    let page_38_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Flow(jumpcut::pagination::FlowUnit {
                        element_id,
                        ..
                    }) if element_id == "el-00800" && block.fragment == Fragment::ContinuedToNext
                )
            })
        })
        .expect("expected el-00800 top fragment");
    let page_39_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Flow(jumpcut::pagination::FlowUnit {
                        element_id,
                        ..
                    }) if element_id == "el-00800" && block.fragment == Fragment::ContinuedFromPrev
                )
            })
        })
        .expect("expected el-00800 bottom fragment");

    let page_38_text = flow_fragment_text(page_38_block, &config.geometry);
    let page_39_text = flow_fragment_text(page_39_block, &config.geometry);

    assert!(
        page_38_text.trim_end().ends_with("to greet Edward."),
        "expected page 38 fragment to end after 'to greet Edward.', got: {:?}",
        page_38_text
    );
    assert!(
        page_39_text
            .trim_start()
            .starts_with("Friendly but a little drunk, he's the closest thing the town"),
        "expected page 39 fragment to start with 'Friendly but a little drunk, he's the closest thing the town', got: {:?}",
        page_39_text
    );
}

#[test]
fn big_fish_pages_53_54_split_el_01146_after_was_gone_for_a_long_time() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 52 | 53 | 54));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry_for_screenplay("big-fish"),
    };
    let blocks = jumpcut::pagination::composer::compose(&semantic.units, &config.geometry);
    let pages = jumpcut::pagination::paginator::paginate(&blocks, config.lines_per_page, &config.geometry);

    let page_53_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Dialogue(jumpcut::pagination::DialogueUnit {
                        block_id,
                        ..
                    }) if block_id == "block-00343" && block.fragment == Fragment::ContinuedToNext
                )
            })
        })
        .expect("expected el-01146 top fragment");
    let page_54_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Dialogue(jumpcut::pagination::DialogueUnit {
                        block_id,
                        ..
                    }) if block_id == "block-00343" && block.fragment == Fragment::ContinuedFromPrev
                )
            })
        })
        .expect("expected el-01146 bottom fragment");

    let page_53_text = dialogue_block_fragment_text(page_53_block);
    let page_54_text = dialogue_block_fragment_text(page_54_block);

    assert!(
        page_53_text.trim_end().ends_with("was gone for a long time."),
        "expected page 53 fragment to end after 'was gone for a long time.', got: {:?}",
        page_53_text
    );
    assert!(
        page_54_text
            .trim_start()
            .starts_with("And when he finally came back, he looked"),
        "expected page 54 fragment to start with 'And when he finally came back, he looked', got: {:?}",
        page_54_text
    );
}

#[test]
fn big_fish_pages_81_82_keep_el_01742_whole_on_page_82() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 81 | 82));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry_for_screenplay("big-fish"),
    };
    let actual = PaginatedScreenplay::paginate(
        semantic,
        config,
        fixture.style_profile.clone(),
        fixture.scope.clone(),
    );

    let page_81_item = actual
        .pages
        .iter()
        .find(|page| page.metadata.number == 81)
        .and_then(|page| page.items.iter().find(|item| item.element_id == "el-01742"));
    let page_82_item = actual
        .pages
        .iter()
        .find(|page| page.metadata.number == 82)
        .and_then(|page| page.items.iter().find(|item| item.element_id == "el-01742"))
        .expect("expected el-01742 on page 82");

    assert!(
        page_81_item.is_none(),
        "expected el-01742 to be pushed entirely off page 81, got: {:?}",
        page_81_item
    );
    assert_eq!(page_82_item.fragment, Fragment::Whole);
}

#[test]
fn big_fish_pages_93_94_keep_block_00582_whole_on_page_94() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 93 | 94));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry_for_screenplay("big-fish"),
    };
    let actual = PaginatedScreenplay::paginate(
        semantic,
        config,
        fixture.style_profile.clone(),
        fixture.scope.clone(),
    );

    let page_93_block_items = actual
        .pages
        .iter()
        .find(|page| page.metadata.number == 93)
        .map(|page| {
            page.items
                .iter()
                .filter(|item| item.block_id.as_deref() == Some("block-00582"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let page_94_block_items = actual
        .pages
        .iter()
        .find(|page| page.metadata.number == 94)
        .map(|page| {
            page.items
                .iter()
                .filter(|item| item.block_id.as_deref() == Some("block-00582"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    assert!(
        page_93_block_items.is_empty(),
        "expected block-00582 to be pushed entirely off page 93, got: {:?}",
        page_93_block_items
    );
    assert_eq!(page_94_block_items.len(), 2);
    assert!(
        page_94_block_items
            .iter()
            .all(|item| item.fragment == Fragment::Whole),
        "expected block-00582 to stay whole on page 94, got: {:?}",
        page_94_block_items
    );
}

#[test]
fn big_fish_pages_34_35_split_block_00213_after_go() {
    let mut fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json");
    fixture.pages.retain(|page| matches!(page.number, 34 | 35));

    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry_for_screenplay("big-fish"),
    };
    let blocks = jumpcut::pagination::composer::compose(&semantic.units, &config.geometry);
    let pages = jumpcut::pagination::paginator::paginate(&blocks, config.lines_per_page, &config.geometry);

    let page_34_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Dialogue(jumpcut::pagination::DialogueUnit {
                        block_id,
                        ..
                    }) if block_id == "block-00213" && block.fragment == Fragment::ContinuedToNext
                )
            })
        })
        .expect("expected block-00213 top fragment");
    let page_35_block = pages
        .iter()
        .find_map(|page| {
            page.blocks.iter().find(|block| {
                matches!(
                    block.unit,
                    jumpcut::pagination::SemanticUnit::Dialogue(jumpcut::pagination::DialogueUnit {
                        block_id,
                        ..
                    }) if block_id == "block-00213" && block.fragment == Fragment::ContinuedFromPrev
                )
            })
        })
        .expect("expected block-00213 bottom fragment");

    let page_34_text = dialogue_block_fragment_text(page_34_block);
    let page_35_text = dialogue_block_fragment_text(page_35_block);

    assert!(
        page_34_text.trim_end().ends_with("go."),
        "expected page 34 fragment to end after 'go.', got: {:?}",
        page_34_text
    );
    assert!(
        page_35_text
            .trim_start()
            .starts_with("But take with you this Key"),
        "expected page 35 fragment to start with 'But take with you this Key', got: {:?}",
        page_35_text
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
fn big_fish_line_break_parity_reports_el_00787_as_an_exact_match() {
    let report = build_line_break_parity_report(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
    );
    let item = report
        .items
        .iter()
        .find(|item| item.element_id == "el-00787")
        .unwrap();

    assert_eq!(item.match_kind, "exact_unique");
    assert_eq!(item.pdf_line_count, Some(1));
    // el-00787: "...into the spiderwebs." is 61 chars. With action width = 61 it fits
    // on one line, matching the PDF without any punctuation-overhang special casing.
    assert_eq!(item.expected_wrapped_lines.len(), 1);
    assert_eq!(item.lines_agree, Some(true));
}

#[test]
fn big_fish_line_break_parity_recovers_el_00533_hyphenated_pdf_match() {
    let report = build_line_break_parity_report(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
    );
    let item = report
        .items
        .iter()
        .find(|item| item.element_id == "el-00533")
        .unwrap();

    assert_eq!(item.match_kind, "exact_unique");
    assert_eq!(item.pdf_line_count, Some(2));
    assert_eq!(item.lines_agree, Some(true));
    assert_eq!(item.expected_wrapped_lines.len(), 2);
    assert_eq!(item.pdf_lines.len(), 2);
}

#[test]
// #[ignore = "Temporarily disabled"]
fn big_fish_macro_parity_holds_baseline() {
    let report = build_line_break_parity_report(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
    );
    
    // As of the new Geometry Engine integration and Parenthetical wrap fixes, there are exactly 0 disagreements
    // across the entire 120-page screenplay compared to Canonical Final Draft PDFs!
    assert_eq!(
        report.disagreement_count, 0,
        "Expected exact macro parity baseline of 0 disagreements. If this worsened, fix the regression!"
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
fn brick_n_steel_macro_parity_holds_baseline() {
    let report = build_line_break_parity_report(
        "brick-n-steel",
        "tests/fixtures/corpus/public/brick-n-steel/source/source.fountain",
        "tests/fixtures/corpus/public/brick-n-steel/canonical/page-breaks.json",
    );

    assert_eq!(
        report.disagreement_count, 0,
        "Expected Brick & Steel line-break parity against the Final Draft PDF to have 0 disagreements. If this fails, inspect the report and decide which pagination assumptions are wrong."
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
fn little_women_macro_parity_holds_baseline() {
    let report = build_line_break_parity_report(
        "little-women",
        "tests/fixtures/corpus/public/little-women/source/source.fountain",
        "tests/fixtures/corpus/public/little-women/canonical/page-breaks.json",
    );

    assert_eq!(
        report.disagreement_count, 0,
        "Expected Little Women line-break parity against the Final Draft PDF to have 0 disagreements. If this fails, inspect the report and decide which pagination assumptions are wrong."
    );
}

#[test]
fn mostly_genius_line_break_diagnostic_report_includes_multicam_act_markers() {
    let report = build_line_break_parity_report(
        "mostly-genius",
        "tests/fixtures/corpus/public/mostly-genius/source/source.fountain",
        "tests/fixtures/corpus/public/mostly-genius/canonical/page-breaks.json",
    );

    let new_act_count = report
        .items
        .iter()
        .filter(|item| item.kind == "New Act")
        .count();
    let end_of_act_count = report
        .items
        .iter()
        .filter(|item| item.kind == "End of Act")
        .count();

    assert_eq!(new_act_count, 3);
    assert_eq!(end_of_act_count, 3);
    assert!(
        report.exact_unique_count > 0,
        "Expected the multicam diagnostic report to find at least some exact-unique PDF matches."
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
fn brick_n_steel_full_script_page_break_parity_holds_baseline() {
    let fixture: PageBreakFixture = read_fixture(
        "tests/fixtures/corpus/public/brick-n-steel/canonical/page-breaks.json",
    );
    let fountain = fs::read_to_string("tests/fixtures/corpus/public/brick-n-steel/source/source.fountain")
        .unwrap();
    let screenplay = parse(&fountain);
    let actual = PaginatedScreenplay::from_screenplay(
        "brick-n-steel",
        &screenplay,
        54.0,
        fixture.scope.clone(),
    );
    let report = compare_paginated_to_fixture(&actual, &fixture);

    assert_eq!(
        report.total_issues(),
        0,
        "Expected Brick & Steel full-script page-break parity against the Final Draft canonical fixture to have 0 issues. If this fails, inspect the report and decide which pagination assumptions are wrong."
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
fn little_women_full_script_page_break_parity_holds_baseline() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/corpus/public/little-women/canonical/page-breaks.json");
    let fountain = fs::read_to_string("tests/fixtures/corpus/public/little-women/source/source.fountain")
        .unwrap();
    let screenplay = parse(&fountain);
    let actual = PaginatedScreenplay::from_screenplay(
        "little-women",
        &screenplay,
        54.0,
        fixture.scope.clone(),
    );
    let report = compare_paginated_to_fixture(&actual, &fixture);

    assert_eq!(
        report.total_issues(),
        0,
        "Expected Little Women full-script page-break parity against the Final Draft canonical fixture to have 0 issues. If this fails, inspect the report and decide which pagination assumptions are wrong."
    );
}


#[test]
fn dual_dialogue_parity_items_use_dual_dialogue_width_and_surface_dual_metadata() {
    let geometry = LayoutGeometry::default();
    let element = NormalizedElement {
        element_id: "el-dual".into(),
        kind: "Dialogue".into(),
        text: "12345678901234567890123456789 12345678901234567890123456789".into(),
        fragment: None,
        starts_new_page: false,
        scene_number: None,
        block_kind: Some("DialogueBlock".into()),
        block_id: Some("block-00001".into()),
        dual_dialogue_group: Some("dual-00001".into()),
        dual_dialogue_side: Some(1),
    };

    let item = build_line_break_parity_item(
        1,
        &element.element_id,
        "Dialogue",
        &Fragment::Whole,
        None,
        &element.dual_dialogue_group,
        element.dual_dialogue_side,
        Some(&element),
        &[
            "12345678901234567890123456789".into(),
            "12345678901234567890123456789".into(),
        ],
        &geometry,
    );

    assert_eq!(item.width_chars, Some(29));
    assert_eq!(item.dual_dialogue_group.as_deref(), Some("dual-00001"));
    assert_eq!(item.dual_dialogue_side, Some(1));
    assert_eq!(item.lines_agree, Some(true));
}

fn run_window_parity_check(
    fixture: &PageBreakFixture,
    semantic: &jumpcut::pagination::SemanticScreenplay,
    geometry: LayoutGeometry,
) -> jumpcut::pagination::ComparisonReport {
    let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry,
    };
    let full_actual = PaginatedScreenplay::paginate(
        semantic.clone(),
        config,
        fixture.style_profile.clone(),
        fixture.scope.clone(),
    );
    let actual = slice_paginated_to_fixture_window(&full_actual, &page_numbers);
    compare_paginated_to_fixture(&actual, fixture)
}

fn geometry_for_screenplay(screenplay_id: &str) -> LayoutGeometry {
    let path = Path::new("tests/fixtures/corpus/public")
        .join(screenplay_id)
        .join("source/source.fountain");
    let fountain = fs::read_to_string(path).unwrap();
    let screenplay = parse(&fountain);
    PaginationConfig::from_screenplay(&screenplay, 54.0).geometry
}

fn normalized_window_from_fountain(
    screenplay_id: &str,
    fountain_path: &str,
    fixture: &PageBreakFixture,
) -> NormalizedScreenplay {
    let fountain = fs::read_to_string(fountain_path).unwrap();
    let screenplay = parse(&fountain);
    let normalized = normalize_screenplay(screenplay_id, &screenplay);
    let expected_ids: Vec<&str> = fixture
        .pages
        .iter()
        .flat_map(|page| page.items.iter().map(|item| item.element_id.as_str()))
        .collect();
    let first_id = expected_ids.first().unwrap();
    let last_id = expected_ids.last().unwrap();

    NormalizedScreenplay {
        screenplay: normalized.screenplay,
        starting_page_number: fixture.pages.first().map(|page| page.number),
        elements: normalized
            .elements
            .into_iter()
            .filter(|element| {
                element.element_id.as_str() >= *first_id && element.element_id.as_str() <= *last_id
            })
            .collect(),
    }
}

fn slice_paginated_to_fixture_window(
    actual: &PaginatedScreenplay,
    page_numbers: &[u32],
) -> PaginatedScreenplay {
    PaginatedScreenplay {
        screenplay: actual.screenplay.clone(),
        style_profile: actual.style_profile.clone(),
        source: actual.source.clone(),
        scope: actual.scope.clone(),
        pages: actual
            .pages
            .iter()
            .filter(|page| page_numbers.contains(&page.metadata.number))
            .cloned()
            .collect(),
    }
}

fn flow_fragment_text(
    block: &jumpcut::pagination::composer::LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> String {
    let jumpcut::pagination::SemanticUnit::Flow(flow) = block.unit else {
        panic!("expected flow block");
    };

    if let Some(plan) = block.flow_split.as_ref() {
        return match block.fragment {
            Fragment::Whole => flow.text.clone(),
            Fragment::ContinuedToNext => plan.top_text.clone(),
            Fragment::ContinuedFromPrev => plan.bottom_text.clone(),
            Fragment::ContinuedFromPrevAndToNext => plan.top_text.clone(),
        };
    }

    let element_type = ElementType::from_flow_kind(&flow.kind);
    let config = jumpcut::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    let wrapped_lines = jumpcut::pagination::wrapping::wrap_text_for_element(&flow.text, &config);
    let fragment_line_count = (block.content_lines / geometry.line_height).round() as usize;

    match block.fragment {
        Fragment::Whole => wrapped_lines,
        Fragment::ContinuedToNext => wrapped_lines.into_iter().take(fragment_line_count).collect(),
        Fragment::ContinuedFromPrev => {
            let len = wrapped_lines.len();
            wrapped_lines
                .into_iter()
                .skip(len.saturating_sub(fragment_line_count))
                .collect()
        }
        Fragment::ContinuedFromPrevAndToNext => wrapped_lines.into_iter().take(fragment_line_count).collect(),
    }
    .join("\n")
}

fn dialogue_block_fragment_text(block: &jumpcut::pagination::composer::LayoutBlock<'_>) -> String {
    let jumpcut::pagination::SemanticUnit::Dialogue(unit) = block.unit else {
        panic!("expected dialogue block");
    };
    let plan = block
        .dialogue_split
        .as_ref()
        .expect("expected dialogue split plan");

    unit.parts
        .iter()
        .zip(plan.parts.iter())
        .map(|(part, part_plan)| match block.fragment {
            Fragment::Whole => {
                if part_plan.bottom_text.is_empty() {
                    part.text.clone()
                } else {
                    [part_plan.top_text.as_str(), part_plan.bottom_text.as_str()]
                        .into_iter()
                        .filter(|text| !text.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Fragment::ContinuedToNext => part_plan.top_text.clone(),
            Fragment::ContinuedFromPrev => part_plan.bottom_text.clone(),
            Fragment::ContinuedFromPrevAndToNext => part_plan.top_text.clone(),
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_fixture<T: DeserializeOwned>(path: &str) -> T {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn debug_flow_geometry(
    kind: &str,
    source_style: &str,
    flow_kind: FlowKind,
    geometry: &LayoutGeometry,
) -> DebugGeometry {
    let (left_indent_in, right_indent_in) = match flow_kind {
        FlowKind::SceneHeading => (
            geometry.action_left,
            geometry.action_right,
        ),
        FlowKind::Transition => (
            geometry.transition_left,
            geometry.transition_right,
        ),
        _ => (
            geometry.action_left,
            geometry.action_right,
        ),
    };

    DebugGeometry {
        kind: kind.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::from_flow_kind(&flow_kind)),
    }
}

fn debug_dialogue_geometry(
    kind: &str,
    source_style: &str,
    part_kind: DialoguePartKind,
    geometry: &LayoutGeometry,
) -> DebugGeometry {
    let (left_indent_in, right_indent_in) = match part_kind {
        DialoguePartKind::Character => (
            geometry.character_left,
            geometry.character_right,
        ),
        DialoguePartKind::Parenthetical => (
            geometry.parenthetical_left,
            geometry.parenthetical_right,
        ),
        DialoguePartKind::Lyric => (
            geometry.lyric_left,
            geometry.lyric_right,
        ),
        DialoguePartKind::Dialogue => (
            geometry.dialogue_left,
            geometry.dialogue_right,
        ),
    };

    DebugGeometry {
        kind: kind.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::from_dialogue_part_kind(&part_kind)),
    }
}

fn text_preview(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect()
}

fn normalized_element_map(normalized: &NormalizedScreenplay) -> HashMap<String, NormalizedElement> {
    normalized
        .elements
        .iter()
        .cloned()
        .map(|element| (element.element_id.clone(), element))
        .collect()
}

fn canonical_pdf_line_count_debug(
    screenplay_id: &str,
    fixture: &PageBreakFixture,
    normalized: &NormalizedScreenplay,
) -> CanonicalPdfLineCountDebug {
    let elements = normalized_element_map(normalized);
    let pdf_pages = public_pdf_pages(screenplay_id);
    let mut supported_items = 0;
    let mut exact_unique_items = 0;
    let mut exact_ambiguous_items = 0;
    let mut unsupported_items = 0;
    let mut items = Vec::new();

    for page in &fixture.pages {
        let page_lines = pdf_pages
            .get(&page.number)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for item in &page.items {
            let result = canonical_pdf_match_for_item(
                page.number,
                &item.element_id,
                &item.kind,
                &item.fragment,
                item.line_range,
                &elements,
                page_lines,
            );
            match result.match_kind.as_str() {
                "exact_unique" => {
                    supported_items += 1;
                    exact_unique_items += 1;
                }
                "exact_ambiguous" => {
                    supported_items += 1;
                    exact_ambiguous_items += 1;
                }
                _ => unsupported_items += 1,
            }
            items.push(result);
        }
    }

    CanonicalPdfLineCountDebug {
        screenplay: normalized.screenplay.clone(),
        supported_items,
        exact_unique_items,
        exact_ambiguous_items,
        unsupported_items,
        items,
    }
}

fn canonical_pdf_match_for_item(
    page_number: u32,
    element_id: &str,
    kind: &str,
    fragment: &Fragment,
    line_range: Option<LineRange>,
    elements: &HashMap<String, NormalizedElement>,
    page_lines: &[String],
) -> CanonicalPdfLineCountItem {
    let Some(element) = elements.get(element_id) else {
        return CanonicalPdfLineCountItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: None,
            match_kind: "missing-element".into(),
            pdf_line_count: None,
            line_span: None,
        };
    };

    let Some(candidate_text) = canonical_pdf_text_for_item(fragment, line_range, element) else {
        return CanonicalPdfLineCountItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&element.text)),
            match_kind: "unsupported-fragment".into(),
            pdf_line_count: None,
            line_span: None,
        };
    };
    let normalized_text = normalize_pdf_match_text(&candidate_text);
    let matches = exact_pdf_line_matches(page_lines, &normalized_text);

    let (match_kind, pdf_line_count, line_span) = match matches.as_slice() {
        [(start, end)] => (
            "exact_unique".into(),
            Some(end - start + 1),
            Some((*start, *end)),
        ),
        [] => ("unmatched".into(), None, None),
        _ => ("exact_ambiguous".into(), None, None),
    };

    CanonicalPdfLineCountItem {
        page_number,
        element_id: element_id.into(),
        kind: kind.into(),
        text_preview: Some(text_preview(&candidate_text)),
        match_kind,
        pdf_line_count,
        line_span,
    }
}

fn canonical_pdf_text_for_item(
    fragment: &Fragment,
    line_range: Option<LineRange>,
    element: &NormalizedElement,
) -> Option<String> {
    match (fragment, line_range) {
        (Fragment::Whole, None) => Some(element.text.clone()),
        (_, Some(LineRange(start, end))) => Some(slice_explicit_lines(&element.text, start, end)),
        _ => None,
    }
}

fn exact_pdf_line_matches(page_lines: &[String], candidate_text: &str) -> Vec<(u32, u32)> {
    let mut matches = Vec::new();
    for start in 0..page_lines.len() {
        let mut accumulated = String::new();
        for end in start..page_lines.len() {
            if !accumulated.is_empty() {
                accumulated.push(' ');
            }
            accumulated.push_str(&page_lines[end]);
            let normalized = normalize_pdf_match_text(&accumulated);
            if normalized == candidate_text {
                matches.push((start as u32 + 1, end as u32 + 1));
            }
            if normalized.len() > candidate_text.len() + 40 {
                break;
            }
        }
    }
    matches
}

fn normalize_pdf_match_text(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            let prev = out.chars().last();
            let mut next_index = index + 1;
            while next_index < chars.len() && chars[next_index].is_whitespace() {
                next_index += 1;
            }

            let next = chars.get(next_index).copied();
            let joins_hyphenated_word =
                matches!(prev, Some('-')) && matches!(next, Some(c) if c.is_alphanumeric());

            if !joins_hyphenated_word && !out.is_empty() && next.is_some() && !out.ends_with(' ') {
                out.push(' ');
            }
            index = next_index;
            continue;
        }

        out.push(ch);
        index += 1;
    }

    out.trim().to_string()
}

fn public_pdf_pages(screenplay_id: &str) -> HashMap<u32, Vec<String>> {
    let path = Path::new("tests/fixtures/corpus/public")
        .join(screenplay_id)
        .join("extracted/pdf-pages.json");
    let pdf_pages: PublicPdfPages =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    pdf_pages
        .pages
        .into_iter()
        .map(|page| (page.number, page.text.lines().map(str::to_string).collect()))
        .collect()
}


fn build_line_break_parity_report(
    screenplay_id: &str,
    fountain_path: &str,
    canonical_page_breaks_path: &str,
) -> LineBreakParityReport {
    let measurement = geometry_for_screenplay(screenplay_id);
    let fountain = fs::read_to_string(fountain_path).unwrap();
    let screenplay = parse(&fountain);
    let parsed = normalize_screenplay(screenplay_id, &screenplay);
    let canonical: PageBreakFixture = read_fixture(canonical_page_breaks_path);
    let pdf_pages = public_pdf_pages(screenplay_id);
    let elements: HashMap<String, NormalizedElement> = parsed
        .elements
        .into_iter()
        .map(|element| (element.element_id.clone(), element))
        .collect();

    let mut items = Vec::new();
    let mut exact_unique_count = 0;
    let mut exact_ambiguous_count = 0;
    let mut unsupported_count = 0;
    let mut disagreement_count = 0;

    for page in &canonical.pages {
        let page_lines = pdf_pages
            .get(&page.number)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for item in &page.items {
            let result = build_line_break_parity_item(
                page.number,
                &item.element_id,
                &item.kind,
                &item.fragment,
                item.line_range,
                &item.dual_dialogue_group,
                item.dual_dialogue_side,
                elements.get(&item.element_id),
                page_lines,
                &measurement,
            );

            match result.match_kind.as_str() {
                "exact_unique" => {
                    exact_unique_count += 1;
                    if result.lines_agree == Some(false) {
                        disagreement_count += 1;
                    }
                }
                "exact_ambiguous" => exact_ambiguous_count += 1,
                _ => unsupported_count += 1,
            }

            items.push(result);
        }
    }

    LineBreakParityReport {
        screenplay: screenplay_id.into(),
        exact_unique_count,
        exact_ambiguous_count,
        unsupported_count,
        disagreement_count,
        measurement: LineBreakParityMeasurement {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, &measurement),
                debug_flow_geometry(
                    "Scene Heading",
                    "Scene Heading",
                    FlowKind::SceneHeading,
                    &measurement,
                ),
                debug_flow_geometry(
                    "Transition",
                    "Transition",
                    FlowKind::Transition,
                    &measurement,
                ),
                debug_flow_geometry(
                    "Cold Opening",
                    "Cold Opening",
                    FlowKind::ColdOpening,
                    &measurement,
                ),
                debug_flow_geometry("New Act", "New Act", FlowKind::NewAct, &measurement),
                debug_flow_geometry("End of Act", "End of Act", FlowKind::EndOfAct, &measurement),
                debug_flow_geometry(
                    "Section",
                    "Action (fallback)",
                    FlowKind::Section,
                    &measurement,
                ),
                debug_flow_geometry(
                    "Synopsis",
                    "Action (fallback)",
                    FlowKind::Synopsis,
                    &measurement,
                ),
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry(
                    "Dialogue",
                    "Dialogue",
                    DialoguePartKind::Dialogue,
                    &measurement,
                ),
                debug_dialogue_geometry(
                    "Character",
                    "Character",
                    DialoguePartKind::Character,
                    &measurement,
                ),
                debug_dialogue_geometry(
                    "Parenthetical",
                    "Parenthetical",
                    DialoguePartKind::Parenthetical,
                    &measurement,
                ),
                debug_dialogue_geometry("Lyric", "Lyric", DialoguePartKind::Lyric, &measurement),
            ],
        },
        items,
    }
}

fn build_line_break_parity_item(
    page_number: u32,
    element_id: &str,
    kind: &str,
    fragment: &Fragment,
    line_range: Option<LineRange>,
    dual_dialogue_group: &Option<String>,
    dual_dialogue_side: Option<u8>,
    element: Option<&NormalizedElement>,
    page_lines: &[String],
    measurement: &LayoutGeometry,
) -> LineBreakParityItem {
    let Some(element) = element else {
        return LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: None,
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: None,
            expected_wrapped_lines: Vec::new(),
            match_kind: "missing-element".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        };
    };

    let Some(candidate_text) = canonical_pdf_text_for_item(fragment, line_range, element) else {
        return LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&element.text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: None,
            expected_wrapped_lines: Vec::new(),
            match_kind: "unsupported-fragment".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        };
    };

    let element_type = ElementType::from_item_kind(kind, dual_dialogue_side);
    let config = jumpcut::pagination::wrapping::WrapConfig::from_geometry(measurement, element_type);
    let width_chars = config.exact_width_chars;
    let expected_wrapped_lines = jumpcut::pagination::wrapping::wrap_text_for_element(
        &candidate_text,
        &config,
    )
        .into_iter()
        .map(|line| normalize_pdf_match_text(&line))
        .collect::<Vec<_>>();
    let normalized_text = normalize_pdf_match_text(&candidate_text);
    let matches = exact_pdf_line_matches(page_lines, &normalized_text);

    match matches.as_slice() {
        [(start, end)] => {
            let pdf_lines = page_lines[*start as usize - 1..*end as usize]
                .iter()
                .map(|line| normalize_pdf_match_text(line))
                .collect::<Vec<_>>();
            let lines_agree = expected_wrapped_lines == pdf_lines;

            LineBreakParityItem {
                page_number,
                element_id: element_id.into(),
                kind: kind.into(),
                text_preview: Some(text_preview(&candidate_text)),
                dual_dialogue_group: dual_dialogue_group.clone(),
                dual_dialogue_side,
                width_chars: Some(width_chars),
                expected_wrapped_lines,
                match_kind: "exact_unique".into(),
                pdf_line_count: Some(end - start + 1),
                pdf_line_span: Some((*start, *end)),
                pdf_lines,
                candidate_spans: vec![(*start, *end)],
                lines_agree: Some(lines_agree),
            }
        }
        [] => LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&candidate_text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: Some(width_chars),
            expected_wrapped_lines,
            match_kind: "unmatched".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: Vec::new(),
            lines_agree: None,
        },
        _ => LineBreakParityItem {
            page_number,
            element_id: element_id.into(),
            kind: kind.into(),
            text_preview: Some(text_preview(&candidate_text)),
            dual_dialogue_group: dual_dialogue_group.clone(),
            dual_dialogue_side,
            width_chars: Some(width_chars),
            expected_wrapped_lines,
            match_kind: "exact_ambiguous".into(),
            pdf_line_count: None,
            pdf_line_span: None,
            pdf_lines: Vec::new(),
            candidate_spans: matches,
            lines_agree: None,
        },
    }
}

fn slice_explicit_lines(text: &str, start: u32, end: u32) -> String {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line_no = index as u32 + 1;
            (line_no >= start && line_no <= end).then_some(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Serialize)]
struct LineBreakParityReport {
    screenplay: String,
    exact_unique_count: usize,
    exact_ambiguous_count: usize,
    unsupported_count: usize,
    disagreement_count: usize,
    measurement: LineBreakParityMeasurement,
    items: Vec<LineBreakParityItem>,
}

#[derive(Serialize)]
struct LineBreakParityMeasurement {
    flow_geometries: Vec<DebugGeometry>,
    dialogue_geometries: Vec<DebugGeometry>,
}

#[derive(Serialize)]
struct LineBreakParityItem {
    page_number: u32,
    element_id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dual_dialogue_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dual_dialogue_side: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width_chars: Option<usize>,
    expected_wrapped_lines: Vec<String>,
    match_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf_line_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf_line_span: Option<(u32, u32)>,
    pdf_lines: Vec<String>,
    candidate_spans: Vec<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lines_agree: Option<bool>,
}

#[derive(Serialize)]
struct DebugGeometry {
    kind: String,
    source_style: String,
    left_indent_in: f32,
    right_indent_in: f32,
    width_chars: usize,
}

#[derive(Serialize)]
struct CanonicalPdfLineCountDebug {
    screenplay: String,
    supported_items: usize,
    exact_unique_items: usize,
    exact_ambiguous_items: usize,
    unsupported_items: usize,
    items: Vec<CanonicalPdfLineCountItem>,
}

#[derive(Serialize)]
struct CanonicalPdfLineCountItem {
    page_number: u32,
    element_id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    match_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf_line_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_span: Option<(u32, u32)>,
}

#[derive(serde::Deserialize)]
struct PublicPdfPages {
    pages: Vec<PublicPdfPage>,
}

#[derive(serde::Deserialize)]
struct PublicPdfPage {
    number: u32,
    text: String,
}
