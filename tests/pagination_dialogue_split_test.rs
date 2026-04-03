use jumpcut::pagination::dialogue_split::{
    plan_dialogue_split, plan_dialogue_split_parts, DialogueTextPart,
};
use jumpcut::pagination::{Cohesion, DialoguePart, DialoguePartKind, DialogueUnit, LayoutGeometry};

#[test]
fn dialogue_split_prefers_a_sentence_boundary_for_the_mayor_case() {
    let dialogue = DialogueUnit {
        block_id: "block-mayor".into(),
        parts: vec![
            DialoguePart {
                element_id: "el-mayor-character".into(),
                kind: DialoguePartKind::Character,
                text: "MAYOR".into(),
                inline_text: None,
                render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
            },
            DialoguePart {
                element_id: "el-mayor-parenthetical".into(),
                kind: DialoguePartKind::Parenthetical,
                text: "(loudly, for the crowd)".into(),
                inline_text: None,
                render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
            },
            DialoguePart {
                element_id: "el-mayor-dialogue".into(),
                kind: DialoguePartKind::Dialogue,
                text: "Edward Bloom, first son of Ashton, it's with a heavy heart we see you go. But take with you this Key to the City, and know that any time you want to come back, all our doors are open to you.".into(),
                inline_text: None,
                render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
            },
        ],
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    };

    let plan = plan_dialogue_split(&dialogue, &LayoutGeometry::default(), 5.0, 2, 2).unwrap();

    assert!(plan.ends_sentence);
    assert_eq!(
        plan.parts[2].top_text.trim_end(),
        "Edward Bloom, first son of Ashton, it's with a heavy heart we see you go."
    );
    assert_eq!(
        plan.parts[2].bottom_text.trim_start(),
        "But take with you this Key to the City, and know that any time you want to come back, all our doors are open to you."
    );
}

#[test]
fn dialogue_split_pushes_the_whole_block_when_only_a_too_short_top_fragment_fits() {
    let parts = dialogue_parts(&[
        (DialoguePartKind::Character, "MAYOR"),
        (DialoguePartKind::Dialogue, "Go."),
        (
            DialoguePartKind::Dialogue,
            "This farewell speech continues for several more lines",
        ),
        (
            DialoguePartKind::Dialogue,
            "so that the block has room to split in more than",
        ),
        (
            DialoguePartKind::Dialogue,
            "one possible place when the page boundary hits.",
        ),
    ]);

    let plan =
        plan_dialogue_split_parts(&stub_dialogue_unit(), &parts, &LayoutGeometry::default(), 2.0, 2, 2);

    assert_eq!(plan, None);
}

#[test]
fn dialogue_split_can_start_the_continuation_with_a_parenthetical() {
    let parts = dialogue_parts(&[
        (DialoguePartKind::Character, "MAYOR"),
        (DialoguePartKind::Dialogue, "Edward Bloom, first son of Ashton,"),
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

    let plan =
        plan_dialogue_split_parts(&stub_dialogue_unit(), &parts, &LayoutGeometry::default(), 6.0, 2, 2)
            .unwrap();

    assert_eq!(plan.top_line_count, 5);
}

#[test]
fn dialogue_split_prefers_the_sentence_boundary_that_also_fills_the_page() {
    let mut parts = dialogue_parts(&[
        (DialoguePartKind::Character, "EDWARD (CONT'D)"),
        (
            DialoguePartKind::Dialogue,
            "For the next couple weeks, I didn't",
        ),
        (
            DialoguePartKind::Dialogue,
            "have another dream.  Until one",
        ),
        (
            DialoguePartKind::Dialogue,
            "night the crow came back and said,",
        ),
        (
            DialoguePartKind::Dialogue,
            "\"Your Daddy is going to die.\"",
        ),
        (DialoguePartKind::Parenthetical, "(beat)"),
    ]);
    parts.extend(
        [
            "Well, I didn't know what to do.",
            "But finally I told my father.  And",
            "he said not to worry, but I could",
            "tell he was rattled.  That next",
            "day, he wasn't himself, always",
            "looking around, waiting for",
            "something to drop on his head.",
            "Because the crow didn't tell how it",
            "was going to happen, just those",
            "words:  your Daddy is going to die.",
            "Well, he went into town early and",
            "was gone for a long time.",
            "And when he finally came back, he looked",
            "terrible, like he was waiting for",
            "the axe to fall all day.  He said",
            "to my mother, \"Good God.  I just",
            "had the worst day of my life.\"",
        ]
        .into_iter()
        .map(|text| DialogueTextPart {
            kind: DialoguePartKind::Dialogue,
            text: text.into(),
        }),
    );

    let plan =
        plan_dialogue_split_parts(&stub_dialogue_unit(), &parts, &LayoutGeometry::default(), 19.0, 2, 2)
            .unwrap();

    assert_eq!(plan.top_line_count, 18);
    assert_eq!(parts[plan.top_line_count - 1].text, "was gone for a long time.");
    assert_eq!(
        parts[plan.top_line_count].text,
        "And when he finally came back, he looked"
    );
}

#[test]
fn dialogue_split_plan_can_split_at_a_sentence_boundary_inside_a_wrapped_line() {
    let dialogue = DialogueUnit {
        block_id: "block-01146".into(),
        parts: vec![
            DialoguePart {
                element_id: "el-01145".into(),
                kind: DialoguePartKind::Character,
                text: "EDWARD (CONT'D)".into(),
                inline_text: None,
                render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
            },
            DialoguePart {
                element_id: "el-01146".into(),
                kind: DialoguePartKind::Dialogue,
                text: "Well, I didn't know what to do.  But finally I told my father.  And he said not to worry, but I could tell he was rattled.  That next day, he wasn't himself, always looking around, waiting for something to drop on his head.  Because the crow didn't tell how it was going to happen, just those words:  your Daddy is going to die.  Well, he went into town early and was gone for a long time.  And when he finally came back, he looked terrible, like he was waiting for the axe to fall all day.  He said to my mother, \"Good God.  I just had the worst day of my life.\"".into(),
                inline_text: None,
                render_attributes: jumpcut::render_attributes::RenderAttributes::default(),
            },
        ],
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    };

    let plan = plan_dialogue_split(&dialogue, &LayoutGeometry::default(), 14.0, 2, 2).unwrap();

    assert_eq!(plan.top_line_count, 13);
    assert_eq!(plan.bottom_line_count, 6);
    assert_eq!(plan.parts[1].top_lines.len(), 12);
    assert_eq!(plan.parts[1].bottom_lines.len(), 6);
    assert_eq!(plan.parts[1].top_end_offset, plan.parts[1].top_text.len());
    assert_eq!(plan.parts[1].bottom_start_offset, plan.parts[1].top_end_offset);
    assert_eq!(
        plan.parts[1].top_text.trim_end(),
        "Well, I didn't know what to do.  But finally I told my father.  And he said not to worry, but I could tell he was rattled.  That next day, he wasn't himself, always looking around, waiting for something to drop on his head.  Because the crow didn't tell how it was going to happen, just those words:  your Daddy is going to die.  Well, he went into town early and was gone for a long time."
    );
    assert!(
        plan.parts[1]
            .bottom_text
            .trim_start()
            .starts_with("And when he finally came back, he looked"),
        "expected continuation text to start at the next sentence boundary, got: {:?}",
        plan.parts[1].bottom_text
    );
}

fn dialogue_parts(parts: &[(DialoguePartKind, &str)]) -> Vec<DialogueTextPart> {
    parts
        .iter()
        .map(|(kind, text)| DialogueTextPart {
            kind: kind.clone(),
            text: (*text).into(),
        })
        .collect()
}

fn stub_dialogue_unit() -> DialogueUnit {
    DialogueUnit {
        block_id: "stub-block".into(),
        parts: Vec::new(),
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    }
}
