//! Paired contracts for behavior that intentionally differs by pagination profile.
//!
//! Shared pagination mechanics belong in the focused composer, paginator, wrapping,
//! and semantic test modules. Add a test here when Jumpcut and Industry are expected
//! to make different decisions for the same screenplay input.

use jumpcut::pagination::dialogue_split::{DialogueTextPart, plan_dialogue_split_parts};
use jumpcut::pagination::wrapping::{WrapConfig, wrap_text_for_element};
use jumpcut::pagination::{
    Cohesion, DialoguePartKind, DialogueUnit, LayoutGeometry, PaginationPolicy, SemanticOptions,
    SemanticUnit, build_semantic_screenplay_with_options, normalize_screenplay,
};
use jumpcut::parse;

#[test]
fn profiles_wrap_interruption_dashes_differently() {
    let text = "everything you said was impossible -- everything! -- I felt like such a fool";
    let jumpcut = PaginationPolicy::jumpcut();
    let industry = PaginationPolicy::industry();

    let jumpcut_lines = wrap_text_for_element(
        text,
        &WrapConfig {
            exact_width_chars: 35,
            interruption_dash_wrap: jumpcut.interruption_dash_wrap,
        },
    );
    let industry_lines = wrap_text_for_element(
        text,
        &WrapConfig {
            exact_width_chars: 35,
            interruption_dash_wrap: industry.interruption_dash_wrap,
        },
    );

    assert_eq!(
        jumpcut_lines,
        vec![
            "everything you said was impossible",
            "-- everything! -- I felt like such",
            "a fool",
        ]
    );
    assert_eq!(
        industry_lines,
        vec![
            "everything you said was impossible -",
            "- everything! -- I felt like such a",
            "fool",
        ]
    );
}

#[test]
fn profiles_treat_parenthetical_page_starts_differently() {
    let parts = dialogue_parts(&[
        (DialoguePartKind::Character, "MAYOR"),
        (
            DialoguePartKind::Dialogue,
            "Edward Bloom, first son of Ashton,",
        ),
        (
            DialoguePartKind::Dialogue,
            "you have always been too big for this town,",
        ),
        (DialoguePartKind::Dialogue, "and everyone here knows it."),
        (
            DialoguePartKind::Dialogue,
            "Horses don't like to dance much, Daniel.",
        ),
        (DialoguePartKind::Parenthetical, "(quietly)"),
        (
            DialoguePartKind::Dialogue,
            "But if you ever do come back, you will find",
        ),
        (
            DialoguePartKind::Dialogue,
            "every porch light burning, every front door open,",
        ),
        (
            DialoguePartKind::Dialogue,
            "and every one of us waiting to see what story",
        ),
        (DialoguePartKind::Dialogue, "you bring home next."),
    ]);
    let dialogue = stub_dialogue_unit();
    let geometry = LayoutGeometry::default();
    let jumpcut = PaginationPolicy::jumpcut();
    let industry = PaginationPolicy::industry();

    let jumpcut_plan = plan_dialogue_split_parts(
        &dialogue,
        &parts,
        &geometry,
        jumpcut.interruption_dash_wrap,
        7.0,
        jumpcut.orphan_limit,
        jumpcut.widow_limit,
        jumpcut.avoid_page_start_parenthetical,
    )
    .unwrap();
    let industry_plan = plan_dialogue_split_parts(
        &dialogue,
        &parts,
        &geometry,
        industry.interruption_dash_wrap,
        7.0,
        industry.orphan_limit,
        industry.widow_limit,
        industry.avoid_page_start_parenthetical,
    )
    .unwrap();

    assert_ne!(
        first_continuation_kind(&parts, &jumpcut_plan),
        Some(&DialoguePartKind::Parenthetical)
    );
    assert_eq!(
        first_continuation_kind(&parts, &industry_plan),
        Some(&DialoguePartKind::Parenthetical)
    );
}

#[test]
fn profiles_treat_dual_dialogue_as_different_continuation_context() {
    let screenplay = parse(
        "INT. CARRIAGE - DAY\n\nLAURIE\nHello there.\n\nAMY ^\nHi.\n\nLAURIE\nStill talking.\n\nAMY\nMe too.",
    );
    let jumpcut = PaginationPolicy::jumpcut();
    let industry = PaginationPolicy::industry();

    let jumpcut_flags = dialogue_continuation_flags(
        &screenplay,
        "jumpcut-dual-contd",
        SemanticOptions::from_policy(&jumpcut),
    );
    let industry_flags = dialogue_continuation_flags(
        &screenplay,
        "industry-dual-contd",
        SemanticOptions::from_policy(&industry),
    );

    assert_eq!(jumpcut_flags, vec![false, false]);
    assert_eq!(industry_flags, vec![true, false]);
}

fn dialogue_continuation_flags(
    screenplay: &jumpcut::Screenplay,
    screenplay_id: &str,
    options: SemanticOptions,
) -> Vec<bool> {
    build_semantic_screenplay_with_options(normalize_screenplay(screenplay_id, screenplay), options)
        .units
        .iter()
        .filter_map(|unit| match unit {
            SemanticUnit::Dialogue(dialogue) => Some(dialogue.should_append_contd),
            _ => None,
        })
        .collect()
}

fn first_continuation_kind<'a>(
    parts: &'a [DialogueTextPart],
    plan: &jumpcut::pagination::dialogue_split::DialogueSplitPlan,
) -> Option<&'a DialoguePartKind> {
    parts
        .iter()
        .zip(plan.parts.iter())
        .find(|(_, split)| !split.bottom_text.is_empty())
        .map(|(part, _)| &part.kind)
}

fn dialogue_parts(parts: &[(DialoguePartKind, &str)]) -> Vec<DialogueTextPart> {
    parts
        .iter()
        .map(|(kind, text)| DialogueTextPart {
            kind: kind.clone(),
            text: (*text).into(),
            layout_overrides: Default::default(),
        })
        .collect()
}

fn stub_dialogue_unit() -> DialogueUnit {
    DialogueUnit {
        block_id: "profile-contract".into(),
        parts: Vec::new(),
        should_append_contd: false,
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    }
}
