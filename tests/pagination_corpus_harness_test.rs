use jumpcut::pagination::{
    build_semantic_screenplay, compare_paginated_to_fixture,
    normalize_screenplay, wrapping::ElementType, ComparisonIssueKind,
    DialoguePartKind, FdxExtractedSettings, FlowKind, Fragment, LayoutGeometry, LineRange,
    NormalizedElement, NormalizedScreenplay, PageBreakFixture, PageBreakFixtureSourceRefs,
    PaginatedScreenplay, PaginationConfig,
};
use jumpcut::parse;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
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
        normalized_slice_from_fountain(
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
        normalized_slice_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
    let semantic = build_semantic_screenplay(normalized);

    let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("big-fish"));
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
        report
            .issues
            .iter()
            .all(|issue| issue.text_preview.is_some()),
        "expected all issues to carry text previews: {:?}",
        report.issues
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
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
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("big-fish"));

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
                missing: run
                    .report
                    .issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run
                    .report
                    .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report,
            })
            .unwrap()
        );
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "writes a single Big Fish review packet for human inspection"]
fn build_big_fish_review_packet() {
    let debug_dir = Path::new("target/pagination-debug/big-fish-review");
    fs::create_dir_all(debug_dir).unwrap();

    let mut summaries = Vec::new();

    for (path, stem) in [
        (
            "tests/fixtures/pagination/big-fish.p38-40.page-breaks.json",
            "p38-40",
        ),
        (
            "tests/fixtures/pagination/big-fish.p42-44.page-breaks.json",
            "p42-44",
        ),
        (
            "tests/fixtures/pagination/big-fish.p55-57.page-breaks.json",
            "p55-57",
        ),
        (
            "tests/fixtures/pagination/big-fish.p77-79.page-breaks.json",
            "p77-79",
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
        let semantic = build_semantic_screenplay(normalized.clone());
        let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("big-fish"));
        let previews = preview_map(&normalized);
        let debug_fixture = paginated_to_debug_fixture(
            &run.actual,
            &fixture.source,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            &previews,
        );
        let pdf_line_counts = canonical_pdf_line_count_debug("big-fish", &fixture, &normalized);

        let actual_path = debug_dir.join(format!("{stem}.actual.page-breaks.json"));
        let comparison_path = debug_dir.join(format!("{stem}.comparison-report.json"));
        let pdf_path = debug_dir.join(format!("{stem}.pdf-line-counts.json"));
        fs::write(
            &actual_path,
            serde_json::to_string_pretty(&debug_fixture).unwrap(),
        )
        .unwrap();
        fs::write(
            &comparison_path,
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: path.to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: run.report.total_issues(),
                wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: run
                    .report
                    .issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run
                    .report
                    .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &pdf_path,
            serde_json::to_string_pretty(&pdf_line_counts).unwrap(),
        )
        .unwrap();

        summaries.push(BigFishReviewSummary {
            stem: stem.into(),
            fixture_path: path.into(),
            page_range: fixture
                .pages
                .iter()
                .map(|page| page.number)
                .collect::<Vec<_>>(),
            lines_per_page: run.lines_per_page,
            total_issues: run.report.total_issues(),
            wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: run
                .report
                .issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run
                .report
                .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
        });
    }

    let review = render_big_fish_review_packet(&summaries);
    let review_path = debug_dir.join("REVIEW.md");
    fs::write(&review_path, review).unwrap();

    println!("wrote {}", review_path.display());
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
fn brick_n_steel_full_script_page_break_parity_holds_baseline() {
    let fixture: PageBreakFixture = read_fixture(
        "tests/fixtures/corpus/public/brick-n-steel/canonical/page-breaks.json",
    );
    let normalized = normalized_slice_from_fountain(
        "brick-n-steel",
        "tests/fixtures/corpus/public/brick-n-steel/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized);
    let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("brick-n-steel"));

    assert_eq!(
        run.report.total_issues(),
        0,
        "Expected Brick & Steel full-script page-break parity against the Final Draft canonical fixture to have 0 issues. If this fails, inspect the report and decide which pagination assumptions are wrong."
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "writes a script-wide Big Fish line-break parity packet"]
fn build_big_fish_line_break_parity_packet() {
    let report = build_line_break_parity_report(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
    );
    let debug_dir = Path::new("target/pagination-debug/big-fish-linebreak-parity");
    fs::create_dir_all(debug_dir).unwrap();

    let json_path = debug_dir.join("parity.json");
    fs::write(&json_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();

    let review_path = debug_dir.join("REVIEW.md");
    fs::write(&review_path, render_line_break_parity_review(&report)).unwrap();

    println!("wrote {}", review_path.display());
    println!("wrote {}", json_path.display());
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "diagnostic corpus probe"]
fn probe_selected_public_windows_against_canonical_fixtures() {
    for (path, screenplay_id, fountain_path) in [
        (
            "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
            "brick-n-steel",
            "tests/fixtures/corpus/public/brick-n-steel/source/source.fountain",
        ),
        (
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "little-women",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
        ),
        (
            "tests/fixtures/pagination/little-women.p13-14.page-breaks.json",
            "little-women",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized);
        let run = best_probe_run(
            &fixture,
            &semantic,
            measurement_for_screenplay(screenplay_id),
        );

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
                missing: run
                    .report
                    .issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run
                    .report
                    .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report,
            })
            .unwrap()
        );
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "diagnostic corpus probe"]
fn probe_big_fish_public_slice_against_canonical_fixture() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized =
        normalized_slice_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
    let semantic = build_semantic_screenplay(normalized);
    let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("big-fish"));
    println!(
        "{}",
        serde_json::to_string_pretty(&ProbeDebugOutput {
            lines_per_page: run.lines_per_page,
            score: run.score,
            total_issues: run.report.total_issues(),
            wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: run
                .report
                .issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run
                .report
                .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: run.report,
        })
        .unwrap()
    );
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "writes current paginated output json for manual comparison"]
fn dump_big_fish_public_slice_paginated_output_json() {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized =
        normalized_slice_from_fountain(
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            &fixture,
        );
    let semantic = build_semantic_screenplay(normalized.clone());
    let run = best_probe_run(&fixture, &semantic, measurement_for_screenplay("big-fish"));
    let previews = preview_map(&normalized);

    let debug_fixture = paginated_to_debug_fixture(
        &run.actual,
        &fixture.source,
        &normalized,
        run.lines_per_page,
        &run.geometry,
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
            missing: run
                .report
                .issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run
                .report
                .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: run.report,
        })
        .unwrap(),
    )
    .unwrap();

    println!("wrote {}", actual_path.display());
    println!("wrote {}", report_path.display());
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "writes current paginated output json for selected public windows"]
fn dump_selected_public_windows_paginated_output_json() {
    for (path, screenplay_id, fountain_path, stem) in [
        (
            "tests/fixtures/pagination/brick-n-steel.p2-4.page-breaks.json",
            "brick-n-steel",
            "tests/fixtures/corpus/public/brick-n-steel/source/source.fountain",
            "brick-n-steel.p2-4",
        ),
        (
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "little-women",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
            "little-women.p4-6",
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(path);
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized.clone());
        let run = best_probe_run(
            &fixture,
            &semantic,
            measurement_for_screenplay(screenplay_id),
        );
        let previews = preview_map(&normalized);
        let debug_fixture = paginated_to_debug_fixture(
            &run.actual,
            &fixture.source,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            &previews,
        );

        let debug_dir = Path::new("target/pagination-debug");
        fs::create_dir_all(debug_dir).unwrap();

        let actual_path = debug_dir.join(format!("{stem}.actual.page-breaks.json"));
        fs::write(
            &actual_path,
            serde_json::to_string_pretty(&debug_fixture).unwrap(),
        )
        .unwrap();

        let report_path = debug_dir.join(format!("{stem}.comparison-report.json"));
        fs::write(
            &report_path,
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: path.to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: run.report.total_issues(),
                wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: run
                    .report
                    .issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: run
                    .report
                    .issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: run.report,
            })
            .unwrap(),
        )
        .unwrap();

        let pdf_line_counts = canonical_pdf_line_count_debug(screenplay_id, &fixture, &normalized);
        let pdf_line_count_path = debug_dir.join(format!("{stem}.pdf-line-counts.json"));
        fs::write(
            &pdf_line_count_path,
            serde_json::to_string_pretty(&pdf_line_counts).unwrap(),
        )
        .unwrap();

        println!("wrote {}", actual_path.display());
        println!("wrote {}", report_path.display());
        println!("wrote {}", pdf_line_count_path.display());
    }
}

#[test]
// #[ignore = "Temporarily disabled"]
#[ignore = "exports rich JSON for the visual pagination comparator tool"]
fn export_visual_comparison_data() {
    let debug_dir = Path::new("target/pagination-debug/visual");
    fs::create_dir_all(debug_dir).unwrap();

    for (screenplay_id, fixture_path, fountain_path, label, fixed_lpp) in [
        (
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            "big-fish",
            Some(54u32), // Proven by window probe tests
        ),
        (
            "little-women",
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
            "little-women-p4-6",
            None, // Use probe sweep
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(fixture_path);
        let normalized = normalized_slice_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized.clone());
        let measurement = measurement_for_screenplay(screenplay_id);

        let run = if let Some(lpp) = fixed_lpp {
            // For full-script exports, use a known-good lines_per_page directly.
            // The best_probe_run sweep (1-60) doesn't converge well over 100+ pages.
            let config = PaginationConfig {
                lines_per_page: lpp as f32,
                geometry: measurement.clone(),
            };
            let actual = PaginatedScreenplay::paginate(
                semantic.clone(),
                config.clone(),
                fixture.style_profile.clone(),
                fixture.scope.clone(),
            );
            let report = compare_paginated_to_fixture(&actual, &fixture);
            let score = (
                report.total_issues(),
                report.issue_count(ComparisonIssueKind::WrongPage),
                report.issue_count(ComparisonIssueKind::WrongFragment),
            );
            ProbeRun {
                lines_per_page: lpp as f32,
                score,
                actual,
                geometry: measurement,
                report,
            }
        } else {
            best_probe_run(&fixture, &semantic, measurement)
        };

        let elements = normalized_element_map(&normalized);

        // Build a lookup from (element_id, occurrence) → (actual_page, actual_fragment)
        let mut actual_lookup: HashMap<(String, usize), (u32, Fragment)> = HashMap::new();
        let mut actual_counters: HashMap<String, usize> = HashMap::new();
        for page in &run.actual.pages {
            for item in &page.items {
                let occ = actual_counters
                    .entry(item.element_id.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                actual_lookup.insert(
                    (item.element_id.clone(), *occ),
                    (page.metadata.number, item.fragment.clone()),
                );
            }
        }

        // Build pages from the canonical fixture, enriched with actual data
        let mut expected_counters: HashMap<String, usize> = HashMap::new();
        let mut visual_pages = Vec::new();
        for fixture_page in &fixture.pages {
            let mut page_items = Vec::new();
            let mut page_measured_lines: u32 = 0;

            for item in &fixture_page.items {
                let occ = expected_counters
                    .entry(item.element_id.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);

                let element = elements.get(&item.element_id);
                let full_text = element.map(|e| e.text.clone()).unwrap_or_default();

                let _width_chars = width_chars_for_parity_kind(
                    &item.kind,
                    item.block_id.is_some(),
                    &run.geometry,
                );

                let line_range_tuple = item.line_range.map(|lr| (lr.0, lr.1));

                // Compute the text to wrap (respecting line_range for flow splits)
                let wrap_text = match (line_range_tuple, element) {
                    (Some((s, e)), Some(_)) => slice_explicit_lines(&full_text, s, e),
                    _ => full_text.clone(),
                };
                let element_type = ElementType::from_flow_kind(&flow_width_kind(&item.kind));
                let config = jumpcut::pagination::wrapping::WrapConfig::from_geometry(&run.geometry, element_type);
                let wrapped_lines = jumpcut::pagination::wrapping::wrap_text_for_element(&wrap_text, &config);
                let width_chars = config.exact_width_chars;

                // Build a temp PageItem for measurement helpers
                let temp_page_item = jumpcut::pagination::PageItem {
                    element_id: item.element_id.clone(),
                    kind: item.kind.clone(),
                    fragment: item.fragment.clone(),
                    line_range: line_range_tuple,
                    block_id: item.block_id.clone(),
                    dual_dialogue_group: item.dual_dialogue_group.clone(),
                    dual_dialogue_side: item.dual_dialogue_side,
                    continuation_markers: Vec::new(),
                };

                let items_measured = measured_lines_for_item(&temp_page_item, &elements, &run.geometry);
                let content_lines = items_measured.content_lines;
                let spacing_before = items_measured.spacing_above;

                page_measured_lines += (content_lines + spacing_before) as u32;

                // Lookup actual placement
                let actual = actual_lookup.get(&(item.element_id.clone(), *occ));
                let actual_page = actual.map(|(p, _)| *p);
                let actual_fragment = actual.map(|(_, f)| f.clone());

                let status = match actual {
                    Some((ap, af)) => {
                        if *ap != fixture_page.number {
                            "wrong_page"
                        } else if *af != item.fragment {
                            "wrong_fragment"
                        } else {
                            "match"
                        }
                    }
                    None => "missing",
                };

                page_items.push(VisualComparisonItem {
                    element_id: item.element_id.clone(),
                    kind: item.kind.clone(),
                    full_text: full_text.clone(),
                    block_id: item.block_id.clone(),
                    canonical_page: fixture_page.number,
                    canonical_fragment: format!("{:?}", item.fragment),
                    actual_page,
                    actual_fragment: actual_fragment.map(|f| format!("{:?}", f)),
                    width_chars,
                    wrapped_lines,
                content_lines,
                top_spacing_lines: 0.0,
                bottom_spacing_lines: 0.0,
                spacing_before_lines: spacing_before,
                status: status.into(),
                line_range: line_range_tuple,
            });

            if true {
                // previous_unit_measurement = Some(unit_measurement);
                // previous_unit_key = Some(unit_key);
            }
        }

        visual_pages.push(VisualComparisonPage {
            page_number: fixture_page.number,
            lines_per_page: run.lines_per_page,
            measured_total_lines: page_measured_lines,
            item_count: page_items.len(),
            items: page_items,
        });
    }

        // Build measurement geometry summary
        let geom = VisualMeasurementSummary {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, &run.geometry),
                debug_flow_geometry(
                    "Scene Heading",
                    "Scene Heading",
                    FlowKind::SceneHeading,
                    &run.geometry,
                ),
                debug_flow_geometry(
                    "Transition",
                    "Transition",
                    FlowKind::Transition,
                    &run.geometry,
                ),
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry(
                    "Character",
                    "Character",
                    DialoguePartKind::Character,
                    &run.geometry,
                ),
                debug_dialogue_geometry(
                    "Dialogue",
                    "Dialogue",
                    DialoguePartKind::Dialogue,
                    &run.geometry,
                ),
                debug_dialogue_geometry(
                    "Parenthetical",
                    "Parenthetical",
                    DialoguePartKind::Parenthetical,
                    &run.geometry,
                ),
                debug_dialogue_geometry("Lyric", "Lyric", DialoguePartKind::Lyric, &run.geometry),
            ],
        };

        let total_elements: usize = visual_pages.iter().map(|p| p.items.len()).sum();
        let total_matches: usize = visual_pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|i| i.status == "match")
            .count();
        let total_wrong_page: usize = visual_pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|i| i.status == "wrong_page")
            .count();
        let total_wrong_fragment: usize = visual_pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|i| i.status == "wrong_fragment")
            .count();
        let total_missing: usize = visual_pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|i| i.status == "missing")
            .count();

        let output = VisualComparisonOutput {
            screenplay: screenplay_id.into(),
            label: label.into(),
            lines_per_page: run.lines_per_page,
            score: run.score,
            total_elements,
            total_matches,
            total_wrong_page,
            total_wrong_fragment,
            total_missing,
            measurement: geom,
            pages: visual_pages,
        };

        let out_path = debug_dir.join(format!("{label}.comparison.json"));
        fs::write(&out_path, serde_json::to_string_pretty(&output).unwrap()).unwrap();
        println!("wrote {}", out_path.display());
    }
}

fn best_probe_run(
    fixture: &PageBreakFixture,
    semantic: &jumpcut::pagination::SemanticScreenplay,
    geometry: LayoutGeometry,
) -> ProbeRun {
    let mut best = None;
    let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
    for lpp_int in 54..=54 {
        let lines_per_page = lpp_int as f32;
        let config = PaginationConfig {
            lines_per_page,
            geometry: geometry.clone(),
        };
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
                        geometry: geometry.clone(),
                        report,
                    },
                ))
            }
        }
    }

    best.unwrap().2
}

fn measurement_for_screenplay(screenplay_id: &str) -> LayoutGeometry {
    let path = Path::new("tests/fixtures/corpus/public")
        .join(screenplay_id)
        .join("extracted/fdx-settings.json");
    let settings: FdxExtractedSettings =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    LayoutGeometry::from_fdx_settings(&settings)
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
                element.element_id.as_str() >= *first_id && element.element_id.as_str() <= *last_id
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
        .map(|element| (element.element_id.clone(), text_preview(&element.text)))
        .collect()
}

fn paginated_to_debug_fixture(
    actual: &PaginatedScreenplay,
    source: &PageBreakFixtureSourceRefs,
    normalized: &NormalizedScreenplay,
    lines_per_page: f32,
    geometry: &LayoutGeometry,
    previews: &HashMap<String, String>,
) -> DebugPageBreakFixture {
    let elements = normalized_element_map(normalized);

    DebugPageBreakFixture {
        screenplay: actual.screenplay.clone(),
        style_profile: actual.style_profile.clone(),
        source: source.clone(),
        scope: actual.scope.clone(),
        lines_per_page: lines_per_page,
        measurement: DebugMeasurement {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, geometry),
                debug_flow_geometry(
                    "Scene Heading",
                    "Scene Heading",
                    FlowKind::SceneHeading,
                    geometry,
                ),
                debug_flow_geometry(
                    "Transition",
                    "Transition",
                    FlowKind::Transition,
                    geometry,
                ),
                // Other flow kinds use Action fallback in this debug view
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry(
                    "Dialogue",
                    "Dialogue",
                    DialoguePartKind::Dialogue,
                    geometry,
                ),
                debug_dialogue_geometry(
                    "Character",
                    "Character",
                    DialoguePartKind::Character,
                    geometry,
                ),
                debug_dialogue_geometry(
                    "Parenthetical",
                    "Parenthetical",
                    DialoguePartKind::Parenthetical,
                    geometry,
                ),
                debug_dialogue_geometry("Lyric", "Lyric", DialoguePartKind::Lyric, geometry),
            ],
            action_spacing_before: geometry.action_spacing_before,
            scene_heading_spacing_before: geometry.scene_heading_spacing_before,
            character_spacing_before: geometry.character_spacing_before,
            transition_spacing_before: geometry.transition_spacing_before,
            lyric_spacing_before: geometry.lyric_spacing_before,
        },
        pages: actual
            .pages
            .iter()
            .map(|page| debug_page(page, &elements, geometry, geometry, previews))
            .collect(),
    }
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

fn debug_page(
    page: &jumpcut::pagination::Page,
    elements: &HashMap<String, NormalizedElement>,
    _measurement: &LayoutGeometry, // Renamed to _measurement as it's not used directly
    geometry: &LayoutGeometry,
    previews: &HashMap<String, String>,
) -> DebugPageBreakFixturePage {
    let mut measured_total_lines = 0;
    let mut items = Vec::with_capacity(page.items.len());
    for item in &page.items {
        let items_measured = measured_lines_for_item(item, elements, geometry);
        let measured_lines = items_measured.content_lines;
        let spacing_before_lines = items_measured.spacing_above;

        items.push(DebugPageBreakItem {
            element_id: item.element_id.clone(),
            kind: item.kind.clone(),
            text_preview: previews.get(&item.element_id).cloned(),
            measured_lines,
            spacing_before_lines,
            intrinsic_top_spacing_lines: 0.0, // Deprecated in favor of block spacing
            intrinsic_bottom_spacing_lines: 0.0,
            fragment: item.fragment.clone(),
            line_range: item.line_range,
            block_id: item.block_id.clone(),
            dual_dialogue_group: item.dual_dialogue_group.clone(),
            dual_dialogue_side: item.dual_dialogue_side,
        });

        measured_total_lines += (measured_lines + spacing_before_lines) as u32; // Rough approximation for debug JSON
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
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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

fn render_big_fish_review_packet(summaries: &[BigFishReviewSummary]) -> String {
    let focus_windows: Vec<&BigFishReviewSummary> = summaries
        .iter()
        .filter(|summary| summary.total_issues > 0)
        .collect();
    let ordered_windows: Vec<&BigFishReviewSummary> = focus_windows
        .iter()
        .copied()
        .chain(summaries.iter().filter(|summary| summary.total_issues == 0))
        .collect();

    let mut review = String::from(
        "# Big Fish Pagination Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo test --test pagination_corpus_harness_test build_big_fish_review_packet -- --ignored --nocapture\n\
```\n\n\
Read files in this order:\n\n\
1. `target/pagination-debug/big-fish-review/REVIEW.md`\n",
    );

    for (index, summary) in ordered_windows.iter().enumerate() {
        let step = index * 3 + 2;
        review.push_str(&format!(
            "{step}. `target/pagination-debug/big-fish-review/{stem}.comparison-report.json`\n\
{step_plus_one}. `{fixture}`\n\
{step_plus_two}. `target/pagination-debug/big-fish-review/{stem}.actual.page-breaks.json`\n",
            step = step,
            step_plus_one = step + 1,
            step_plus_two = step + 2,
            stem = summary.stem,
            fixture = summary.fixture_path,
        ));
    }

    review.push_str(
        "\nPrimary review question:\n\n\
- For the first nonzero window above, does the canonical break look like missing spacing/rhythm in our model, an obvious line-wrap-count problem, or a split-choice problem?\n\n\
Useful extra files:\n\n\
- `target/pagination-debug/big-fish-review/<window>.pdf-line-counts.json` gives exact-unique PDF line counts where text alignment is recoverable.\n\
- `tests/fixtures/corpus/public/big-fish/source/source.fountain` is the vendored local source text.\n\n\
Current window summary:\n\n",
    );

    for summary in summaries {
        review.push_str(&format!(
            "- `{stem}` pages {start}-{end}: lines_per_page={lines}, total={total}, wrong_page={wrong_page}, wrong_fragment={wrong_fragment}, missing={missing}, unexpected={unexpected}\n  canonical: `{fixture}`\n  actual: `target/pagination-debug/big-fish-review/{stem}.actual.page-breaks.json`\n  report: `target/pagination-debug/big-fish-review/{stem}.comparison-report.json`\n  pdf: `target/pagination-debug/big-fish-review/{stem}.pdf-line-counts.json`\n",
            stem = summary.stem,
            start = summary.page_range.first().copied().unwrap_or_default(),
            end = summary.page_range.last().copied().unwrap_or_default(),
            lines = summary.lines_per_page,
            total = summary.total_issues,
            wrong_page = summary.wrong_page,
            wrong_fragment = summary.wrong_fragment,
            missing = summary.missing,
            unexpected = summary.unexpected,
            fixture = summary.fixture_path,
        ));
    }

    if let Some(summary) = focus_windows.first() {
        review.push_str(&format!(
            "\nIf you only look at one thing, start with `{stem}.comparison-report.json`.\n",
            stem = summary.stem,
        ));
    } else {
        review.push_str("\nAll selected Big Fish windows currently match.\n");
    }

    review
}

fn build_line_break_parity_report(
    screenplay_id: &str,
    fountain_path: &str,
    canonical_page_breaks_path: &str,
) -> LineBreakParityReport {
    let measurement = measurement_for_screenplay(screenplay_id);
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
                &item.block_id,
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
    block_id: &Option<String>,
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

    let width_chars = width_chars_for_parity_kind(kind, block_id.is_some(), measurement);
    let expected_wrapped_lines = wrap_lines_for_parity_kind(kind, &candidate_text, width_chars)
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

fn width_chars_for_parity_kind(
    kind: &str,
    is_in_block: bool,
    geometry: &LayoutGeometry,
) -> usize {
    match kind {
        "Character" => jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::Character),
        "Parenthetical" => {
            jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::Parenthetical)
        }
        "Dialogue" => jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::Dialogue),
        "Lyric" if is_in_block => {
            jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::Lyric)
        }
        "Lyric" => jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::Lyric),
        other => flow_width_for_kind(other, geometry),
    }
}

fn wrap_lines_for_parity_kind(_kind: &str, text: &str, width_chars: usize) -> Vec<String> {
    jumpcut::pagination::wrapping::wrap_text_for_element(
        text,
        &jumpcut::pagination::wrapping::WrapConfig::with_exact_width_chars(width_chars),
    )
}

fn render_line_break_parity_review(report: &LineBreakParityReport) -> String {
    let disagreements: Vec<&LineBreakParityItem> = report
        .items
        .iter()
        .filter(|item| item.lines_agree == Some(false))
        .collect();

    let mut review = format!(
        "# Big Fish Line-Break Parity Review\n\n\
Run this command to regenerate this packet:\n\n\
```bash\n\
cargo test --test pagination_corpus_harness_test build_big_fish_line_break_parity_packet -- --ignored --nocapture\n\
```\n\n\
Files in this packet:\n\n\
- `target/pagination-debug/big-fish-linebreak-parity/REVIEW.md`\n\
- `target/pagination-debug/big-fish-linebreak-parity/parity.json`\n\n\
Coverage summary:\n\n\
- exact unique items: {exact_unique}\n\
- exact ambiguous items: {exact_ambiguous}\n\
- unsupported/unmatched items: {unsupported}\n\
- exact-unique line disagreements: {disagreements}\n\n\
How to read `parity.json`:\n\n\
- `expected_wrapped_lines` are our current wrapped lines for the recoverable text fragment\n\
- `pdf_lines` are the exact PDF-extracted lines when the page match is unique\n\
- `lines_agree = false` means the text match was trustworthy but our wrapping disagreed with the PDF\n\
- `match_kind = exact_ambiguous` means the same text appears multiple times on that page; do not trust it as ground truth\n\
- `match_kind = unsupported-fragment` usually means a split dialogue fragment whose exact per-page text cannot be reconstructed from the canonical fixture alone\n\n\
Read these first:\n\n",
        exact_unique = report.exact_unique_count,
        exact_ambiguous = report.exact_ambiguous_count,
        unsupported = report.unsupported_count,
        disagreements = report.disagreement_count,
    );

    for item in disagreements.iter().take(10) {
        review.push_str(&format!(
            "- `{element_id}` page {page} `{kind}` width={width:?}\n  expected: {expected:?}\n  pdf: {pdf:?}\n",
            element_id = item.element_id,
            page = item.page_number,
            kind = item.kind,
            width = item.width_chars,
            expected = item.expected_wrapped_lines,
            pdf = item.pdf_lines,
        ));
    }

    review.push_str("\nIf you only inspect one example, search for `el-00787` in `parity.json`.\n");

    review
}
struct MeasuredItem {
    content_lines: f32,
    spacing_above: f32,
}

fn measured_lines_for_item(
    item: &jumpcut::pagination::PageItem,
    elements: &HashMap<String, NormalizedElement>,
    geometry: &LayoutGeometry,
) -> MeasuredItem {
    let Some(element) = elements.get(&item.element_id) else {
        return MeasuredItem { content_lines: 0.0, spacing_above: 0.0 };
    };

    let element_type = match item.kind.as_str() {
        "Character" => jumpcut::pagination::wrapping::ElementType::Character,
        "Parenthetical" => jumpcut::pagination::wrapping::ElementType::Parenthetical,
        "Dialogue" => jumpcut::pagination::wrapping::ElementType::Dialogue,
        "Lyric" => jumpcut::pagination::wrapping::ElementType::Lyric,
        other => jumpcut::pagination::wrapping::ElementType::from_flow_kind(&flow_width_kind(other)),
    };

    let config = jumpcut::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    let text = match item.line_range {
        Some((start, end)) => slice_explicit_lines(&element.text, start, end),
        None => element.text.clone(),
    };
    let content_lines = jumpcut::pagination::wrapping::wrap_text_for_element(&text, &config).len() as f32 * geometry.line_height;

    let spacing_above = match element_type {
        jumpcut::pagination::wrapping::ElementType::Action => geometry.action_spacing_before,
        jumpcut::pagination::wrapping::ElementType::SceneHeading => geometry.scene_heading_spacing_before,
        jumpcut::pagination::wrapping::ElementType::Character => geometry.character_spacing_before,
        jumpcut::pagination::wrapping::ElementType::Transition => geometry.transition_spacing_before,
        jumpcut::pagination::wrapping::ElementType::Lyric => geometry.lyric_spacing_before,
        _ => 1.0, 
    };

    MeasuredItem {
        content_lines,
        spacing_above,
    }
}

// Unit measurement helpers removed as redundant with MeasuredItem logic in harness


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

fn flow_width_for_kind(kind: &str, geometry: &LayoutGeometry) -> usize {
    let flow_kind = flow_width_kind(kind);
    jumpcut::pagination::margin::calculate_element_width(geometry, jumpcut::pagination::wrapping::ElementType::from_flow_kind(&flow_kind))
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
    lines_per_page: f32,
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
    lines_per_page: f32,
    score: (usize, usize, usize),
    total_issues: usize,
    wrong_page: usize,
    wrong_fragment: usize,
    missing: usize,
    unexpected: usize,
    report: jumpcut::pagination::ComparisonReport,
}

struct BigFishReviewSummary {
    stem: String,
    fixture_path: String,
    page_range: Vec<u32>,
    lines_per_page: f32,
    total_issues: usize,
    wrong_page: usize,
    wrong_fragment: usize,
    missing: usize,
    unexpected: usize,
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

struct ProbeRun {
    lines_per_page: f32,
    score: (usize, usize, usize),
    actual: PaginatedScreenplay,
    geometry: LayoutGeometry,
    report: jumpcut::pagination::ComparisonReport,
}

#[derive(Serialize)]
struct DebugPageBreakFixture {
    screenplay: String,
    style_profile: String,
    source: PageBreakFixtureSourceRefs,
    scope: jumpcut::pagination::PaginationScope,
    lines_per_page: f32,
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
    measured_lines: f32,
    spacing_before_lines: f32,
    intrinsic_top_spacing_lines: f32,
    intrinsic_bottom_spacing_lines: f32,
    fragment: jumpcut::pagination::Fragment,
    line_range: Option<(u32, u32)>,
    block_id: Option<String>,
    dual_dialogue_group: Option<String>,
    dual_dialogue_side: Option<u8>,
}

#[derive(Serialize)]
struct DebugMeasurement {
    flow_geometries: Vec<DebugGeometry>,
    dialogue_geometries: Vec<DebugGeometry>,
    action_spacing_before: f32,
    scene_heading_spacing_before: f32,
    character_spacing_before: f32,
    transition_spacing_before: f32,
    lyric_spacing_before: f32,
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

#[derive(Serialize)]
struct VisualComparisonOutput {
    screenplay: String,
    label: String,
    lines_per_page: f32,
    score: (usize, usize, usize),
    total_elements: usize,
    total_matches: usize,
    total_wrong_page: usize,
    total_wrong_fragment: usize,
    total_missing: usize,
    measurement: VisualMeasurementSummary,
    pages: Vec<VisualComparisonPage>,
}

#[derive(Serialize)]
struct VisualMeasurementSummary {
    flow_geometries: Vec<DebugGeometry>,
    dialogue_geometries: Vec<DebugGeometry>,
}

#[derive(Serialize)]
struct VisualComparisonPage {
    page_number: u32,
    lines_per_page: f32,
    measured_total_lines: u32,
    item_count: usize,
    items: Vec<VisualComparisonItem>,
}

#[derive(Serialize)]
struct VisualComparisonItem {
    element_id: String,
    kind: String,
    full_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_id: Option<String>,
    canonical_page: u32,
    canonical_fragment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_fragment: Option<String>,
    width_chars: usize,
    wrapped_lines: Vec<String>,
    content_lines: f32,
    top_spacing_lines: f32,
    bottom_spacing_lines: f32,
    spacing_before_lines: f32,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_range: Option<(u32, u32)>,
}
