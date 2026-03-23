use jumpcut::pagination::{
    build_semantic_screenplay, compare_paginated_to_fixture, normalize_screenplay,
    ComparisonIssueKind, NormalizedScreenplay, PageBreakFixture, PaginatedScreenplay,
    PaginationConfig,
};
use jumpcut::parse;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

#[test]
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
#[ignore = "diagnostic corpus probe"]
fn probe_big_fish_public_slice_against_canonical_fixture() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized = normalized_slice_from_fountain(
        "big-fish",
        "benches/Big-Fish.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);

    let mut best = None;
    for lines_per_page in 1..=12 {
        let actual = PaginatedScreenplay::paginate(
            semantic.clone(),
            PaginationConfig { lines_per_page },
            fixture.style_profile.clone(),
            fixture.scope.clone(),
        );
        let report = compare_paginated_to_fixture(&actual, &fixture);
        let score = (
            report.total_issues(),
            report.issue_count(ComparisonIssueKind::WrongPage),
            report.issue_count(ComparisonIssueKind::WrongFragment),
        );

        match &best {
            Some((best_score, _, _)) if best_score <= &score => {}
            _ => best = Some((score, lines_per_page, report)),
        }
    }

    let (score, lines_per_page, report) = best.unwrap();
    println!(
        "best lines_per_page={lines_per_page} score={score:?} total={} wrong_page={} wrong_fragment={} missing={} unexpected={}",
        report.total_issues(),
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issue_count(ComparisonIssueKind::WrongFragment),
        report.issue_count(ComparisonIssueKind::MissingOccurrence),
        report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
    );
    for issue in &report.issues {
        println!("{issue:?}");
    }
}

fn normalized_slice_from_fountain(
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
                element.element_id.as_str() >= *first_id
                    && element.element_id.as_str() <= *last_id
            })
            .collect(),
    }
}

fn read_fixture<T: DeserializeOwned>(path: &str) -> T {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}
