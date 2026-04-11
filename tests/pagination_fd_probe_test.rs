use jumpcut::pagination::{build_semantic_screenplay, Fragment, PaginationConfig};
use jumpcut::parse;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum ProbeStatus {
    Draft,
    Active,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum ProbeTargetKind {
    Dialogue,
    Flow,
}

#[derive(Debug, Deserialize)]
struct ProbeTarget {
    kind: ProbeTargetKind,
    contains_text: String,
    #[serde(default)]
    speaker: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ProbeExpectation {
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

#[derive(Debug, Deserialize)]
struct FinalDraftProbe {
    probe_id: String,
    description: String,
    status: ProbeStatus,
    lines_per_page: f32,
    target: ProbeTarget,
    expected: ProbeExpectation,
    #[serde(default)]
    final_draft_notes: Vec<String>,
}

#[derive(Debug)]
struct ProbeFixture {
    dir: PathBuf,
    spec: FinalDraftProbe,
}

#[test]
fn final_draft_probe_fixtures_parse() {
    let probes = load_probe_fixtures();
    assert!(
        !probes.is_empty(),
        "expected at least one probe fixture under tests/fixtures/fd-probes"
    );

    for probe in probes {
        assert!(
            !probe.spec.probe_id.trim().is_empty(),
            "{}: probe_id must not be empty",
            probe.dir.display()
        );
        assert!(
            !probe.spec.description.trim().is_empty(),
            "{}: description must not be empty",
            probe.dir.display()
        );
        assert!(
            probe.spec.lines_per_page > 0.0,
            "{}: lines_per_page must be positive",
            probe.dir.display()
        );
        assert!(
            !probe.spec.target.contains_text.trim().is_empty(),
            "{}: target.contains_text must not be empty",
            probe.dir.display()
        );
        if probe.spec.status == ProbeStatus::Active {
            assert!(
                !probe.spec.final_draft_notes.is_empty(),
                "{}: active probes should include at least one Final Draft note",
                probe.dir.display()
            );
        }
    }
}

#[test]
fn active_final_draft_probes_match_expected_split_behavior() {
    for probe in load_probe_fixtures()
        .into_iter()
        .filter(|probe| probe.spec.status == ProbeStatus::Active)
    {
        assert_probe_matches(&probe);
    }
}

fn assert_probe_matches(probe: &ProbeFixture) {
    let fountain_path = probe.dir.join("source.fountain");
    let fountain = fs::read_to_string(&fountain_path).unwrap();
    let screenplay = parse(&fountain);
    let config = PaginationConfig::from_screenplay(&screenplay, probe.spec.lines_per_page);
    let actual = jumpcut::pagination::PaginatedScreenplay::from_screenplay(
        &probe.spec.probe_id,
        &screenplay,
        probe.spec.lines_per_page,
        jumpcut::pagination::PaginationScope {
            title_page_count: None,
            body_start_page: None,
        },
    );
    let semantic = build_semantic_screenplay(jumpcut::pagination::normalize_screenplay(
        &probe.spec.probe_id,
        &screenplay,
    ));
    let composed_blocks = jumpcut::pagination::composer::compose(&semantic.units, &config.geometry);
    let layout_pages = jumpcut::pagination::paginator::paginate(
        &composed_blocks,
        probe.spec.lines_per_page,
        &config.geometry,
    );

    match &probe.spec.expected {
        ProbeExpectation::Split {
            top_page,
            bottom_page,
            top_fragment_ends_with,
            bottom_fragment_starts_with,
        } => {
            let top_block = find_matching_block(
                &probe.spec,
                &actual,
                &layout_pages,
                *top_page,
                Fragment::ContinuedToNext,
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}: expected target block on page {} with fragment ContinuedToNext",
                    probe.dir.display(),
                    top_page
                )
            });
            let bottom_block = find_matching_block(
                &probe.spec,
                &actual,
                &layout_pages,
                *bottom_page,
                Fragment::ContinuedFromPrev,
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}: expected target block on page {} with fragment ContinuedFromPrev",
                    probe.dir.display(),
                    bottom_page
                )
            });

            assert!(
                top_block.trim_end().ends_with(top_fragment_ends_with),
                "{}: expected top fragment to end with {:?}, got {:?}",
                probe.dir.display(),
                top_fragment_ends_with,
                top_block
            );
            assert!(
                bottom_block
                    .trim_start()
                    .starts_with(bottom_fragment_starts_with),
                "{}: expected bottom fragment to start with {:?}, got {:?}",
                probe.dir.display(),
                bottom_fragment_starts_with,
                bottom_block
            );
        }
        ProbeExpectation::PushWhole {
            absent_from_page,
            whole_on_page,
            starts_with,
        } => {
            let absent = find_matching_block(
                &probe.spec,
                &actual,
                &layout_pages,
                *absent_from_page,
                Fragment::ContinuedToNext,
            )
            .or_else(|| {
                find_matching_block(
                    &probe.spec,
                    &actual,
                    &layout_pages,
                    *absent_from_page,
                    Fragment::Whole,
                )
            });
            assert!(
                absent.is_none(),
                "{}: expected target block to be absent from page {}, got {:?}",
                probe.dir.display(),
                absent_from_page,
                absent
            );

            let whole_block = find_matching_block(
                &probe.spec,
                &actual,
                &layout_pages,
                *whole_on_page,
                Fragment::Whole,
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}: expected target block to stay whole on page {}",
                    probe.dir.display(),
                    whole_on_page
                )
            });

            assert!(
                whole_block.trim_start().starts_with(starts_with),
                "{}: expected whole block to start with {:?}, got {:?}",
                probe.dir.display(),
                starts_with,
                whole_block
            );
        }
    }
}

fn find_matching_block(
    spec: &FinalDraftProbe,
    actual: &jumpcut::pagination::PaginatedScreenplay,
    layout_pages: &[jumpcut::pagination::paginator::Page<'_>],
    page_number: u32,
    fragment: Fragment,
) -> Option<String> {
    actual
        .pages
        .iter()
        .zip(layout_pages.iter())
        .find(|(page, _)| page.metadata.number == page_number)
        .and_then(|(_, layout_page)| {
            layout_page
                .blocks
                .iter()
                .find(|block| block.fragment == fragment && block_matches_target(block, spec))
                .map(|block| rendered_block_text(block, &spec.target.kind))
        })
}

fn block_matches_target(
    block: &jumpcut::pagination::composer::LayoutBlock<'_>,
    spec: &FinalDraftProbe,
) -> bool {
    match (spec.target.kind, block.unit) {
        (ProbeTargetKind::Dialogue, jumpcut::pagination::SemanticUnit::Dialogue(dialogue)) => {
            let text = dialogue
                .parts
                .iter()
                .map(|part| part.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let speaker_matches = spec.target.speaker.as_ref().is_none_or(|speaker| {
                dialogue.parts.iter().any(|part| {
                    matches!(part.kind, jumpcut::pagination::DialoguePartKind::Character)
                        && part.text.trim() == speaker.trim()
                })
            });
            speaker_matches && text.contains(&spec.target.contains_text)
        }
        (ProbeTargetKind::Flow, jumpcut::pagination::SemanticUnit::Flow(flow)) => {
            flow.text.contains(&spec.target.contains_text)
        }
        _ => false,
    }
}

fn rendered_block_text(
    block: &jumpcut::pagination::composer::LayoutBlock<'_>,
    target_kind: &ProbeTargetKind,
) -> String {
    match target_kind {
        ProbeTargetKind::Dialogue => {
            let jumpcut::pagination::SemanticUnit::Dialogue(dialogue) = block.unit else {
                panic!("expected dialogue block");
            };

            if matches!(block.fragment, Fragment::Whole) {
                return dialogue
                    .parts
                    .iter()
                    .filter(|part| {
                        !matches!(part.kind, jumpcut::pagination::DialoguePartKind::Character)
                    })
                    .map(|part| part.text.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
            }

            let plan = block.dialogue_split.as_ref().expect(&format!(
                "expected dialogue split metadata for probe: {:?}",
                block.unit
            ));

            dialogue
                .parts
                .iter()
                .zip(plan.parts.iter())
                .map(|(_part, part_plan)| match block.fragment {
                    Fragment::Whole => unreachable!(),
                    Fragment::ContinuedToNext => part_plan.top_text.clone(),
                    Fragment::ContinuedFromPrev => part_plan.bottom_text.clone(),
                    Fragment::ContinuedFromPrevAndToNext => part_plan.top_text.clone(),
                })
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
        ProbeTargetKind::Flow => {
            let jumpcut::pagination::SemanticUnit::Flow(flow) = block.unit else {
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

fn load_probe_fixtures() -> Vec<ProbeFixture> {
    let root = Path::new("tests/fixtures/fd-probes");
    let mut fixtures = fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.join("expected.json").exists())
        .map(|dir| ProbeFixture {
            spec: serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).unwrap())
                .unwrap_or_else(|err| panic!("{}: {err}", dir.display())),
            dir,
        })
        .collect::<Vec<_>>();
    fixtures.sort_by(|a, b| a.spec.probe_id.cmp(&b.spec.probe_id));
    fixtures
}
