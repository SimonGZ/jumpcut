use std::fs;

use jumpcut::pagination::composer::compose_with_mode;
use jumpcut::pagination::paginator::paginate_with_mode;
use jumpcut::pagination::{
    Fragment, PageBreakFixture, PaginatedScreenplay, ScreenplayLayoutProfile, SemanticUnit,
    build_semantic_screenplay, compare_paginated_to_fixture, normalize_screenplay,
};
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

#[test]
fn final_draft_pushes_three_line_giants_dialogue_whole_to_next_page() {
    let source = include_str!("fixtures/pagination/final-draft/giants-short-dialogue.fountain");
    let mut screenplay = parse(source);
    screenplay
        .metadata
        .insert("fmt".into(), vec![ElementText::Plain("final-draft".into())]);
    let profile = ScreenplayLayoutProfile::from_screenplay(&screenplay);
    let geometry = profile.to_pagination_geometry();
    let normalized = normalize_screenplay("giants-short-dialogue", &screenplay);
    let semantic = build_semantic_screenplay(normalized);
    let blocks = compose_with_mode(&semantic.units, &geometry, profile.interruption_dash_wrap);
    let pages = paginate_with_mode(
        &blocks,
        profile.lines_per_page,
        &geometry,
        profile.interruption_dash_wrap,
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
