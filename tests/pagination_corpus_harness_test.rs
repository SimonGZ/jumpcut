use jumpcut::pagination::{
    build_semantic_screenplay, compare_paginated_to_fixture, normalize_screenplay,
    ComparisonIssueKind, NormalizedScreenplay, PageBreakFixture, PaginatedScreenplay,
    PaginationConfig,
};
use jumpcut::parse;
use serde::Serialize;
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
fn big_fish_public_slice_stays_at_or_better_than_width_measurement_baseline() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized = normalized_slice_from_fountain(
        "big-fish",
        "benches/Big-Fish.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);

    let (_, _, report) = best_probe_report(&fixture, &semantic);

    assert!(
        report.total_issues() <= 13,
        "expected total issues <= 13, got {}: {:?}",
        report.total_issues(),
        report.issues
    );
    assert!(
        report.issue_count(ComparisonIssueKind::WrongPage) <= 7,
        "expected wrong-page issues <= 7, got {}: {:?}",
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issues
    );
    assert!(
        report.issue_count(ComparisonIssueKind::WrongFragment) <= 1,
        "expected wrong-fragment issues <= 1, got {}: {:?}",
        report.issue_count(ComparisonIssueKind::WrongFragment),
        report.issues
    );
    assert!(
        report.issues.iter().all(|issue| issue.text_preview.is_some()),
        "expected all issues to carry text previews: {:?}",
        report.issues
    );
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
    let (score, lines_per_page, report) = best_probe_report(&fixture, &semantic);
    println!(
        "{}",
        serde_json::to_string_pretty(&ProbeDebugOutput {
            lines_per_page,
            score,
            total_issues: report.total_issues(),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report,
        })
        .unwrap()
    );
}

fn best_probe_report(
    fixture: &PageBreakFixture,
    semantic: &jumpcut::pagination::SemanticScreenplay,
) -> ((usize, usize, usize), u32, jumpcut::pagination::ComparisonReport) {
    let mut best = None;
    for lines_per_page in 1..=20 {
        let actual = PaginatedScreenplay::paginate(
            semantic.clone(),
            PaginationConfig::screenplay(lines_per_page),
            fixture.style_profile.clone(),
            fixture.scope.clone(),
        );
        let report = compare_paginated_to_fixture(&actual, fixture);
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

    best.unwrap()
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

#[derive(Serialize)]
struct ProbeDebugOutput {
    lines_per_page: u32,
    score: (usize, usize, usize),
    total_issues: usize,
    wrong_page: usize,
    wrong_fragment: usize,
    missing: usize,
    unexpected: usize,
    report: jumpcut::pagination::ComparisonReport,
}
