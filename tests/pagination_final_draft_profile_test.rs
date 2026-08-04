use std::fs;

use jumpcut::pagination::{PageBreakFixture, PaginatedScreenplay, compare_paginated_to_fixture};
use jumpcut::{ElementText, parse};

const PUBLIC_CORPUS: &[&str] = &[
    "big-fish",
    "big-fish-scene-numbers",
    "brick-n-steel",
    "gumshoe",
    "little-women",
    "mostly-genius",
    "vikings",
];

#[test]
fn final_draft_profile_public_corpus_snapshot() {
    for screenplay_id in PUBLIC_CORPUS {
        let root = format!("tests/fixtures/corpus/public/{screenplay_id}");
        let fixture: PageBreakFixture = serde_json::from_str(
            &fs::read_to_string(format!("{root}/canonical/page-breaks.json")).unwrap(),
        )
        .unwrap();
        let source = fs::read_to_string(format!("{root}/source/source.fountain")).unwrap();
        let baseline_screenplay = parse(&source);
        let baseline = PaginatedScreenplay::from_screenplay(
            screenplay_id,
            &baseline_screenplay,
            54.0,
            fixture.scope.clone(),
        );
        let baseline_report = compare_paginated_to_fixture(&baseline, &fixture);

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
                format!("{existing_fmt} final-draft").trim().to_string(),
            )],
        );

        let actual = PaginatedScreenplay::from_screenplay(
            screenplay_id,
            &screenplay,
            54.0,
            fixture.scope.clone(),
        );
        let report = compare_paginated_to_fixture(&actual, &fixture);

        println!(
            "{screenplay_id}: pages {}/{}, issues {} (ranked baseline {})",
            report.actual_page_count,
            report.expected_page_count,
            report.total_issues(),
            baseline_report.total_issues(),
        );
        assert_eq!(report.actual_page_count, report.expected_page_count);
        assert!(
            report.total_issues() <= baseline_report.total_issues(),
            "final-draft regressed {screenplay_id}: {} issues versus {} for ranked",
            report.total_issues(),
            baseline_report.total_issues()
        );
    }
}
