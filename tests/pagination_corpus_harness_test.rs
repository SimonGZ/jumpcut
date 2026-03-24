use jumpcut::pagination::{
    boundary_spacing_lines, build_semantic_screenplay, compare_paginated_to_fixture,
    measure_dialogue_part_lines, measure_dialogue_unit, measure_flow_unit, measure_lyric_unit,
    measure_text_lines, normalize_screenplay, ComparisonIssueKind, DialoguePartKind, FlowKind,
    MeasurementConfig, NormalizedElement, NormalizedScreenplay, PageBreakFixture,
    PageBreakFixtureSourceRefs, PaginatedScreenplay, PaginationConfig, UnitMeasurement,
};
use jumpcut::parse;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
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
fn selected_big_fish_window_probe_baselines_hold() {
    for (path, expected_lines, expected_score, expected_counts) in [
        (
            "tests/fixtures/pagination/big-fish.p38-40.page-breaks.json",
            41,
            (0, 0, 0),
            (0, 0),
        ),
        (
            "tests/fixtures/pagination/big-fish.p42-44.page-breaks.json",
            39,
            (0, 0, 0),
            (0, 0),
        ),
        (
            "tests/fixtures/pagination/big-fish.p55-57.page-breaks.json",
            50,
            (0, 0, 0),
            (0, 0),
        ),
        (
            "tests/fixtures/pagination/big-fish.p77-79.page-breaks.json",
            42,
            (0, 0, 0),
            (0, 0),
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(
            "big-fish",
            "benches/Big-Fish.fountain",
            &fixture,
        );
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(&fixture, &semantic);
        let report = &run.report;

        assert_eq!(run.lines_per_page, expected_lines, "{path}");
        assert_eq!(run.score, expected_score, "{path}");
        assert_eq!(
            report.issue_count(ComparisonIssueKind::MissingOccurrence),
            expected_counts.0,
            "{path}"
        );
        assert_eq!(
            report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            expected_counts.1,
            "{path}"
        );
        assert!(
            report
                .issues
                .iter()
                .filter(|issue| issue.kind != ComparisonIssueKind::UnexpectedOccurrence)
                .all(|issue| issue.text_preview.is_some()),
            "{path}: expected previews on non-unexpected issues"
        );
    }
}

#[test]
fn selected_public_window_probe_baselines_hold() {
    for (path, screenplay_id, fountain_path, expected_lines, expected_score, expected_counts) in [
        (
            "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
            "brick-n-steel",
            "../jumpcut-layout-corpus/corpus/public/brick-n-steel/source/source.fountain",
            38,
            (2, 2, 0),
            (0, 0),
        ),
        (
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "little-women",
            "../jumpcut-layout-corpus/corpus/public/little-women/source/source.fountain",
            51,
            (6, 3, 0),
            (1, 2),
        ),
        (
            "tests/fixtures/pagination/little-women.p13-14.page-breaks.json",
            "little-women",
            "../jumpcut-layout-corpus/corpus/public/little-women/source/source.fountain",
            40,
            (0, 0, 0),
            (0, 0),
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(&fixture, &semantic);
        let report = &run.report;

        assert_eq!(run.lines_per_page, expected_lines, "{path}");
        assert_eq!(run.score, expected_score, "{path}");
        assert_eq!(
            report.issue_count(ComparisonIssueKind::MissingOccurrence),
            expected_counts.0,
            "{path}"
        );
        assert_eq!(
            report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            expected_counts.1,
            "{path}"
        );
        assert!(
            report
                .issues
                .iter()
                .filter(|issue| issue.kind != ComparisonIssueKind::UnexpectedOccurrence)
                .all(|issue| issue.text_preview.is_some()),
            "{path}: expected previews on non-unexpected issues"
        );
    }
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

    let run = best_probe_run(&fixture, &semantic);
    let report = &run.report;

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
        report.issues.iter().all(|issue| issue.text_preview.is_some()),
        "expected all issues to carry text previews: {:?}",
        report.issues
    );
}

#[test]
#[ignore = "diagnostic corpus probe"]
fn probe_big_fish_selected_windows_against_canonical_fixtures() {
    for path in [
        "tests/fixtures/pagination/big-fish.p38-40.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p42-44.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p55-57.page-breaks.json",
        "tests/fixtures/pagination/big-fish.p77-79.page-breaks.json",
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(
            "big-fish",
            "benches/Big-Fish.fountain",
            &fixture,
        );
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(&fixture, &semantic);

        println!(
            "{}",
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: path.to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: run.report.total_issues(),
                wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: run.report.issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run.report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report,
            })
            .unwrap()
        );
    }
}

#[test]
#[ignore = "diagnostic corpus probe"]
fn probe_selected_public_windows_against_canonical_fixtures() {
    for (path, screenplay_id, fountain_path) in [
        (
            "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
            "brick-n-steel",
            "../jumpcut-layout-corpus/corpus/public/brick-n-steel/source/source.fountain",
        ),
        (
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "little-women",
            "../jumpcut-layout-corpus/corpus/public/little-women/source/source.fountain",
        ),
        (
            "tests/fixtures/pagination/little-women.p13-14.page-breaks.json",
            "little-women",
            "../jumpcut-layout-corpus/corpus/public/little-women/source/source.fountain",
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(&fixture, &semantic);

        println!(
            "{}",
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: path.to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: run.report.total_issues(),
                wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: run.report.issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run.report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report,
            })
            .unwrap()
        );
    }
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
    let run = best_probe_run(&fixture, &semantic);
    println!(
        "{}",
        serde_json::to_string_pretty(&ProbeDebugOutput {
            lines_per_page: run.lines_per_page,
            score: run.score,
            total_issues: run.report.total_issues(),
            wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: run.report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run.report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: run.report,
        })
        .unwrap()
    );
}

#[test]
#[ignore = "writes current paginated output json for manual comparison"]
fn dump_big_fish_public_slice_paginated_output_json() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized = normalized_slice_from_fountain(
        "big-fish",
        "benches/Big-Fish.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized.clone());
    let run = best_probe_run(&fixture, &semantic);
    let previews = preview_map(&normalized);

    let debug_fixture = paginated_to_debug_fixture(
        &run.actual,
        &fixture.source,
        &normalized,
        run.lines_per_page,
        &run.measurement,
        &previews,
    );
    let debug_dir = Path::new("target/pagination-debug");
    fs::create_dir_all(debug_dir).unwrap();

    let actual_path = debug_dir.join("big-fish.actual.page-breaks.json");
    fs::write(
        &actual_path,
        serde_json::to_string_pretty(&debug_fixture).unwrap(),
    )
    .unwrap();

    let report_path = debug_dir.join("big-fish.comparison-report.json");
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&ProbeDebugOutput {
            lines_per_page: run.lines_per_page,
            score: run.score,
            total_issues: run.report.total_issues(),
            wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: run.report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run.report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: run.report,
        })
        .unwrap(),
    )
    .unwrap();

    println!("wrote {}", actual_path.display());
    println!("wrote {}", report_path.display());
}

fn best_probe_run(
    fixture: &PageBreakFixture,
    semantic: &jumpcut::pagination::SemanticScreenplay,
) -> ProbeRun {
    let mut best = None;
    let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
    for lines_per_page in 1..=60 {
        let config = PaginationConfig::screenplay(lines_per_page);
        let full_actual = PaginatedScreenplay::paginate(
            semantic.clone(),
            config.clone(),
            fixture.style_profile.clone(),
            fixture.scope.clone(),
        );
        let actual = paginated_page_window(&full_actual, &page_numbers);
        let report = compare_paginated_to_fixture(&actual, fixture);
        let score = (
            report.total_issues(),
            report.issue_count(ComparisonIssueKind::WrongPage),
            report.issue_count(ComparisonIssueKind::WrongFragment),
        );

        match &best {
            Some((best_score, _, _)) if best_score <= &score => {}
            _ => {
                best = Some((
                    score,
                    lines_per_page,
                    ProbeRun {
                        lines_per_page,
                        score,
                        actual,
                        measurement: config.measurement,
                        report,
                    },
                ))
            }
        }
    }

    best.unwrap().2
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

fn paginated_page_window(
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

fn read_fixture<T: DeserializeOwned>(path: &str) -> T {
    let content = fs::read_to_string(Path::new(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn preview_map(normalized: &NormalizedScreenplay) -> HashMap<String, String> {
    normalized
        .elements
        .iter()
        .map(|element| {
            (
                element.element_id.clone(),
                text_preview(&element.text),
            )
        })
        .collect()
}

fn paginated_to_debug_fixture(
    actual: &PaginatedScreenplay,
    source: &PageBreakFixtureSourceRefs,
    normalized: &NormalizedScreenplay,
    lines_per_page: u32,
    measurement: &MeasurementConfig,
    previews: &HashMap<String, String>,
) -> DebugPageBreakFixture {
    let elements = normalized_element_map(normalized);

    DebugPageBreakFixture {
        screenplay: actual.screenplay.clone(),
        style_profile: actual.style_profile.clone(),
        source: source.clone(),
        scope: actual.scope.clone(),
        lines_per_page,
        measurement: DebugMeasurement {
            action_width_chars: measurement.width_chars_for_flow_kind(&FlowKind::Action),
            dialogue_width_chars: measurement.width_chars_for_dialogue_part(&DialoguePartKind::Dialogue),
            character_width_chars: measurement.width_chars_for_dialogue_part(&DialoguePartKind::Character),
            parenthetical_width_chars: measurement.width_chars_for_dialogue_part(&DialoguePartKind::Parenthetical),
            action_top_spacing_lines: measurement.action_top_spacing_lines,
            action_bottom_spacing_lines: measurement.action_bottom_spacing_lines,
            scene_heading_top_spacing_lines: measurement.scene_heading_top_spacing_lines,
            scene_heading_bottom_spacing_lines: measurement.scene_heading_bottom_spacing_lines,
            transition_top_spacing_lines: measurement.transition_top_spacing_lines,
            transition_bottom_spacing_lines: measurement.transition_bottom_spacing_lines,
            dialogue_top_spacing_lines: measurement.dialogue_top_spacing_lines,
            dialogue_bottom_spacing_lines: measurement.dialogue_bottom_spacing_lines,
            lyric_top_spacing_lines: measurement.lyric_top_spacing_lines,
            lyric_bottom_spacing_lines: measurement.lyric_bottom_spacing_lines,
        },
        pages: actual
            .pages
            .iter()
            .map(|page| debug_page(page, &elements, measurement, previews))
            .collect(),
    }
}

fn debug_page(
    page: &jumpcut::pagination::Page,
    elements: &HashMap<String, NormalizedElement>,
    measurement: &MeasurementConfig,
    previews: &HashMap<String, String>,
) -> DebugPageBreakFixturePage {
    let mut measured_total_lines = 0;
    let mut previous_unit_measurement: Option<UnitMeasurement> = None;
    let mut previous_unit_key: Option<String> = None;
    let mut items = Vec::with_capacity(page.items.len());

    for item in &page.items {
        let unit_key = debug_unit_key(item);
        let unit_measurement = measured_unit_for_item(item, elements, measurement);
        let is_first_in_unit = previous_unit_key.as_ref() != Some(&unit_key);
        let spacing_before_lines = if is_first_in_unit {
            boundary_spacing_lines(previous_unit_measurement.as_ref(), Some(&unit_measurement))
        } else {
            0
        };
        let (intrinsic_top_spacing_lines, intrinsic_bottom_spacing_lines) = if is_first_in_unit {
            (
                unit_measurement.top_spacing_lines,
                unit_measurement.bottom_spacing_lines,
            )
        } else {
            (0, 0)
        };
        let measured_lines = measured_lines_for_item(item, elements, measurement);
        measured_total_lines += measured_lines + spacing_before_lines;

        items.push(DebugPageBreakItem {
            element_id: item.element_id.clone(),
            kind: item.kind.clone(),
            text_preview: previews.get(&item.element_id).cloned(),
            measured_lines,
            spacing_before_lines,
            intrinsic_top_spacing_lines,
            intrinsic_bottom_spacing_lines,
            fragment: item.fragment.clone(),
            line_range: item.line_range,
            block_id: item.block_id.clone(),
            dual_dialogue_group: item.dual_dialogue_group.clone(),
            dual_dialogue_side: item.dual_dialogue_side,
        });

        if is_first_in_unit {
            previous_unit_measurement = Some(unit_measurement);
            previous_unit_key = Some(unit_key);
        }
    }

    DebugPageBreakFixturePage {
        number: page.metadata.number,
        measured_total_lines,
        items,
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

fn normalized_element_map(
    normalized: &NormalizedScreenplay,
) -> HashMap<String, NormalizedElement> {
    normalized
        .elements
        .iter()
        .cloned()
        .map(|element| (element.element_id.clone(), element))
        .collect()
}

fn measured_lines_for_item(
    item: &jumpcut::pagination::PageItem,
    elements: &HashMap<String, NormalizedElement>,
    measurement: &MeasurementConfig,
) -> u32 {
    let Some(element) = elements.get(&item.element_id) else {
        return 0;
    };

    match item.kind.as_str() {
        "Character" => measure_dialogue_part_lines(
            &DialoguePartKind::Character,
            &element.text,
            measurement,
        ),
        "Parenthetical" => measure_dialogue_part_lines(
            &DialoguePartKind::Parenthetical,
            &element.text,
            measurement,
        ),
        "Dialogue" => measure_dialogue_part_lines(
            &DialoguePartKind::Dialogue,
            &element.text,
            measurement,
        ),
        "Lyric" => measure_dialogue_part_lines(
            &DialoguePartKind::Lyric,
            &element.text,
            measurement,
        ),
        other => {
            let text = match item.line_range {
                Some((start, end)) => slice_explicit_lines(&element.text, start, end),
                None => element.text.clone(),
            };
            measure_text_lines(&text, flow_width_for_kind(other, measurement))
        }
    }
}

fn measured_unit_for_item(
    item: &jumpcut::pagination::PageItem,
    elements: &HashMap<String, NormalizedElement>,
    measurement: &MeasurementConfig,
) -> UnitMeasurement {
    let Some(element) = elements.get(&item.element_id) else {
        return UnitMeasurement {
            content_lines: 0,
            top_spacing_lines: 0,
            bottom_spacing_lines: 0,
        };
    };

    match item.kind.as_str() {
        "Character" | "Parenthetical" | "Dialogue" => measure_dialogue_unit(
            &jumpcut::pagination::DialogueUnit {
                block_id: item.block_id.clone().unwrap_or_else(|| item.element_id.clone()),
                parts: vec![jumpcut::pagination::DialoguePart {
                    element_id: item.element_id.clone(),
                    kind: match item.kind.as_str() {
                        "Character" => DialoguePartKind::Character,
                        "Parenthetical" => DialoguePartKind::Parenthetical,
                        _ => DialoguePartKind::Dialogue,
                    },
                    text: element.text.clone(),
                }],
                cohesion: jumpcut::pagination::Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                },
            },
            measurement,
        ),
        "Lyric" if item.block_id.is_some() => measure_dialogue_unit(
            &jumpcut::pagination::DialogueUnit {
                block_id: item.block_id.clone().unwrap_or_else(|| item.element_id.clone()),
                parts: vec![jumpcut::pagination::DialoguePart {
                    element_id: item.element_id.clone(),
                    kind: DialoguePartKind::Lyric,
                    text: element.text.clone(),
                }],
                cohesion: jumpcut::pagination::Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                },
            },
            measurement,
        ),
        "Lyric" => measure_lyric_unit(
            &jumpcut::pagination::LyricUnit {
                element_id: item.element_id.clone(),
                text: element.text.clone(),
                cohesion: jumpcut::pagination::Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                },
            },
            measurement,
        ),
        other => measure_flow_unit(
            &jumpcut::pagination::FlowUnit {
                element_id: item.element_id.clone(),
                kind: flow_width_kind(other),
                text: match item.line_range {
                    Some((start, end)) => slice_explicit_lines(&element.text, start, end),
                    None => element.text.clone(),
                },
                line_range: item.line_range,
                scene_number: element.scene_number.clone(),
                cohesion: jumpcut::pagination::Cohesion {
                    keep_together: false,
                    keep_with_next: false,
                    can_split: true,
                },
            },
            measurement,
        ),
    }
}

fn debug_unit_key(item: &jumpcut::pagination::PageItem) -> String {
    match (&item.block_id, &item.dual_dialogue_group, item.dual_dialogue_side) {
        (Some(block_id), Some(group_id), Some(side)) => {
            format!("dual:{group_id}:{side}:{block_id}")
        }
        (Some(block_id), _, _) => format!("block:{block_id}"),
        _ => format!("element:{}", item.element_id),
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

fn flow_width_for_kind(kind: &str, measurement: &MeasurementConfig) -> usize {
    let flow_kind = flow_width_kind(kind);
    measurement.width_chars_for_flow_kind(&flow_kind)
}

fn flow_width_kind(kind: &str) -> FlowKind {
    match kind {
        "Scene Heading" => FlowKind::SceneHeading,
        "Transition" => FlowKind::Transition,
        "Section" => FlowKind::Section,
        "Synopsis" => FlowKind::Synopsis,
        "Cold Opening" => FlowKind::ColdOpening,
        "New Act" => FlowKind::NewAct,
        "End of Act" => FlowKind::EndOfAct,
        _ => FlowKind::Action,
    }
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

#[derive(Serialize)]
struct FixtureProbeDebugOutput {
    fixture_path: String,
    page_numbers: Vec<u32>,
    lines_per_page: u32,
    score: (usize, usize, usize),
    total_issues: usize,
    wrong_page: usize,
    wrong_fragment: usize,
    missing: usize,
    unexpected: usize,
    report: jumpcut::pagination::ComparisonReport,
}

struct ProbeRun {
    lines_per_page: u32,
    score: (usize, usize, usize),
    actual: PaginatedScreenplay,
    measurement: MeasurementConfig,
    report: jumpcut::pagination::ComparisonReport,
}

#[derive(Serialize)]
struct DebugPageBreakFixture {
    screenplay: String,
    style_profile: String,
    source: PageBreakFixtureSourceRefs,
    scope: jumpcut::pagination::PaginationScope,
    lines_per_page: u32,
    measurement: DebugMeasurement,
    pages: Vec<DebugPageBreakFixturePage>,
}

#[derive(Serialize)]
struct DebugPageBreakFixturePage {
    number: u32,
    measured_total_lines: u32,
    items: Vec<DebugPageBreakItem>,
}

#[derive(Serialize)]
struct DebugPageBreakItem {
    element_id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    measured_lines: u32,
    spacing_before_lines: u32,
    intrinsic_top_spacing_lines: u32,
    intrinsic_bottom_spacing_lines: u32,
    fragment: jumpcut::pagination::Fragment,
    line_range: Option<(u32, u32)>,
    block_id: Option<String>,
    dual_dialogue_group: Option<String>,
    dual_dialogue_side: Option<u8>,
}

#[derive(Serialize)]
struct DebugMeasurement {
    action_width_chars: usize,
    dialogue_width_chars: usize,
    character_width_chars: usize,
    parenthetical_width_chars: usize,
    action_top_spacing_lines: u32,
    action_bottom_spacing_lines: u32,
    scene_heading_top_spacing_lines: u32,
    scene_heading_bottom_spacing_lines: u32,
    transition_top_spacing_lines: u32,
    transition_bottom_spacing_lines: u32,
    dialogue_top_spacing_lines: u32,
    dialogue_bottom_spacing_lines: u32,
    lyric_top_spacing_lines: u32,
    lyric_bottom_spacing_lines: u32,
}
