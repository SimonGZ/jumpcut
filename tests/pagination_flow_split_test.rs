use jumpcut::pagination::flow_split::choose_flow_split;

#[test]
fn flow_split_prefers_a_sentence_boundary_when_a_legal_split_exists() {
    let wrapped_lines = vec![
        "A forty-year old man named BEAMEN comes out of the seed store".to_string(),
        "to greet Edward.".to_string(),
        "Friendly but a little drunk, he's the closest thing the town".to_string(),
        "has to a mayor. He's carrying a clipboard.".to_string(),
    ];

    let decision = choose_flow_split(&wrapped_lines, 3, 2, 2).unwrap();

    assert_eq!(decision.top_line_count, 2);
}
