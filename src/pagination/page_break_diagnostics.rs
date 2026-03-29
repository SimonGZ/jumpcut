use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::parse;
use crate::pagination::{
    build_semantic_screenplay, compare_paginated_to_fixture, normalize_screenplay,
    ComparisonIssueKind, DialoguePartKind, FlowKind, Fragment, LayoutGeometry, LineRange,
    NormalizedElement, NormalizedScreenplay, PageBreakFixture, PageBreakFixtureSourceRefs,
    PaginatedScreenplay, PaginationConfig, SemanticUnit, wrapping::ElementType,
};

pub fn write_big_fish_public_slice_json(debug_dir: &Path) {
    let fixture: PageBreakFixture =
        read_fixture("tests/fixtures/pagination/big-fish.split-page-breaks.json");
    let normalized = normalized_window_from_fountain(
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &fixture,
    );
    let semantic = build_semantic_screenplay(normalized.clone());
    let run = run_window_diagnostics(&fixture, &semantic, geometry_for_screenplay("big-fish"));
    let previews = preview_map(&normalized);
    let report = enrich_report_previews(run.report, &previews);
    let debug_fixture = paginated_to_debug_fixture(
        &run.actual,
        &fixture.source,
        &normalized,
        run.lines_per_page,
        &run.geometry,
        &previews,
    );

    fs::create_dir_all(debug_dir).unwrap();
    for stale_name in [
        "big-fish.actual.page-breaks.json",
        "big-fish.comparison-report.json",
        "big-fish.page-endings.json",
    ] {
        let _ = fs::remove_file(debug_dir.join(stale_name));
    }
    fs::write(
        debug_dir.join("big-fish.p18-19.actual.page-breaks.json"),
        serde_json::to_string_pretty(&debug_fixture).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("big-fish.p18-19.comparison-report.json"),
        serde_json::to_string_pretty(&ProbeDebugOutput {
            lines_per_page: run.lines_per_page,
            score: run.score,
            total_issues: report.total_issues(),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report,
        })
        .unwrap(),
    )
    .unwrap();
    let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
    let page_endings = build_page_endings_report(
        "big-fish",
        &run.actual,
        &normalized,
        54.0,
        &run.geometry,
        Some(&page_numbers),
    );
    fs::write(
        debug_dir.join("big-fish.p18-19.page-endings.json"),
        serde_json::to_string_pretty(&page_endings).unwrap(),
    )
    .unwrap();
}

pub fn write_selected_public_windows_json(debug_dir: &Path) {
    fs::create_dir_all(debug_dir).unwrap();

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
        let normalized = normalized_window_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized.clone());
        let run = run_window_diagnostics(&fixture, &semantic, geometry_for_screenplay(screenplay_id));
        let previews = preview_map(&normalized);
        let report = enrich_report_previews(run.report, &previews);
        let debug_fixture = paginated_to_debug_fixture(
            &run.actual,
            &fixture.source,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            &previews,
        );

        fs::write(
            debug_dir.join(format!("{stem}.actual.page-breaks.json")),
            serde_json::to_string_pretty(&debug_fixture).unwrap(),
        )
        .unwrap();
        fs::write(
            debug_dir.join(format!("{stem}.comparison-report.json")),
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: path.to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: report.total_issues(),
                wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report,
            })
            .unwrap(),
        )
        .unwrap();

        let pdf_line_counts = canonical_pdf_line_count_debug(screenplay_id, &fixture, &normalized);
        fs::write(
            debug_dir.join(format!("{stem}.pdf-line-counts.json")),
            serde_json::to_string_pretty(&pdf_line_counts).unwrap(),
        )
        .unwrap();
        let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
        let page_endings = build_page_endings_report(
            screenplay_id,
            &run.actual,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            Some(&page_numbers),
        );
        fs::write(
            debug_dir.join(format!("{stem}.page-endings.json")),
            serde_json::to_string_pretty(&page_endings).unwrap(),
        )
        .unwrap();
    }
}

pub fn write_big_fish_review_packet(debug_dir: &Path) {
    write_window_review_packet(
        debug_dir,
        "big-fish",
        "tests/fixtures/corpus/public/big-fish/source/source.fountain",
        &[
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
        ],
        render_big_fish_review_packet,
    );
}

pub fn write_little_women_review_packet(debug_dir: &Path) {
    write_window_review_packet(
        debug_dir,
        "little-women",
        "tests/fixtures/corpus/public/little-women/source/source.fountain",
        &[
            (
                "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
                "p4-6",
            ),
            (
                "tests/fixtures/pagination/little-women.p13-14.page-breaks.json",
                "p13-14",
            ),
        ],
        render_little_women_review_packet,
    );
}

pub fn write_little_women_full_script_page_break_packet(debug_dir: &Path) {
    let fixture_path = "tests/fixtures/corpus/public/little-women/canonical/page-breaks.json";
    let fixture: PageBreakFixture = read_fixture(fixture_path);
    let fountain = fs::read_to_string("tests/fixtures/corpus/public/little-women/source/source.fountain")
        .unwrap();
    let screenplay = parse(&fountain);
    let normalized = normalize_screenplay("little-women", &screenplay);
    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);
    let actual = PaginatedScreenplay::from_screenplay(
        "little-women",
        &screenplay,
        54.0,
        fixture.scope.clone(),
    );
    let report = compare_paginated_to_fixture(&actual, &fixture);
    let score = (
        report.total_issues(),
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issue_count(ComparisonIssueKind::WrongFragment),
    );
    let previews = preview_map(&normalized);
    let report = enrich_report_previews(report, &previews);
    let debug_fixture = paginated_to_debug_fixture(
        &actual,
        &fixture.source,
        &normalized,
        54.0,
        &config.geometry,
        &previews,
    );

    fs::create_dir_all(debug_dir).unwrap();
    fs::write(
        debug_dir.join("actual.page-breaks.json"),
        serde_json::to_string_pretty(&debug_fixture).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("comparison-report.json"),
        serde_json::to_string_pretty(&FixtureProbeDebugOutput {
            fixture_path: fixture_path.to_string(),
            page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
            lines_per_page: 54.0,
            score,
            total_issues: report.total_issues(),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: report.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("pseudo-pdf.txt"),
        render_pseudo_pdf_output(&actual, &normalized, 54.0, &config.geometry),
    )
    .unwrap();
    let page_endings = build_page_endings_report(
        "little-women",
        &actual,
        &normalized,
        54.0,
        &config.geometry,
        None,
    );
    fs::write(
        debug_dir.join("page-endings.json"),
        serde_json::to_string_pretty(&page_endings).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("REVIEW.md"),
        render_little_women_full_script_review_packet(54.0, score, &fixture, &report),
    )
    .unwrap();
}

pub fn write_big_fish_full_script_page_break_packet(debug_dir: &Path) {
    let fixture_path = "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json";
    let fixture: PageBreakFixture = read_fixture(fixture_path);
    let fountain = fs::read_to_string("tests/fixtures/corpus/public/big-fish/source/source.fountain")
        .unwrap();
    let screenplay = parse(&fountain);
    let normalized = normalize_screenplay("big-fish", &screenplay);
    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);
    let actual = PaginatedScreenplay::from_screenplay(
        "big-fish",
        &screenplay,
        54.0,
        fixture.scope.clone(),
    );
    let report = compare_paginated_to_fixture(&actual, &fixture);
    let score = (
        report.total_issues(),
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issue_count(ComparisonIssueKind::WrongFragment),
    );
    let previews = preview_map(&normalized);
    let report = enrich_report_previews(report, &previews);
    let debug_fixture = paginated_to_debug_fixture(
        &actual,
        &fixture.source,
        &normalized,
        54.0,
        &config.geometry,
        &previews,
    );

    fs::create_dir_all(debug_dir).unwrap();
    write_fixture_symlink(debug_dir, fixture_path);
    fs::write(
        debug_dir.join("actual.page-breaks.json"),
        serde_json::to_string_pretty(&debug_fixture).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("comparison-report.json"),
        serde_json::to_string_pretty(&FixtureProbeDebugOutput {
            fixture_path: fixture_path.to_string(),
            page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
            lines_per_page: 54.0,
            score,
            total_issues: report.total_issues(),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: report.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("pseudo-pdf.txt"),
        render_pseudo_pdf_output(&actual, &normalized, 54.0, &config.geometry),
    )
    .unwrap();
    let page_endings = build_page_endings_report(
        "big-fish",
        &actual,
        &normalized,
        54.0,
        &config.geometry,
        None,
    );
    fs::write(
        debug_dir.join("page-endings.json"),
        serde_json::to_string_pretty(&page_endings).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("REVIEW.md"),
        render_big_fish_full_script_review_packet(54.0, score, &fixture, &report),
    )
    .unwrap();
}

fn write_fixture_symlink(debug_dir: &Path, fixture_path: &str) {
    let link_path = debug_dir.join("canonical.page-breaks.json");
    let relative_target = Path::new("..")
        .join("..")
        .join("..")
        .join(fixture_path);
    let _ = fs::remove_file(&link_path);
    symlink(&relative_target, &link_path).unwrap();
}

pub fn write_mostly_genius_full_script_page_break_packet(debug_dir: &Path) {
    let fixture_path = "tests/fixtures/corpus/public/mostly-genius/canonical/page-breaks.json";
    let fixture: PageBreakFixture = read_fixture(fixture_path);
    let fountain = fs::read_to_string("tests/fixtures/corpus/public/mostly-genius/source/source.fountain")
        .unwrap();
    let screenplay = parse(&fountain);
    let normalized = normalize_screenplay("mostly-genius", &screenplay);
    let config = PaginationConfig::from_screenplay(&screenplay, 54.0);
    let actual = PaginatedScreenplay::from_screenplay(
        "mostly-genius",
        &screenplay,
        54.0,
        fixture.scope.clone(),
    );
    let report = compare_paginated_to_fixture(&actual, &fixture);
    let score = (
        report.total_issues(),
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issue_count(ComparisonIssueKind::WrongFragment),
    );
    let previews = preview_map(&normalized);
    let report = enrich_report_previews(report, &previews);
    let debug_fixture = paginated_to_debug_fixture(
        &actual,
        &fixture.source,
        &normalized,
        54.0,
        &config.geometry,
        &previews,
    );

    fs::create_dir_all(debug_dir).unwrap();
    fs::write(
        debug_dir.join("actual.page-breaks.json"),
        serde_json::to_string_pretty(&debug_fixture).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("comparison-report.json"),
        serde_json::to_string_pretty(&FixtureProbeDebugOutput {
            fixture_path: fixture_path.to_string(),
            page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
            lines_per_page: 54.0,
            score,
            total_issues: report.total_issues(),
            wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
            report: report.clone(),
        })
        .unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("pseudo-pdf.txt"),
        render_pseudo_pdf_output(&actual, &normalized, 54.0, &config.geometry),
    )
    .unwrap();
    let page_endings = build_page_endings_report(
        "mostly-genius",
        &actual,
        &normalized,
        54.0,
        &config.geometry,
        None,
    );
    fs::write(
        debug_dir.join("page-endings.json"),
        serde_json::to_string_pretty(&page_endings).unwrap(),
    )
    .unwrap();
    fs::write(
        debug_dir.join("REVIEW.md"),
        render_mostly_genius_full_script_review_packet(54.0, score, &fixture, &report),
    )
    .unwrap();
}

pub fn write_fd_probe_packets(debug_dir: &Path) {
    fs::create_dir_all(debug_dir).unwrap();
    let probe_root = Path::new("tests/fixtures/fd-probes");
    let mut summaries = Vec::new();

    let mut probe_dirs = fs::read_dir(probe_root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.join("expected.json").exists())
        .collect::<Vec<_>>();
    probe_dirs.sort();

    for probe_dir in probe_dirs {
        let expected_path = probe_dir.join("expected.json");
        let fountain_path = probe_dir.join("source.fountain");
        let spec: FinalDraftProbeSpec =
            serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();
        let fountain = fs::read_to_string(&fountain_path).unwrap();
        let screenplay = parse(&fountain);
        let normalized = normalize_screenplay(&spec.probe_id, &screenplay);
        let config = PaginationConfig::from_screenplay(&screenplay, spec.lines_per_page);
        let actual = PaginatedScreenplay::from_screenplay(
            &spec.probe_id,
            &screenplay,
            spec.lines_per_page,
            spec_scope(),
        );
        let semantic = build_semantic_screenplay(normalized.clone());
        let composed = crate::pagination::composer::compose(&semantic.units, &config.geometry);
        let layout_pages =
            crate::pagination::paginator::paginate(&composed, spec.lines_per_page, &config.geometry);
        let actual_matches = collect_fd_probe_matches(&spec, &actual, &layout_pages);

        let probe_debug_dir = debug_dir.join(probe_dir.file_name().unwrap());
        fs::create_dir_all(&probe_debug_dir).unwrap();
        fs::write(
            probe_debug_dir.join("pseudo-pdf.txt"),
            render_pseudo_pdf_output(&actual, &normalized, spec.lines_per_page, &config.geometry),
        )
        .unwrap();
        fs::write(
            probe_debug_dir.join("actual-observation.json"),
            serde_json::to_string_pretty(&FdProbeActualObservation {
                probe_id: spec.probe_id.clone(),
                description: spec.description.clone(),
                status: spec.status.clone(),
                lines_per_page: spec.lines_per_page,
                target: spec.target.clone(),
                expected: spec.expected.clone(),
                actual_matches,
            })
            .unwrap(),
        )
        .unwrap();

        summaries.push(FdProbeSummary {
            folder: probe_dir.file_name().unwrap().to_string_lossy().into_owned(),
            probe_id: spec.probe_id,
            status: spec.status,
        });
    }

    fs::write(debug_dir.join("REVIEW.md"), render_fd_probe_review(&summaries)).unwrap();
}

fn write_window_review_packet(
    debug_dir: &Path,
    screenplay_id: &str,
    fountain_path: &str,
    fixtures: &[(&str, &str)],
    review_renderer: fn(&[ReviewSummary]) -> String,
) {
    fs::create_dir_all(debug_dir).unwrap();

    let mut summaries = Vec::new();
    for (fixture_path, stem) in fixtures {
        let fixture: PageBreakFixture = read_fixture(fixture_path);
        let normalized = normalized_window_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized.clone());
        let run = run_window_diagnostics(&fixture, &semantic, geometry_for_screenplay(screenplay_id));
        let previews = preview_map(&normalized);
        let report = enrich_report_previews(run.report.clone(), &previews);
        let debug_fixture = paginated_to_debug_fixture(
            &run.actual,
            &fixture.source,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            &previews,
        );
        let pdf_line_counts = canonical_pdf_line_count_debug(screenplay_id, &fixture, &normalized);

        fs::write(
            debug_dir.join(format!("{stem}.actual.page-breaks.json")),
            serde_json::to_string_pretty(&debug_fixture).unwrap(),
        )
        .unwrap();
        fs::write(
            debug_dir.join(format!("{stem}.comparison-report.json")),
            serde_json::to_string_pretty(&FixtureProbeDebugOutput {
                fixture_path: (*fixture_path).to_string(),
                page_numbers: fixture.pages.iter().map(|page| page.number).collect(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_issues: report.total_issues(),
                wrong_page: report.issue_count(ComparisonIssueKind::WrongPage),
                wrong_fragment: report.issue_count(ComparisonIssueKind::WrongFragment),
                missing: report.issue_count(ComparisonIssueKind::MissingOccurrence),
                unexpected: report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
                report: report.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(
            debug_dir.join(format!("{stem}.pdf-line-counts.json")),
            serde_json::to_string_pretty(&pdf_line_counts).unwrap(),
        )
        .unwrap();
        fs::write(
            debug_dir.join(format!("{stem}.pseudo-pdf.txt")),
            render_pseudo_pdf_output(&run.actual, &normalized, run.lines_per_page, &run.geometry),
        )
        .unwrap();
        let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
        let page_endings = build_page_endings_report(
            screenplay_id,
            &run.actual,
            &normalized,
            run.lines_per_page,
            &run.geometry,
            Some(&page_numbers),
        );
        fs::write(
            debug_dir.join(format!("{stem}.page-endings.json")),
            serde_json::to_string_pretty(&page_endings).unwrap(),
        )
        .unwrap();

        summaries.push(ReviewSummary {
            stem: (*stem).into(),
            fixture_path: (*fixture_path).into(),
            page_range: fixture.pages.iter().map(|page| page.number).collect(),
            lines_per_page: run.lines_per_page,
            total_issues: run.report.total_issues(),
            wrong_page: run.report.issue_count(ComparisonIssueKind::WrongPage),
            wrong_fragment: run.report.issue_count(ComparisonIssueKind::WrongFragment),
            missing: run.report.issue_count(ComparisonIssueKind::MissingOccurrence),
            unexpected: run.report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
        });
    }

    fs::write(debug_dir.join("REVIEW.md"), review_renderer(&summaries)).unwrap();
}

pub fn write_visual_comparison_data(debug_dir: &Path) {
    fs::create_dir_all(debug_dir).unwrap();

    for (screenplay_id, fixture_path, fountain_path, label, fixed_lpp) in [
        (
            "big-fish",
            "tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json",
            "tests/fixtures/corpus/public/big-fish/source/source.fountain",
            "big-fish",
            Some(54u32),
        ),
        (
            "little-women",
            "tests/fixtures/pagination/little-women.p4-6.page-breaks.json",
            "tests/fixtures/corpus/public/little-women/source/source.fountain",
            "little-women-p4-6",
            None,
        ),
    ] {
        let fixture: PageBreakFixture = read_fixture(fixture_path);
        let normalized = normalized_window_from_fountain(screenplay_id, fountain_path, &fixture);
        let semantic = build_semantic_screenplay(normalized.clone());
        let measurement = geometry_for_screenplay(screenplay_id);

        let run = if let Some(lpp) = fixed_lpp {
            let config = PaginationConfig {
                lines_per_page: lpp as f32,
                geometry: measurement.clone(),
            };
            let actual = PaginatedScreenplay::paginate(
                semantic.clone(),
                config,
                fixture.style_profile.clone(),
                fixture.scope.clone(),
            );
            let report = compare_paginated_to_fixture(&actual, &fixture);
            let score = (
                report.total_issues(),
                report.issue_count(ComparisonIssueKind::WrongPage),
                report.issue_count(ComparisonIssueKind::WrongFragment),
            );
            PaginatedWindowRun {
                lines_per_page: lpp as f32,
                score,
                actual,
                geometry: measurement,
                report,
            }
        } else {
            run_window_diagnostics(&fixture, &semantic, measurement)
        };

        let elements = normalized_element_map(&normalized);
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
                let line_range_tuple = item.line_range.map(|lr| (lr.0, lr.1));
                let wrap_text = match (line_range_tuple, element) {
                    (Some((s, e)), Some(_)) => slice_explicit_lines(&full_text, s, e),
                    _ => full_text.clone(),
                };
                let element_type = ElementType::from_item_kind(&item.kind, item.dual_dialogue_side);
                let config = crate::pagination::wrapping::WrapConfig::from_geometry(&run.geometry, element_type);
                let wrapped_lines = crate::pagination::wrapping::wrap_text_for_element(&wrap_text, &config);
                let width_chars = config.exact_width_chars;

                let (content_lines, spacing_before) = measure_visual_item(
                    &item.element_id,
                    &item.kind,
                    line_range_tuple,
                    item.dual_dialogue_side,
                    &elements,
                    &run.geometry,
                );
                page_measured_lines += (content_lines + spacing_before) as u32;

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
            }

            visual_pages.push(VisualComparisonPage {
                page_number: fixture_page.number,
                lines_per_page: run.lines_per_page,
                measured_total_lines: page_measured_lines,
                item_count: page_items.len(),
                items: page_items,
            });
        }

        let measurement_summary = VisualMeasurementSummary {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, &run.geometry),
                debug_flow_geometry("Scene Heading", "Scene Heading", FlowKind::SceneHeading, &run.geometry),
                debug_flow_geometry("Transition", "Transition", FlowKind::Transition, &run.geometry),
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry("Character", "Character", DialoguePartKind::Character, &run.geometry),
                debug_dialogue_geometry("Dialogue", "Dialogue", DialoguePartKind::Dialogue, &run.geometry),
                debug_dialogue_geometry(
                    "Parenthetical",
                    "Parenthetical",
                    DialoguePartKind::Parenthetical,
                    &run.geometry,
                ),
                debug_dialogue_geometry("Lyric", "Lyric", DialoguePartKind::Lyric, &run.geometry),
            ],
        };

        let total_elements: usize = visual_pages.iter().map(|page| page.items.len()).sum();
        let total_matches: usize = visual_pages
            .iter()
            .flat_map(|page| &page.items)
            .filter(|item| item.status == "match")
            .count();
        let total_wrong_page: usize = visual_pages
            .iter()
            .flat_map(|page| &page.items)
            .filter(|item| item.status == "wrong_page")
            .count();
        let total_wrong_fragment: usize = visual_pages
            .iter()
            .flat_map(|page| &page.items)
            .filter(|item| item.status == "wrong_fragment")
            .count();
        let total_missing: usize = visual_pages
            .iter()
            .flat_map(|page| &page.items)
            .filter(|item| item.status == "missing")
            .count();

        fs::write(
            debug_dir.join(format!("{label}.comparison.json")),
            serde_json::to_string_pretty(&VisualComparisonOutput {
                screenplay: screenplay_id.into(),
                label: label.into(),
                lines_per_page: run.lines_per_page,
                score: run.score,
                total_elements,
                total_matches,
                total_wrong_page,
                total_wrong_fragment,
                total_missing,
                measurement: measurement_summary,
                pages: visual_pages,
            })
            .unwrap(),
        )
        .unwrap();
    }
}

fn run_window_diagnostics(
    fixture: &PageBreakFixture,
    semantic: &crate::pagination::SemanticScreenplay,
    geometry: LayoutGeometry,
) -> PaginatedWindowRun {
    let page_numbers: Vec<u32> = fixture.pages.iter().map(|page| page.number).collect();
    let config = PaginationConfig {
        lines_per_page: 54.0,
        geometry: geometry.clone(),
    };
    let full_actual = PaginatedScreenplay::paginate(
        semantic.clone(),
        config,
        fixture.style_profile.clone(),
        fixture.scope.clone(),
    );
    let actual = slice_paginated_to_fixture_window(&full_actual, &page_numbers);
    let report = compare_paginated_to_fixture(&actual, fixture);
    let score = (
        report.total_issues(),
        report.issue_count(ComparisonIssueKind::WrongPage),
        report.issue_count(ComparisonIssueKind::WrongFragment),
    );

    PaginatedWindowRun {
        lines_per_page: 54.0,
        score,
        actual,
        geometry,
        report,
    }
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

fn enrich_report_previews(
    mut report: crate::pagination::ComparisonReport,
    previews: &HashMap<String, String>,
) -> crate::pagination::ComparisonReport {
    for issue in &mut report.issues {
        if issue.text_preview.is_none() {
            issue.text_preview = previews.get(&issue.element_id).cloned();
        }
    }
    report
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
    let paged_layout_totals = paged_layout_page_totals(normalized, lines_per_page, geometry);

    DebugPageBreakFixture {
        screenplay: actual.screenplay.clone(),
        style_profile: actual.style_profile.clone(),
        source: source.clone(),
        scope: actual.scope.clone(),
        lines_per_page,
        measurement: DebugMeasurement {
            flow_geometries: vec![
                debug_flow_geometry("Action", "Action", FlowKind::Action, geometry),
                debug_flow_geometry("Scene Heading", "Scene Heading", FlowKind::SceneHeading, geometry),
                debug_flow_geometry("Transition", "Transition", FlowKind::Transition, geometry),
            ],
            dialogue_geometries: vec![
                debug_dialogue_geometry("Dialogue", "Dialogue", DialoguePartKind::Dialogue, geometry),
                debug_dialogue_geometry("Character", "Character", DialoguePartKind::Character, geometry),
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
            .zip(paged_layout_totals)
            .map(|(page, block_total_lines)| {
                debug_page(page, &elements, geometry, previews, block_total_lines)
            })
            .collect(),
    }
}

fn debug_flow_geometry(kind: &str, source_style: &str, flow_kind: FlowKind, geometry: &LayoutGeometry) -> DebugGeometry {
    let (left_indent_in, right_indent_in) = match flow_kind {
        FlowKind::SceneHeading => (geometry.action_left, geometry.action_right),
        FlowKind::ColdOpening => (geometry.cold_opening_left, geometry.cold_opening_right),
        FlowKind::NewAct => (geometry.new_act_left, geometry.new_act_right),
        FlowKind::EndOfAct => (geometry.end_of_act_left, geometry.end_of_act_right),
        FlowKind::Transition => (geometry.transition_left, geometry.transition_right),
        _ => (geometry.action_left, geometry.action_right),
    };

    DebugGeometry {
        kind: kind.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: crate::pagination::margin::calculate_element_width(
            geometry,
            crate::pagination::wrapping::ElementType::from_flow_kind(&flow_kind),
        ),
    }
}

fn debug_dialogue_geometry(
    kind: &str,
    source_style: &str,
    part_kind: DialoguePartKind,
    geometry: &LayoutGeometry,
) -> DebugGeometry {
    let (left_indent_in, right_indent_in) = match part_kind {
        DialoguePartKind::Character => (geometry.character_left, geometry.character_right),
        DialoguePartKind::Parenthetical => (geometry.parenthetical_left, geometry.parenthetical_right),
        DialoguePartKind::Lyric => (geometry.lyric_left, geometry.lyric_right),
        DialoguePartKind::Dialogue => (geometry.dialogue_left, geometry.dialogue_right),
    };

    DebugGeometry {
        kind: kind.into(),
        source_style: source_style.into(),
        left_indent_in,
        right_indent_in,
        width_chars: crate::pagination::margin::calculate_element_width(
            geometry,
            crate::pagination::wrapping::ElementType::from_dialogue_part_kind(&part_kind),
        ),
    }
}

fn debug_page(
    page: &crate::pagination::Page,
    elements: &HashMap<String, NormalizedElement>,
    geometry: &LayoutGeometry,
    previews: &HashMap<String, String>,
    block_total_lines: f32,
) -> DebugPageBreakFixturePage {
    let mut items = Vec::with_capacity(page.items.len());
    for item in &page.items {
        let items_measured = measured_lines_for_item(item, elements, geometry);
        items.push(DebugPageBreakItem {
            element_id: item.element_id.clone(),
            kind: item.kind.clone(),
            text_preview: previews.get(&item.element_id).cloned(),
            measured_lines: items_measured.content_lines,
            intrinsic_spacing_before_lines: items_measured.spacing_above,
            width_chars: items_measured.width_chars,
            wrapped_lines: items_measured.wrapped_lines,
            fragment: item.fragment.clone(),
            line_range: item.line_range,
            block_id: item.block_id.clone(),
            dual_dialogue_group: item.dual_dialogue_group.clone(),
            dual_dialogue_side: item.dual_dialogue_side,
        });
    }

    DebugPageBreakFixturePage {
        number: page.metadata.number,
        block_total_lines,
        item_count: page.items.len(),
        block_count: page.blocks.len(),
        items,
    }
}

fn paged_layout_page_totals(
    normalized: &NormalizedScreenplay,
    lines_per_page: f32,
    geometry: &LayoutGeometry,
) -> Vec<f32> {
    let semantic = build_semantic_screenplay(normalized.clone());
    let blocks = crate::pagination::composer::compose(&semantic.units, geometry);
    crate::pagination::paginator::paginate(&blocks, lines_per_page, geometry)
        .into_iter()
        .filter(|page| {
            page.blocks
                .iter()
                .any(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        })
        .map(|page| page.blocks.iter().map(|block| block.spacing_above + block.content_lines).sum())
        .collect()
}

fn render_pseudo_pdf_output(
    actual: &PaginatedScreenplay,
    normalized: &NormalizedScreenplay,
    lines_per_page: f32,
    geometry: &LayoutGeometry,
) -> String {
    let semantic = build_semantic_screenplay(normalized.clone());
    let blocks = crate::pagination::composer::compose(&semantic.units, geometry);
    let layout_pages = crate::pagination::paginator::paginate(&blocks, lines_per_page, geometry)
        .into_iter()
        .filter(|page| {
            page.blocks
                .iter()
                .any(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        });

    let mut out = String::new();
    for (page, layout_page) in actual.pages.iter().zip(layout_pages) {
        out.push_str(&format!("=== PAGE {} START ===\n", page.metadata.number));
        let mut line_no: u32 = 1;

        for block in &layout_page.blocks {
            for _ in 0..(block.spacing_above.round() as usize) {
                out.push_str(&format!("{line_no:02}:\n"));
                line_no += 1;
            }

            for line in render_layout_block_lines(block, geometry) {
                if line.counted {
                    out.push_str(&format!("{line_no:02}: {}\n", line.text));
                    line_no += 1;
                } else {
                    out.push_str(&format!("00: {}\n", line.text));
                }
            }
        }

        out.push_str(&format!("=== PAGE {} END ===\n\n", page.metadata.number));
    }

    out
}

fn build_page_endings_report(
    screenplay_id: &str,
    actual: &PaginatedScreenplay,
    normalized: &NormalizedScreenplay,
    lines_per_page: f32,
    geometry: &LayoutGeometry,
    page_numbers: Option<&[u32]>,
) -> PageEndingReport {
    let canonical_pages = public_pdf_pages(screenplay_id);
    let actual_pages = rendered_actual_page_lines(actual, normalized, lines_per_page, geometry);
    let mut selected_page_numbers: Vec<u32> = match page_numbers {
        Some(page_numbers) => page_numbers.to_vec(),
        None => canonical_pages
            .keys()
            .chain(actual_pages.keys())
            .copied()
            .collect(),
    };
    selected_page_numbers.sort_unstable();
    selected_page_numbers.dedup();

    let pages = selected_page_numbers
        .into_iter()
        .map(|page_number| {
            let canonical_lines = canonical_pages
                .get(&page_number)
                .cloned()
                .unwrap_or_default();
            let actual_lines = actual_pages.get(&page_number).cloned().unwrap_or_default();
            let canonical_last = last_meaningful_line(&canonical_lines);
            let actual_last = last_meaningful_line(&actual_lines);

            PageEndingItem {
                page_number,
                matches: canonical_last == actual_last,
                canonical_last_line: canonical_last,
                actual_last_line: actual_last,
                canonical_raw_last_line: canonical_lines
                    .iter()
                    .rev()
                    .find(|line| !line.trim().is_empty())
                    .cloned(),
                actual_raw_last_line: actual_lines
                    .iter()
                    .rev()
                    .find(|line| !line.trim().is_empty())
                    .cloned(),
            }
        })
        .collect::<Vec<_>>();

    let mismatch_count = pages.iter().filter(|page| !page.matches).count();

    PageEndingReport {
        screenplay: screenplay_id.to_string(),
        page_count: pages.len(),
        mismatch_count,
        pages,
    }
}

fn rendered_actual_page_lines(
    actual: &PaginatedScreenplay,
    normalized: &NormalizedScreenplay,
    lines_per_page: f32,
    geometry: &LayoutGeometry,
) -> HashMap<u32, Vec<String>> {
    let semantic = build_semantic_screenplay(normalized.clone());
    let blocks = crate::pagination::composer::compose(&semantic.units, geometry);
    let layout_pages = crate::pagination::paginator::paginate(&blocks, lines_per_page, geometry)
        .into_iter()
        .filter(|page| {
            page.blocks
                .iter()
                .any(|block| !matches!(block.unit, SemanticUnit::PageStart(_)))
        });

    actual
        .pages
        .iter()
        .zip(layout_pages)
        .map(|(page, layout_page)| {
            let mut lines = Vec::new();

            for block in &layout_page.blocks {
                for _ in 0..(block.spacing_above.round() as usize) {
                    lines.push(String::new());
                }

                lines.extend(
                    render_layout_block_lines(block, geometry)
                        .into_iter()
                        .map(|line| line.text),
                );
            }

            (page.metadata.number, lines)
        })
        .collect()
}

fn last_meaningful_line(lines: &[String]) -> Option<String> {
    lines.iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && !is_footer_page_number_line(line))
        .map(str::to_string)
}

fn is_footer_page_number_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.ends_with('.')
        && trimmed[..trimmed.len() - 1]
            .chars()
            .all(|ch| ch.is_ascii_digit())
}

struct DiagnosticRenderedLine {
    text: String,
    counted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ProbeStatus {
    Draft,
    Active,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ProbeTargetKind {
    Dialogue,
    Flow,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FinalDraftProbeTarget {
    kind: ProbeTargetKind,
    contains_text: String,
    #[serde(default)]
    speaker: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum FinalDraftProbeExpectation {
    Split {
        top_page: u32,
        bottom_page: u32,
        top_fragment_ends_with: String,
        bottom_fragment_starts_with: String,
    },
    PushWhole {
        absent_from_page: u32,
        whole_on_page: u32,
        starts_with: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FinalDraftProbeSpec {
    probe_id: String,
    description: String,
    status: ProbeStatus,
    lines_per_page: f32,
    target: FinalDraftProbeTarget,
    expected: FinalDraftProbeExpectation,
    #[serde(default)]
    final_draft_notes: Vec<String>,
}

#[derive(Serialize)]
struct FdProbeActualObservation {
    probe_id: String,
    description: String,
    status: ProbeStatus,
    lines_per_page: f32,
    target: FinalDraftProbeTarget,
    expected: FinalDraftProbeExpectation,
    actual_matches: Vec<FdProbeActualMatch>,
}

#[derive(Serialize)]
struct FdProbeActualMatch {
    page_number: u32,
    fragment: Fragment,
    text: String,
}

struct FdProbeSummary {
    folder: String,
    probe_id: String,
    status: ProbeStatus,
}

#[derive(Serialize)]
struct PageEndingReport {
    screenplay: String,
    page_count: usize,
    mismatch_count: usize,
    pages: Vec<PageEndingItem>,
}

#[derive(Serialize)]
struct PageEndingItem {
    page_number: u32,
    matches: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_last_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_last_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_raw_last_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_raw_last_line: Option<String>,
}

fn render_layout_block_lines(
    block: &crate::pagination::composer::LayoutBlock<'_>,
    geometry: &LayoutGeometry,
) -> Vec<DiagnosticRenderedLine> {
    if let SemanticUnit::Dialogue(dialogue) = block.unit {
        return render_dialogue_fragment_lines(
            dialogue,
            &block.fragment,
            block.dialogue_split.as_ref(),
            block.content_lines,
            geometry,
        );
    }

    if let SemanticUnit::Flow(flow) = block.unit {
        if let Some(plan) = block.flow_split.as_ref() {
            let text = match block.fragment {
                Fragment::ContinuedToNext => plan.top_text.clone(),
                Fragment::ContinuedFromPrev => plan.bottom_text.clone(),
                Fragment::ContinuedFromPrevAndToNext => plan.top_text.clone(),
                Fragment::Whole => flow.text.clone(),
            };

            return render_indented_lines(
                &text,
                ElementType::from_flow_kind(&flow.kind),
                geometry,
            )
            .into_iter()
            .map(|text| DiagnosticRenderedLine { text, counted: true })
            .collect();
        }
    }

    let all_lines = render_semantic_unit_lines(block.unit, geometry);
    let target_line_count = (block.content_lines / geometry.line_height).round() as usize;

    let lines = match block.fragment {
        Fragment::Whole => all_lines,
        Fragment::ContinuedToNext => all_lines.into_iter().take(target_line_count).collect(),
        Fragment::ContinuedFromPrev => {
            let len = all_lines.len();
            all_lines
                .into_iter()
                .skip(len.saturating_sub(target_line_count))
                .collect()
        }
        Fragment::ContinuedFromPrevAndToNext => all_lines.into_iter().take(target_line_count).collect(),
    };

    lines
        .into_iter()
        .map(|text| DiagnosticRenderedLine { text, counted: true })
        .collect()
}

fn collect_fd_probe_matches(
    spec: &FinalDraftProbeSpec,
    actual: &PaginatedScreenplay,
    layout_pages: &[crate::pagination::paginator::Page<'_>],
) -> Vec<FdProbeActualMatch> {
    actual
        .pages
        .iter()
        .zip(layout_pages.iter())
        .flat_map(|(page, layout_page)| {
            layout_page.blocks.iter().filter_map(|block| {
                fd_probe_block_matches(block, &spec.target).then(|| FdProbeActualMatch {
                    page_number: page.metadata.number,
                    fragment: block.fragment.clone(),
                    text: fd_probe_rendered_block_text(block, &spec.target.kind),
                })
            })
        })
        .collect()
}

fn fd_probe_block_matches(
    block: &crate::pagination::composer::LayoutBlock<'_>,
    target: &FinalDraftProbeTarget,
) -> bool {
    match (&target.kind, block.unit) {
        (ProbeTargetKind::Dialogue, SemanticUnit::Dialogue(dialogue)) => {
            let speaker_matches = target.speaker.as_ref().is_none_or(|speaker| {
                dialogue.parts.iter().any(|part| {
                    matches!(part.kind, DialoguePartKind::Character)
                        && part.text.trim() == speaker.trim()
                })
            });
            speaker_matches
                && dialogue
                    .parts
                    .iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .contains(&target.contains_text)
        }
        (ProbeTargetKind::Flow, SemanticUnit::Flow(flow)) => flow.text.contains(&target.contains_text),
        _ => false,
    }
}

fn fd_probe_rendered_block_text(
    block: &crate::pagination::composer::LayoutBlock<'_>,
    target_kind: &ProbeTargetKind,
) -> String {
    match target_kind {
        ProbeTargetKind::Dialogue => {
            let SemanticUnit::Dialogue(dialogue) = block.unit else {
                panic!("expected dialogue block");
            };
            match block.dialogue_split.as_ref() {
                Some(plan) => dialogue
                    .parts
                    .iter()
                    .zip(plan.parts.iter())
                    .map(|(part, part_plan)| match block.fragment {
                        Fragment::Whole => part.text.clone(),
                        Fragment::ContinuedToNext => part_plan.top_text.clone(),
                        Fragment::ContinuedFromPrev => part_plan.bottom_text.clone(),
                        Fragment::ContinuedFromPrevAndToNext => part_plan.top_text.clone(),
                    })
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n"),
                None => dialogue
                    .parts
                    .iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            }
        }
        ProbeTargetKind::Flow => {
            let SemanticUnit::Flow(flow) = block.unit else {
                panic!("expected flow block");
            };
            match block.flow_split.as_ref() {
                Some(plan) => match block.fragment {
                    Fragment::Whole => flow.text.clone(),
                    Fragment::ContinuedToNext => plan.top_text.clone(),
                    Fragment::ContinuedFromPrev => plan.bottom_text.clone(),
                    Fragment::ContinuedFromPrevAndToNext => plan.top_text.clone(),
                },
                None => flow.text.clone(),
            }
        }
    }
}

fn spec_scope() -> crate::pagination::PaginationScope {
    crate::pagination::PaginationScope {
        title_page_count: None,
        body_start_page: None,
    }
}

fn render_dialogue_fragment_lines(
    dialogue: &crate::pagination::DialogueUnit,
    fragment: &Fragment,
    split_plan: Option<&crate::pagination::dialogue_split::DialogueSplitPlan>,
    content_lines: f32,
    geometry: &LayoutGeometry,
) -> Vec<DiagnosticRenderedLine> {
    if let Some(plan) = split_plan {
        match fragment {
            Fragment::ContinuedToNext => {
                let mut lines = plan
                    .parts
                    .iter()
                    .zip(dialogue.parts.iter())
                    .flat_map(|(part, dialogue_part)| {
                        let element_type =
                            ElementType::from_dialogue_part_kind(&dialogue_part.kind);
                        render_indented_lines(&part.top_text, element_type, geometry)
                    })
                    .map(|text| DiagnosticRenderedLine { text, counted: true })
                    .collect::<Vec<_>>();
                lines.push(render_more_marker_line());
                return lines;
            }
            Fragment::ContinuedFromPrev => {
                let continuation_prefix = render_dialogue_continuation_prefix(dialogue, geometry);
                return continuation_prefix
                    .into_iter()
                    .map(|text| DiagnosticRenderedLine {
                        text,
                        counted: false,
                    })
                    .chain(
                        plan.parts
                            .iter()
                            .zip(dialogue.parts.iter())
                            .flat_map(|(part, dialogue_part)| {
                                let element_type =
                                    ElementType::from_dialogue_part_kind(&dialogue_part.kind);
                                render_indented_lines(&part.bottom_text, element_type, geometry)
                            })
                            .map(|text| DiagnosticRenderedLine { text, counted: true })
                            .collect::<Vec<_>>(),
                    )
                    .collect();
            }
            Fragment::Whole | Fragment::ContinuedFromPrevAndToNext => {}
        }
    }

    let all_lines = render_semantic_unit_lines(&SemanticUnit::Dialogue(dialogue.clone()), geometry);
    let continuation_prefix = render_dialogue_continuation_prefix(dialogue, geometry);
    let target_line_count = (content_lines / geometry.line_height).round() as usize;

    match fragment {
        Fragment::Whole => all_lines
            .into_iter()
            .map(|text| DiagnosticRenderedLine { text, counted: true })
            .collect(),
        Fragment::ContinuedToNext => {
            let mut lines = all_lines
                .into_iter()
                .take(target_line_count.saturating_sub(1))
                .map(|text| DiagnosticRenderedLine { text, counted: true })
                .collect::<Vec<_>>();
            lines.push(render_more_marker_line());
            lines
        }
        Fragment::ContinuedFromPrev => {
            let len = all_lines.len();
            continuation_prefix
                .into_iter()
                .map(|text| DiagnosticRenderedLine {
                    text,
                    counted: false,
                })
                .chain(
                    all_lines
                        .into_iter()
                        .skip(len.saturating_sub(target_line_count))
                        .map(|text| DiagnosticRenderedLine { text, counted: true }),
                )
                .collect()
        }
        Fragment::ContinuedFromPrevAndToNext => {
            let mut lines = continuation_prefix
                .into_iter()
                .map(|text| DiagnosticRenderedLine {
                    text,
                    counted: false,
                })
                .chain(
                    all_lines
                        .into_iter()
                        .take(target_line_count.saturating_sub(1))
                        .map(|text| DiagnosticRenderedLine { text, counted: true }),
                )
                .collect::<Vec<_>>();
            lines.push(render_more_marker_line());
            lines
        }
    }
}

fn render_dialogue_continuation_prefix(
    dialogue: &crate::pagination::DialogueUnit,
    geometry: &LayoutGeometry,
) -> Vec<String> {
    dialogue
        .parts
        .iter()
        .take_while(|part| matches!(part.kind, DialoguePartKind::Character))
        .flat_map(|part| {
            render_indented_lines(
                &continued_character_cue_text(&part.text),
                ElementType::Character,
                geometry,
            )
        })
        .collect()
}

fn continued_character_cue_text(text: &str) -> String {
    let trimmed = text.trim_end();
    let upper = trimmed.to_ascii_uppercase();

    if upper.ends_with("(CONT'D)") || upper.ends_with("(CONT’D)") {
        trimmed.to_string()
    } else {
        format!("{trimmed} (CONT'D)")
    }
}

fn render_more_marker_line() -> DiagnosticRenderedLine {
    DiagnosticRenderedLine {
        text: "(MORE)".to_string(),
        counted: true,
    }
}

fn render_semantic_unit_lines(unit: &SemanticUnit, geometry: &LayoutGeometry) -> Vec<String> {
    match unit {
        SemanticUnit::PageStart(_) => Vec::new(),
        SemanticUnit::Flow(flow) => {
            let element_type = ElementType::from_flow_kind(&flow.kind);
            render_indented_lines(&flow.text, element_type, geometry)
        }
        SemanticUnit::Lyric(lyric) => render_indented_lines(&lyric.text, ElementType::Lyric, geometry),
        SemanticUnit::Dialogue(dialogue) => dialogue
            .parts
            .iter()
            .flat_map(|part| {
                let element_type = ElementType::from_dialogue_part_kind(&part.kind);
                render_indented_lines(&part.text, element_type, geometry)
            })
            .collect(),
        SemanticUnit::DualDialogue(dual) => {
            let left_lines = dual
                .sides
                .iter()
                .find(|side| side.side == 1)
                .map(|side| {
                    render_dual_dialogue_side_lines(
                        &side.dialogue,
                        ElementType::DualDialogueLeft,
                        geometry,
                    )
                })
                .unwrap_or_default();
            let right_lines = dual
                .sides
                .iter()
                .find(|side| side.side == 2)
                .map(|side| {
                    render_dual_dialogue_side_lines(
                        &side.dialogue,
                        ElementType::DualDialogueRight,
                        geometry,
                    )
                })
                .unwrap_or_default();

            let right_indent = indent_spaces_for_element_type(ElementType::DualDialogueRight, geometry);
            let mut lines = Vec::new();
            for index in 0..left_lines.len().max(right_lines.len()) {
                let left = left_lines.get(index).cloned().unwrap_or_default();
                let right = right_lines.get(index).cloned().unwrap_or_default();

                if right.is_empty() {
                    lines.push(left);
                } else if left.is_empty() {
                    lines.push(format!("{:width$}{}", "", right, width = right_indent));
                } else {
                    lines.push(format!("{left:width$}{right}", width = right_indent));
                }
            }
            lines
        }
    }
}

fn render_dual_dialogue_side_lines(
    dialogue: &crate::pagination::DialogueUnit,
    element_type: ElementType,
    geometry: &LayoutGeometry,
) -> Vec<String> {
    let config = crate::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    dialogue
        .parts
        .iter()
        .flat_map(|part| crate::pagination::wrapping::wrap_text_for_element(&part.text, &config))
        .collect()
}

fn render_indented_lines(text: &str, element_type: ElementType, geometry: &LayoutGeometry) -> Vec<String> {
    let config = crate::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    let indent = " ".repeat(indent_spaces_for_element_type(element_type, geometry));
    crate::pagination::wrapping::wrap_text_for_element(text, &config)
        .into_iter()
        .map(|line| format!("{indent}{line}"))
        .collect()
}

fn indent_spaces_for_element_type(element_type: ElementType, geometry: &LayoutGeometry) -> usize {
    let left_indent_in = match element_type {
        ElementType::Action | ElementType::SceneHeading => geometry.action_left,
        ElementType::ColdOpening => geometry.cold_opening_left,
        ElementType::NewAct => geometry.new_act_left,
        ElementType::EndOfAct => geometry.end_of_act_left,
        ElementType::Character => geometry.character_left,
        ElementType::Dialogue => geometry.dialogue_left,
        ElementType::Parenthetical => geometry.parenthetical_left,
        ElementType::Transition => geometry.transition_left,
        ElementType::Lyric => geometry.lyric_left,
        ElementType::DualDialogueLeft => geometry.dual_dialogue_left_left,
        ElementType::DualDialogueRight => geometry.dual_dialogue_right_left,
    };

    ((left_indent_in - geometry.action_left) * geometry.cpi).floor() as usize
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
        let page_lines = pdf_pages.get(&page.number).map(Vec::as_slice).unwrap_or(&[]);
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
        [(start, end)] => ("exact_unique".into(), Some(end - start + 1), Some((*start, *end))),
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
    let pdf_pages: PublicPdfPages = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    pdf_pages
        .pages
        .into_iter()
        .map(|page| (page.number, page.text.lines().map(str::to_string).collect()))
        .collect()
}

fn render_big_fish_review_packet(summaries: &[ReviewSummary]) -> String {
    let focus_windows: Vec<&ReviewSummary> =
        summaries.iter().filter(|summary| summary.total_issues > 0).collect();
    let ordered_windows: Vec<&ReviewSummary> = focus_windows
        .iter()
        .copied()
        .chain(summaries.iter().filter(|summary| summary.total_issues == 0))
        .collect();

    let mut review = String::from(
        "# Big Fish Pagination Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- big-fish-review\n\
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
- `target/pagination-debug/big-fish-review/<window>.page-endings.json` compares our last meaningful line on each page against the canonical extracted PDF page ending.\n\
- `target/pagination-debug/big-fish-review/<window>.pdf-line-counts.json` gives exact-unique PDF line counts where text alignment is recoverable.\n\
- `tests/fixtures/corpus/public/big-fish/source/source.fountain` is the vendored local source text.\n\n\
Current window summary:\n\n",
    );

    for summary in summaries {
        review.push_str(&format!(
            "- `{stem}` pages {start}-{end}: lines_per_page={lines}, total={total}, wrong_page={wrong_page}, wrong_fragment={wrong_fragment}, missing={missing}, unexpected={unexpected}\n  canonical: `{fixture}`\n  actual: `target/pagination-debug/big-fish-review/{stem}.actual.page-breaks.json`\n  report: `target/pagination-debug/big-fish-review/{stem}.comparison-report.json`\n  endings: `target/pagination-debug/big-fish-review/{stem}.page-endings.json`\n  pdf: `target/pagination-debug/big-fish-review/{stem}.pdf-line-counts.json`\n",
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

fn render_little_women_review_packet(summaries: &[ReviewSummary]) -> String {
    let focus_windows: Vec<&ReviewSummary> =
        summaries.iter().filter(|summary| summary.total_issues > 0).collect();
    let ordered_windows: Vec<&ReviewSummary> = focus_windows
        .iter()
        .copied()
        .chain(summaries.iter().filter(|summary| summary.total_issues == 0))
        .collect();

    let mut review = String::from(
        "# Little Women Pagination Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- little-women-review\n\
```\n\n\
Read files in this order:\n\n\
1. `target/pagination-debug/little-women-review/REVIEW.md`\n",
    );

    for (index, summary) in ordered_windows.iter().enumerate() {
        let step = index * 3 + 2;
        review.push_str(&format!(
            "{step}. `target/pagination-debug/little-women-review/{stem}.comparison-report.json`\n\
{step_plus_one}. `{fixture}`\n\
{step_plus_two}. `target/pagination-debug/little-women-review/{stem}.actual.page-breaks.json`\n",
            step = step,
            step_plus_one = step + 1,
            step_plus_two = step + 2,
            stem = summary.stem,
            fixture = summary.fixture_path,
        ));
    }

    review.push_str(
        "\nBackground:\n\n\
- `comparison-report.json` is the quickest way to see where pages or fragments diverge.\n\
- `actual.page-breaks.json` is the current engine output in canonical fixture shape.\n\
- `page-endings.json` compares our last meaningful line on each page against the canonical extracted PDF page ending.\n\
- `pdf-line-counts.json` gives exact-unique PDF line counts where text alignment is recoverable.\n\
- `pseudo-pdf.txt` is a plain-text rendering of the current engine's predicted page lines and blank spacing.\n\
- `tests/fixtures/corpus/public/little-women/source/source.fountain` is the vendored local source text.\n\n\
Current window summary:\n\n",
    );

    for summary in summaries {
        review.push_str(&format!(
            "- `{stem}` pages {start}-{end}: lines_per_page={lines}, total={total}, wrong_page={wrong_page}, wrong_fragment={wrong_fragment}, missing={missing}, unexpected={unexpected}\n  canonical: `{fixture}`\n  actual: `target/pagination-debug/little-women-review/{stem}.actual.page-breaks.json`\n  report: `target/pagination-debug/little-women-review/{stem}.comparison-report.json`\n  endings: `target/pagination-debug/little-women-review/{stem}.page-endings.json`\n  pdf: `target/pagination-debug/little-women-review/{stem}.pdf-line-counts.json`\n  pseudo: `target/pagination-debug/little-women-review/{stem}.pseudo-pdf.txt`\n",
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
        review.push_str("\nAll selected Little Women windows currently match.\n");
    }

    review
}

fn render_little_women_full_script_review_packet(
    lines_per_page: f32,
    score: (usize, usize, usize),
    fixture: &PageBreakFixture,
    report: &crate::pagination::ComparisonReport,
) -> String {
    format!(
        "# Little Women Full-Script Page-Break Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- little-women-full-script\n\
```\n\n\
Read files in this order:\n\n\
1. `target/pagination-debug/little-women-full-script/REVIEW.md`\n\
2. `target/pagination-debug/little-women-full-script/comparison-report.json`\n\
3. `tests/fixtures/corpus/public/little-women/canonical/page-breaks.json`\n\
4. `target/pagination-debug/little-women-full-script/actual.page-breaks.json`\n\
5. `target/pagination-debug/little-women-full-script/page-endings.json`\n\
6. `target/pagination-debug/little-women-full-script/pseudo-pdf.txt`\n\n\
Current full-script summary:\n\n\
- pages: {page_count}\n\
- lines_per_page: {lines_per_page}\n\
- total issues: {total}\n\
- wrong page: {wrong_page}\n\
- wrong fragment: {wrong_fragment}\n\
- missing: {missing}\n\
- unexpected: {unexpected}\n\n\
Notes:\n\n\
- `comparison-report.json` is the quickest way to see where page assignments diverge.\n\
- `actual.page-breaks.json` is the current engine output in canonical fixture shape.\n\
- `page-endings.json` compares our last meaningful line on each page against the canonical extracted PDF page ending.\n\
- `pseudo-pdf.txt` is a plain-text rendering of the current engine's predicted page lines and blank spacing.\n\
- `tests/fixtures/corpus/public/little-women/source/source.fountain` is the vendored local source text.\n",
        page_count = fixture.pages.len(),
        lines_per_page = lines_per_page,
        total = report.total_issues(),
        wrong_page = score.1,
        wrong_fragment = score.2,
        missing = report.issue_count(ComparisonIssueKind::MissingOccurrence),
        unexpected = report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
    )
}

fn render_big_fish_full_script_review_packet(
    lines_per_page: f32,
    _score: (usize, usize, usize),
    fixture: &PageBreakFixture,
    report: &crate::pagination::ComparisonReport,
) -> String {
    format!(
        "# Big Fish Full-Script Page-Break Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- big-fish-full-script\n\
```\n\n\
Read files in this order:\n\n\
1. `target/pagination-debug/big-fish-full-script/REVIEW.md`\n\
2. `target/pagination-debug/big-fish-full-script/comparison-report.json`\n\
3. `tests/fixtures/corpus/public/big-fish/canonical/page-breaks.json`\n\
4. `target/pagination-debug/big-fish-full-script/actual.page-breaks.json`\n\
5. `target/pagination-debug/big-fish-full-script/page-endings.json`\n\
6. `target/pagination-debug/big-fish-full-script/pseudo-pdf.txt`\n\n\
Current full-script summary:\n\n\
- pages: {pages}\n\
- lines_per_page: {lines_per_page}\n\
- total issues: {total}\n\
- wrong page: {wrong_page}\n\
- wrong fragment: {wrong_fragment}\n\
- missing: {missing}\n\
- unexpected: {unexpected}\n\n\
Notes:\n\n\
- `comparison-report.json` is the quickest way to see where page assignments diverge.\n\
- `actual.page-breaks.json` is the current engine output in canonical fixture shape.\n\
- `page-endings.json` compares our last meaningful line on each page against the canonical extracted PDF page ending.\n\
- `pseudo-pdf.txt` is a plain-text rendering of the current engine's predicted page lines and blank spacing.\n\
- `tests/fixtures/corpus/public/big-fish/source/source.fountain` is the vendored local source text.\n",
        pages = fixture.pages.len(),
        lines_per_page = lines_per_page,
        total = report.total_issues(),
        wrong_page = report.issue_count(ComparisonIssueKind::WrongPage),
        wrong_fragment = report.issue_count(ComparisonIssueKind::WrongFragment),
        missing = report.issue_count(ComparisonIssueKind::MissingOccurrence),
        unexpected = report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
    )
}

fn render_mostly_genius_full_script_review_packet(
    lines_per_page: f32,
    score: (usize, usize, usize),
    fixture: &PageBreakFixture,
    report: &crate::pagination::ComparisonReport,
) -> String {
    format!(
        "# Mostly Genius Full-Script Page-Break Review Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- mostly-genius-full-script\n\
```\n\n\
Read files in this order:\n\n\
1. `target/pagination-debug/mostly-genius-full-script/REVIEW.md`\n\
2. `target/pagination-debug/mostly-genius-full-script/comparison-report.json`\n\
3. `tests/fixtures/corpus/public/mostly-genius/canonical/page-breaks.json`\n\
4. `target/pagination-debug/mostly-genius-full-script/actual.page-breaks.json`\n\
5. `target/pagination-debug/mostly-genius-full-script/page-endings.json`\n\
6. `target/pagination-debug/mostly-genius-full-script/pseudo-pdf.txt`\n\n\
Current full-script summary:\n\n\
- pages: {page_count}\n\
- lines_per_page: {lines_per_page}\n\
- total issues: {total}\n\
- wrong page: {wrong_page}\n\
- wrong fragment: {wrong_fragment}\n\
- missing: {missing}\n\
- unexpected: {unexpected}\n\n\
Notes:\n\n\
- This is the sanitized multicam sample, so the main things to inspect first are line-height drift, act-marker handling, and any multicam-specific pacing differences.\n\
- `comparison-report.json` is the quickest way to see where page assignments diverge.\n\
- `actual.page-breaks.json` is the current engine output in canonical fixture shape.\n\
- `page-endings.json` compares our last meaningful line on each page against the canonical extracted PDF page ending.\n\
- `pseudo-pdf.txt` is a plain-text rendering of the current engine's predicted page lines and blank spacing.\n\
- `tests/fixtures/corpus/public/mostly-genius/source/source.fountain` is the vendored local source text.\n",
        page_count = fixture.pages.len(),
        lines_per_page = lines_per_page,
        total = report.total_issues(),
        wrong_page = score.1,
        wrong_fragment = score.2,
        missing = report.issue_count(ComparisonIssueKind::MissingOccurrence),
        unexpected = report.issue_count(ComparisonIssueKind::UnexpectedOccurrence),
    )
}

fn render_fd_probe_review(summaries: &[FdProbeSummary]) -> String {
    let mut review = String::from(
        "# Final Draft Probe Packet\n\n\
Run this command to regenerate everything in this folder:\n\n\
```bash\n\
cargo run --bin pagination-diagnostics -- fd-probes\n\
```\n\n\
Each probe folder contains:\n\n\
- `pseudo-pdf.txt`: current engine output for the probe\n\
- `actual-observation.json`: the target-matching blocks the engine actually produced\n\
- the source fixture in `tests/fixtures/fd-probes/<probe>/`\n\n\
Probe folders:\n\n",
    );

    for summary in summaries {
        review.push_str(&format!(
            "- `{folder}` (`{probe_id}`, status: `{status}`)\n  fixture: `../../../tests/fixtures/fd-probes/{folder}`\n  pseudo: `{folder}/pseudo-pdf.txt`\n  actual: `{folder}/actual-observation.json`\n",
            folder = summary.folder,
            probe_id = summary.probe_id,
            status = match summary.status {
                ProbeStatus::Draft => "draft",
                ProbeStatus::Active => "active",
            }
        ));
    }

    review
}

fn text_preview(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(80).collect()
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

struct MeasuredItem {
    content_lines: f32,
    spacing_above: f32,
    width_chars: usize,
    wrapped_lines: Vec<String>,
}

fn measured_lines_for_item(
    item: &crate::pagination::PageItem,
    elements: &HashMap<String, NormalizedElement>,
    geometry: &LayoutGeometry,
) -> MeasuredItem {
    let Some(element) = elements.get(&item.element_id) else {
        return MeasuredItem {
            content_lines: 0.0,
            spacing_above: 0.0,
            width_chars: 0,
            wrapped_lines: Vec::new(),
        };
    };

    let element_type = crate::pagination::wrapping::ElementType::from_item_kind(
        &item.kind,
        item.dual_dialogue_side,
    );

    let config = crate::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    let text = match item.line_range {
        Some((start, end)) => slice_explicit_lines(&element.text, start, end),
        None => element.text.clone(),
    };
    let wrapped_lines = crate::pagination::wrapping::wrap_text_for_element(&text, &config);
    let content_lines = wrapped_lines.len() as f32 * geometry.line_height;

    let spacing_above = match element_type {
        ElementType::Action => geometry.action_spacing_before,
        ElementType::SceneHeading => geometry.scene_heading_spacing_before,
        ElementType::Character => geometry.character_spacing_before,
        ElementType::Transition => geometry.transition_spacing_before,
        ElementType::Lyric => geometry.lyric_spacing_before,
        _ => 1.0,
    };

    MeasuredItem {
        content_lines,
        spacing_above,
        width_chars: config.exact_width_chars,
        wrapped_lines,
    }
}

fn measure_visual_item(
    element_id: &str,
    kind: &str,
    line_range: Option<(u32, u32)>,
    dual_dialogue_side: Option<u8>,
    elements: &HashMap<String, NormalizedElement>,
    geometry: &LayoutGeometry,
) -> (f32, f32) {
    let Some(element) = elements.get(element_id) else {
        return (0.0, 0.0);
    };

    let element_type = ElementType::from_item_kind(kind, dual_dialogue_side);
    let config = crate::pagination::wrapping::WrapConfig::from_geometry(geometry, element_type);
    let text = match line_range {
        Some((start, end)) => slice_explicit_lines(&element.text, start, end),
        None => element.text.clone(),
    };
    let wrapped_lines = crate::pagination::wrapping::wrap_text_for_element(&text, &config);
    let content_lines = wrapped_lines.len() as f32 * geometry.line_height;
    let spacing_above = match element_type {
        ElementType::Action => geometry.action_spacing_before,
        ElementType::SceneHeading => geometry.scene_heading_spacing_before,
        ElementType::Character => geometry.character_spacing_before,
        ElementType::Transition => geometry.transition_spacing_before,
        ElementType::Lyric => geometry.lyric_spacing_before,
        _ => 1.0,
    };

    (content_lines, spacing_above)
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
    report: crate::pagination::ComparisonReport,
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
    report: crate::pagination::ComparisonReport,
}

struct ReviewSummary {
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

struct PaginatedWindowRun {
    lines_per_page: f32,
    score: (usize, usize, usize),
    actual: PaginatedScreenplay,
    geometry: LayoutGeometry,
    report: crate::pagination::ComparisonReport,
}

#[derive(Serialize)]
struct DebugPageBreakFixture {
    screenplay: String,
    style_profile: String,
    source: PageBreakFixtureSourceRefs,
    scope: crate::pagination::PaginationScope,
    lines_per_page: f32,
    measurement: DebugMeasurement,
    pages: Vec<DebugPageBreakFixturePage>,
}

#[derive(Serialize)]
struct DebugPageBreakFixturePage {
    number: u32,
    block_total_lines: f32,
    item_count: usize,
    block_count: usize,
    items: Vec<DebugPageBreakItem>,
}

#[derive(Serialize)]
struct DebugPageBreakItem {
    element_id: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    measured_lines: f32,
    intrinsic_spacing_before_lines: f32,
    width_chars: usize,
    wrapped_lines: Vec<String>,
    fragment: Fragment,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_range: Option<(u32, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dual_dialogue_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Deserialize)]
struct PublicPdfPages {
    pages: Vec<PublicPdfPage>,
}

#[derive(Deserialize)]
struct PublicPdfPage {
    number: u32,
    text: String,
}
