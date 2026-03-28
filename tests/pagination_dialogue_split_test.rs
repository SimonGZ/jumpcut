use jumpcut::pagination::dialogue_split::{
    choose_dialogue_split, plan_dialogue_split, DialogueLine, DialogueLineRole,
    DialogueSplitDecision,
};
use jumpcut::pagination::{Cohesion, DialoguePart, DialoguePartKind, DialogueUnit, LayoutGeometry};

#[test]
fn dialogue_split_prefers_a_sentence_boundary_for_the_mayor_case() {
    let lines = vec![
        DialogueLine { role: DialogueLineRole::Character, text: "MAYOR".into() },
        DialogueLine { role: DialogueLineRole::Parenthetical, text: "(loudly, for the crowd)".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "Edward Bloom, first son of Ashton,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "it's with a heavy heart we see you go.".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "But take with you this Key to the City,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "and know that any time you want to come".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "back, all our doors are open to you.".into() },
    ];

    let split = choose_dialogue_split(&lines, 5, 2, 2);

    assert_eq!(split, Some(DialogueSplitDecision { top_line_count: 4 }));
}

#[test]
fn dialogue_split_pushes_the_whole_block_when_only_a_too_short_top_fragment_fits() {
    let lines = vec![
        DialogueLine { role: DialogueLineRole::Character, text: "MAYOR".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "Go.".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "This farewell speech continues for several more lines".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "so that the block has room to split in more than".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "one possible place when the page boundary hits.".into() },
    ];

    let split = choose_dialogue_split(&lines, 2, 2, 2);

    assert_eq!(split, None);
}

#[test]
fn dialogue_split_can_start_the_continuation_with_a_parenthetical() {
    let lines = vec![
        DialogueLine { role: DialogueLineRole::Character, text: "MAYOR".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "Edward Bloom, first son of Ashton,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "you have always been too big for this town,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "and everyone here knows it.".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "Horses don't like to dance much, Daniel.".into() },
        DialogueLine { role: DialogueLineRole::Parenthetical, text: "(quietly)".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "But if you ever do come back, you will find".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "every porch light burning, every front door open,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "and every one of us waiting to see what story".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "you bring home next.".into() },
    ];

    let split = choose_dialogue_split(&lines, 6, 2, 2);

    assert_eq!(split, Some(DialogueSplitDecision { top_line_count: 5 }));
}

#[test]
fn dialogue_split_prefers_the_sentence_boundary_that_also_fills_the_page() {
    let mut lines = vec![
        DialogueLine { role: DialogueLineRole::Character, text: "EDWARD (CONT'D)".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "For the next couple weeks, I didn't".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "have another dream.  Until one".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "night the crow came back and said,".into() },
        DialogueLine { role: DialogueLineRole::Dialogue, text: "\"Your Daddy is going to die.\"".into() },
        DialogueLine { role: DialogueLineRole::Parenthetical, text: "(beat)".into() },
    ];
    lines.extend([
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
    .map(|text| DialogueLine { role: DialogueLineRole::Dialogue, text: text.into() }));

    let split = choose_dialogue_split(&lines, 19, 2, 2);

    let decision = split.unwrap();
    assert_eq!(decision, DialogueSplitDecision { top_line_count: 18 });
    assert_eq!(lines[decision.top_line_count - 1].text, "was gone for a long time.");
    assert_eq!(
        lines[decision.top_line_count].text,
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
            },
            DialoguePart {
                element_id: "el-01146".into(),
                kind: DialoguePartKind::Dialogue,
                text: "Well, I didn't know what to do.  But finally I told my father.  And he said not to worry, but I could tell he was rattled.  That next day, he wasn't himself, always looking around, waiting for something to drop on his head.  Because the crow didn't tell how it was going to happen, just those words:  your Daddy is going to die.  Well, he went into town early and was gone for a long time.  And when he finally came back, he looked terrible, like he was waiting for the axe to fall all day.  He said to my mother, \"Good God.  I just had the worst day of my life.\"".into(),
            },
        ],
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    };

    let plan = plan_dialogue_split(&dialogue, &LayoutGeometry::default(), 14, 2, 2).unwrap();

    assert_eq!(plan.top_line_count, 13);
    assert_eq!(plan.bottom_line_count, 6);
    assert_eq!(plan.parts[1].top_lines.len(), 12);
    assert_eq!(plan.parts[1].bottom_lines.len(), 6);
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
