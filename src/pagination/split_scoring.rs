pub(crate) fn choose_best_scored_split<Candidate, Score, I, F>(
    candidates: I,
    mut score_fn: F,
) -> Option<Candidate>
where
    Candidate: Copy,
    Score: Ord,
    I: IntoIterator<Item = Candidate>,
    F: FnMut(Candidate) -> Option<Score>,
{
    let mut best: Option<(Score, Candidate)> = None;

    for candidate in candidates {
        let Some(score) = score_fn(candidate) else {
            continue;
        };

        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, candidate));
        }
    }

    best.map(|(_, candidate)| candidate)
}
