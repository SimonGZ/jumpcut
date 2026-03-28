use jumpcut::pagination::flow_split::choose_flow_split;
use jumpcut::pagination::wrapping::WrapConfig;

#[test]
fn flow_split_prefers_a_sentence_boundary_when_a_legal_split_exists() {
    let text = "A forty-year old man named BEAMEN comes out of the seed store to greet Edward.  Friendly but a little drunk, he's the closest thing the town has to a mayor. He's carrying a clipboard.";
    let config = WrapConfig::with_exact_width_chars(61);

    let decision = choose_flow_split(text, &config, 3, 2, 2).unwrap();

    assert_eq!(decision.top_line_count, 2);
    assert_eq!(decision.top_text.trim_end(), "A forty-year old man named BEAMEN comes out of the seed store to greet Edward.");
    assert!(decision.bottom_text.trim_start().starts_with("Friendly but a little drunk"));
}
