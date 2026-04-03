use jumpcut::pagination::flow_split::{
    choose_flow_split, choose_flow_split_allow_exact_fit_sentence_runt,
};
use jumpcut::pagination::wrapping::WrapConfig;

#[test]
fn flow_split_prefers_a_sentence_boundary_when_a_legal_split_exists() {
    let text = "A forty-year old man named BEAMEN comes out of the seed store to greet Edward.  Friendly but a little drunk, he's the closest thing the town has to a mayor. He's carrying a clipboard.";
    let config = WrapConfig::with_exact_width_chars(61);

    let decision = choose_flow_split(text, &config, 3, 2, 2).unwrap();

    assert_eq!(decision.top_line_count, 2);
    assert_eq!(decision.top_text.trim_end(), "A forty-year old man named BEAMEN comes out of the seed store to greet Edward.");
    assert_eq!(decision.top_end_offset, decision.top_text.len());
    assert_eq!(decision.bottom_start_offset, decision.top_end_offset);
    assert!(decision.bottom_text.trim_start().starts_with("Friendly but a little drunk"));
}

#[test]
fn flow_split_rejects_non_sentence_split_when_no_sentence_boundary_fits() {
    let text = "Rus okuqagozu arev hurur abewyge gewuzad ys Udaxek, oge’u qajoqas eb rus howo raxa uk ok kuqysogoq godaqug qe YZEPYPAX YWES EWODEJO (wygaba-waqu, ohaxu, usuwobozyke). Udaxek’u erodevu ok yxobuhu qe eky qega, kud ok ZAQ ojo z ogusek eb eky ryb, gyzaqav eky asaxy. Udaxek zypuj zagazak.";
    let config = WrapConfig::with_exact_width_chars(61);

    let decision = choose_flow_split(text, &config, 2, 2, 2);

    assert!(
        decision.is_none(),
        "expected no flow split when the first legal sentence boundary needs more top lines than the page can fit"
    );
}

#[test]
fn flow_split_allows_exact_fit_sentence_fragment_even_with_a_short_last_line() {
    let text = "Z ygeb ujazopoda ovepaj kequgarajar yvy uk ok uje Eroryheg ozejy. Udaxek, eb eky raxusore asazypo, useb ys z habu udybyzu rezas yvy uk OVEHO'U UJUQY RYGYVAKY. Hoz ryxu wyde qesy yvy zo rus ebeba qe qaryhyj: Ok uwokeby, qaba gywusyw, yzyv qoso, kud z kyvup qabex apokakeh.";
    let config = WrapConfig::with_exact_width_chars(61);

    let decision =
        choose_flow_split_allow_exact_fit_sentence_runt(text, &config, 2, 2, 2).unwrap();

    assert_eq!(decision.top_line_count, 2);
    assert_eq!(
        decision.top_text.trim_end(),
        "Z ygeb ujazopoda ovepaj kequgarajar yvy uk ok uje Eroryheg ozejy."
    );
    assert!(decision.bottom_text.trim_start().starts_with("Udaxek, eb eky"));
}

#[test]
fn flow_split_keeps_big_fish_runt_sentence_split_illegal_for_ordinary_action() {
    let text = "Looking up, she sees not a shadow but Edward himself standing before her. She GASPS, disbelieving, but his hand is real. It is destiny.";
    let config = WrapConfig::with_exact_width_chars(61);

    let decision = choose_flow_split(text, &config, 2, 2, 2);

    assert!(
        decision.is_none(),
        "ordinary action flow splits should still reject the Big Fish runt-top-line candidate"
    );
}
