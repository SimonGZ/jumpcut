//! End-to-end corpus contracts for the two canonical pagination profiles.
//!
//! Industry is compared directly with the compatibility fixtures. Jumpcut has a
//! separate stability snapshot expressed as its exact difference from those same
//! fixtures. Updating Jumpcut behavior therefore requires an intentional snapshot
//! update without weakening the Industry compatibility contract.

use std::fs;

use jumpcut::pagination::composer::compose_with_wrap;
use jumpcut::pagination::paginator::paginate_with_policy;
use jumpcut::pagination::{
    ComparisonIssueKind, ComparisonReport, Fragment, PageBreakFixture, PaginatedScreenplay,
    ScreenplayLayoutProfile, SemanticUnit, build_semantic_screenplay, compare_paginated_to_fixture,
    normalize_screenplay,
};
use jumpcut::{ElementText, Screenplay, parse};

const PUBLIC_CORPUS: &[&str] = &[
    "big-fish",
    "big-fish-scene-numbers",
    "brick-n-steel",
    "gumshoe",
    "little-women",
    "mostly-genius",
    "vikings",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CorpusSnapshot {
    screenplay: &'static str,
    pages: usize,
    missing: usize,
    unexpected: usize,
    wrong_page: usize,
    wrong_fragment: usize,
}

const JUMPCUT_CORPUS_SNAPSHOT: &[CorpusSnapshot] = &[
    CorpusSnapshot::exact("big-fish", 127),
    CorpusSnapshot::exact("big-fish-scene-numbers", 8),
    CorpusSnapshot::exact("brick-n-steel", 4),
    CorpusSnapshot {
        screenplay: "gumshoe",
        pages: 37,
        missing: 211,
        unexpected: 211,
        wrong_page: 49,
        wrong_fragment: 2,
    },
    CorpusSnapshot::exact("little-women", 15),
    CorpusSnapshot {
        screenplay: "mostly-genius",
        pages: 52,
        missing: 0,
        unexpected: 0,
        wrong_page: 4,
        wrong_fragment: 0,
    },
    CorpusSnapshot {
        screenplay: "vikings",
        pages: 113,
        missing: 902,
        unexpected: 903,
        wrong_page: 3,
        wrong_fragment: 1,
    },
];

impl CorpusSnapshot {
    const fn exact(screenplay: &'static str, pages: usize) -> Self {
        Self {
            screenplay,
            pages,
            missing: 0,
            unexpected: 0,
            wrong_page: 0,
            wrong_fragment: 0,
        }
    }

    fn from_report(screenplay: &'static str, report: &ComparisonReport) -> Self {
        Self {
            screenplay,
            pages: report.actual_page_count,
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
        }
    }
}

#[test]
fn industry_matches_the_public_compatibility_corpus() {
    for screenplay_id in PUBLIC_CORPUS {
        let (fixture, screenplay) = load_corpus_case(screenplay_id, "industry");
        let report = report_for_screenplay(screenplay_id, &screenplay, &fixture);

        assert_eq!(
            report.actual_page_count, report.expected_page_count,
            "industry page count changed for {screenplay_id}"
        );
        assert!(
            report.issues.is_empty(),
            "industry compatibility changed for {screenplay_id}: {:?}",
            report.issues
        );
    }
}

#[test]
fn jumpcut_public_corpus_snapshot_is_stable() {
    let actual = JUMPCUT_CORPUS_SNAPSHOT
        .iter()
        .map(|expected| {
            let (fixture, screenplay) = load_corpus_case(expected.screenplay, "jumpcut");
            let report = report_for_screenplay(expected.screenplay, &screenplay, &fixture);
            CorpusSnapshot::from_report(expected.screenplay, &report)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        actual, JUMPCUT_CORPUS_SNAPSHOT,
        "Jumpcut pagination changed. If the change is intentional, review the focused profile contracts and update this snapshot."
    );
}

#[test]
fn industry_pushes_three_line_giants_dialogue_whole_to_next_page() {
    let source = include_str!("fixtures/pagination/final-draft/giants-short-dialogue.fountain");
    let mut screenplay = parse(source);
    screenplay
        .metadata
        .insert("fmt".into(), vec![ElementText::Plain("industry".into())]);
    let profile = ScreenplayLayoutProfile::from_screenplay(&screenplay);
    let geometry = profile.to_pagination_geometry();
    let normalized = normalize_screenplay("giants-short-dialogue", &screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let blocks = compose_with_wrap(
        &semantic.units,
        &geometry,
        profile.pagination.interruption_dash_wrap,
    );
    let pages = paginate_with_policy(
        &blocks,
        profile.lines_per_page,
        &geometry,
        &profile.pagination,
    );

    let placements = pages
        .iter()
        .enumerate()
        .flat_map(|(page_index, page)| {
            page.blocks.iter().filter_map(move |block| {
                let SemanticUnit::Dialogue(dialogue) = block.unit else {
                    return None;
                };
                dialogue
                    .parts
                    .iter()
                    .any(|part| part.text.starts_with("And it's been sitting"))
                    .then_some((page_index, block.fragment.clone()))
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(placements, vec![(1, Fragment::Whole)]);
}

fn load_corpus_case(screenplay_id: &str, profile: &str) -> (PageBreakFixture, Screenplay) {
    let root = format!("tests/fixtures/corpus/public/{screenplay_id}");
    let fixture = serde_json::from_str(
        &fs::read_to_string(format!("{root}/canonical/page-breaks.json")).unwrap(),
    )
    .unwrap();
    let source = fs::read_to_string(format!("{root}/source/source.fountain")).unwrap();
    let mut screenplay = parse(&source);
    let existing_fmt = screenplay
        .metadata
        .get("fmt")
        .and_then(|values| values.first())
        .map(|value| value.plain_text())
        .unwrap_or_default();
    screenplay.metadata.insert(
        "fmt".into(),
        vec![ElementText::Plain(
            format!("{existing_fmt} {profile}").trim().to_string(),
        )],
    );
    (fixture, screenplay)
}

fn report_for_screenplay(
    screenplay_id: &str,
    screenplay: &Screenplay,
    fixture: &PageBreakFixture,
) -> ComparisonReport {
    let actual = PaginatedScreenplay::from_screenplay(
        screenplay_id,
        screenplay,
        54.0,
        fixture.scope.clone(),
    );
    compare_paginated_to_fixture(&actual, fixture)
}
