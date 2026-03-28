use jumpcut::pagination::dialogue_split::{
    choose_dialogue_split, DialogueLine, DialogueLineRole, DialogueSplitDecision,
};

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

    let split = choose_dialogue_split(&lines, 4, 2, 2);

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

    let split = choose_dialogue_split(&lines, 5, 2, 2);

    assert_eq!(split, Some(DialogueSplitDecision { top_line_count: 5 }));
}
