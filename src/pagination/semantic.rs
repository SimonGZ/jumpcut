use std::collections::BTreeSet;

use super::fixtures::{NormalizedElement, NormalizedScreenplay};
use crate::render_attributes::RenderAttributes;
use crate::styled_text::StyledText;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cohesion {
    pub keep_together: bool,
    pub keep_with_next: bool,
    pub can_split: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticScreenplay {
    pub screenplay: String,
    pub starting_page_number: Option<u32>,
    pub units: Vec<SemanticUnit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticUnit {
    PageStart(PageStartUnit),
    Flow(FlowUnit),
    Dialogue(DialogueUnit),
    Lyric(LyricUnit),
    DualDialogue(DualDialogueUnit),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageStartUnit {
    pub source_element_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlowKind {
    Action,
    SceneHeading,
    Transition,
    Section,
    Synopsis,
    ColdOpening,
    NewAct,
    EndOfAct,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowUnit {
    pub element_id: String,
    pub kind: FlowKind,
    pub text: String,
    pub inline_text: Option<StyledText>,
    pub render_attributes: RenderAttributes,
    pub line_range: Option<(u32, u32)>,
    pub cohesion: Cohesion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialoguePartKind {
    Character,
    Parenthetical,
    Dialogue,
    Lyric,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialoguePart {
    pub element_id: String,
    pub kind: DialoguePartKind,
    pub text: String,
    pub inline_text: Option<StyledText>,
    pub render_attributes: RenderAttributes,
    pub should_append_contd: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueUnit {
    pub block_id: String,
    pub parts: Vec<DialoguePart>,
    pub should_append_contd: bool,
    pub cohesion: Cohesion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DualDialogueSide {
    pub side: u8,
    pub dialogue: DialogueUnit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DualDialogueUnit {
    pub group_id: String,
    pub sides: Vec<DualDialogueSide>,
    pub cohesion: Cohesion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LyricUnit {
    pub element_id: String,
    pub text: String,
    pub inline_text: Option<StyledText>,
    pub render_attributes: RenderAttributes,
    pub cohesion: Cohesion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticOptions {
    pub dual_dialogue_counts_for_contd: bool,
}

impl Default for SemanticOptions {
    fn default() -> Self {
        Self {
            dual_dialogue_counts_for_contd: true,
        }
    }
}

pub fn build_semantic_screenplay(normalized: NormalizedScreenplay) -> SemanticScreenplay {
    build_semantic_screenplay_with_options(normalized, SemanticOptions::default())
}

pub fn build_semantic_screenplay_with_options(
    normalized: NormalizedScreenplay,
    options: SemanticOptions,
) -> SemanticScreenplay {
    let mut units = Vec::new();
    let mut index = 0;
    let mut last_scene_dialogue_speakers = BTreeSet::new();

    while index < normalized.elements.len() {
        let element = &normalized.elements[index];
        if is_non_visual_element_kind(&element.kind) {
            index += 1;
            continue;
        }
        if element.render_attributes.starts_new_page {
            units.push(SemanticUnit::PageStart(PageStartUnit {
                source_element_id: element.element_id.clone(),
            }));
        }

        if let Some(group_id) = &element.dual_dialogue_group {
            let start = index;
            let mut end = index + 1;
            while end < normalized.elements.len()
                && normalized.elements[end].dual_dialogue_group.as_deref()
                    == Some(group_id.as_str())
            {
                end += 1;
            }
            units.push(SemanticUnit::DualDialogue(build_dual_dialogue_unit(
                group_id,
                &normalized.elements[start..end],
                &last_scene_dialogue_speakers,
            )));
            last_scene_dialogue_speakers = if options.dual_dialogue_counts_for_contd {
                last_dual_dialogue_round_speakers(&normalized.elements[start..end])
            } else {
                BTreeSet::new()
            };
            index = end;
            continue;
        }

        if element.block_id.is_some() {
            let block_id = element.block_id.as_deref().unwrap();
            let start = index;
            let mut end = index + 1;
            while end < normalized.elements.len()
                && normalized.elements[end].block_id.as_deref() == Some(block_id)
            {
                end += 1;
            }
            let dialogue = build_dialogue_unit(
                block_id,
                &normalized.elements[start..end],
                &last_scene_dialogue_speakers,
            );
            last_scene_dialogue_speakers = dialogue_contd_speaker(&dialogue.parts)
                .into_iter()
                .collect();
            units.push(SemanticUnit::Dialogue(dialogue));
            index = end;
            continue;
        }

        if element.kind == "Lyric" {
            units.push(SemanticUnit::Lyric(build_lyric_unit(element)));
            index += 1;
            continue;
        }

        units.push(SemanticUnit::Flow(build_flow_unit(element)));
        if element.kind == "Scene Heading" {
            last_scene_dialogue_speakers.clear();
        }
        index += 1;
    }

    SemanticScreenplay {
        screenplay: normalized.screenplay,
        starting_page_number: normalized.starting_page_number,
        units,
    }
}

fn is_non_visual_element_kind(kind: &str) -> bool {
    matches!(kind, "Section" | "Synopsis")
}

fn build_dual_dialogue_unit(
    group_id: &str,
    elements: &[NormalizedElement],
    previous_block_speakers: &BTreeSet<String>,
) -> DualDialogueUnit {
    let left_blocks = dual_dialogue_blocks_for_side(group_id, elements, 1);
    let right_blocks = dual_dialogue_blocks_for_side(group_id, elements, 2);
    let mut previous_round_speakers = previous_block_speakers.clone();
    let mut left_parts = Vec::new();
    let mut right_parts = Vec::new();
    let mut left_should_append = false;
    let mut right_should_append = false;

    for round in 0..left_blocks.len().max(right_blocks.len()) {
        let mut current_round_speakers = BTreeSet::new();

        if let Some((block_id, block_elements)) = left_blocks.get(round) {
            let dialogue = build_dialogue_unit(block_id, block_elements, &previous_round_speakers);
            left_should_append |= dialogue.should_append_contd;
            current_round_speakers.extend(dialogue_contd_speaker(&dialogue.parts));
            left_parts.extend(dialogue.parts);
        }

        if let Some((block_id, block_elements)) = right_blocks.get(round) {
            let dialogue = build_dialogue_unit(block_id, block_elements, &previous_round_speakers);
            right_should_append |= dialogue.should_append_contd;
            current_round_speakers.extend(dialogue_contd_speaker(&dialogue.parts));
            right_parts.extend(dialogue.parts);
        }

        if !current_round_speakers.is_empty() {
            previous_round_speakers = current_round_speakers;
        }
    }

    let mut sides = Vec::new();
    if !left_parts.is_empty() {
        sides.push(DualDialogueSide {
            side: 1,
            dialogue: DialogueUnit {
                block_id: format!("{group_id}-left"),
                should_append_contd: left_should_append,
                parts: left_parts,
                cohesion: dialogue_like_cohesion(),
            },
        });
    }
    if !right_parts.is_empty() {
        sides.push(DualDialogueSide {
            side: 2,
            dialogue: DialogueUnit {
                block_id: format!("{group_id}-right"),
                should_append_contd: right_should_append,
                parts: right_parts,
                cohesion: dialogue_like_cohesion(),
            },
        });
    }

    DualDialogueUnit {
        group_id: group_id.to_string(),
        sides,
        cohesion: dialogue_like_cohesion(),
    }
}

fn build_dialogue_unit(
    block_id: &str,
    elements: &[NormalizedElement],
    previous_scene_speakers: &BTreeSet<String>,
) -> DialogueUnit {
    let current_speaker = dialogue_contd_speaker_for_elements(elements);
    let should_append_contd = current_speaker
        .as_deref()
        .is_some_and(|current| previous_scene_speakers.contains(current));
    let parts = elements
        .iter()
        .map(|element| DialoguePart {
            element_id: element.element_id.clone(),
            kind: dialogue_part_kind(&element.kind),
            text: element.text.clone(),
            inline_text: element.inline_text.clone(),
            render_attributes: element.render_attributes.clone(),
            should_append_contd: should_append_contd && element.kind == "Character",
        })
        .collect::<Vec<_>>();

    DialogueUnit {
        block_id: block_id.to_string(),
        should_append_contd,
        parts,
        cohesion: dialogue_like_cohesion(),
    }
}

fn dialogue_contd_speaker_for_elements(elements: &[NormalizedElement]) -> Option<String> {
    let parts = elements
        .iter()
        .map(|element| DialoguePart {
            element_id: element.element_id.clone(),
            kind: dialogue_part_kind(&element.kind),
            text: element.text.clone(),
            inline_text: element.inline_text.clone(),
            render_attributes: element.render_attributes.clone(),
            should_append_contd: false,
        })
        .collect::<Vec<_>>();

    dialogue_contd_speaker(&parts)
}

fn dual_dialogue_blocks_for_side(
    group_id: &str,
    elements: &[NormalizedElement],
    side: u8,
) -> Vec<(String, Vec<NormalizedElement>)> {
    let mut blocks: Vec<(String, Vec<NormalizedElement>)> = Vec::new();

    for element in elements {
        if element.dual_dialogue_side.unwrap_or(1) != side {
            continue;
        }
        let block_id = element
            .block_id
            .clone()
            .unwrap_or_else(|| format!("dual-{group_id}-side-{side}"));
        match blocks.last_mut() {
            Some((existing_block_id, block_elements)) if *existing_block_id == block_id => {
                block_elements.push(element.clone());
            }
            _ => blocks.push((block_id, vec![element.clone()])),
        }
    }

    blocks
}

fn last_dual_dialogue_round_speakers(elements: &[NormalizedElement]) -> BTreeSet<String> {
    let left_blocks = dual_dialogue_blocks_for_side("round", elements, 1);
    let right_blocks = dual_dialogue_blocks_for_side("round", elements, 2);
    let last_round = left_blocks.len().max(right_blocks.len()).saturating_sub(1);
    let mut speakers = BTreeSet::new();

    if let Some((_, block_elements)) = left_blocks.get(last_round) {
        speakers.extend(dialogue_contd_speaker_for_elements(block_elements));
    }
    if let Some((_, block_elements)) = right_blocks.get(last_round) {
        speakers.extend(dialogue_contd_speaker_for_elements(block_elements));
    }

    speakers
}

fn build_lyric_unit(element: &NormalizedElement) -> LyricUnit {
    LyricUnit {
        element_id: element.element_id.clone(),
        text: element.text.clone(),
        inline_text: element.inline_text.clone(),
        render_attributes: element.render_attributes.clone(),
        cohesion: Cohesion {
            keep_together: false,
            keep_with_next: false,
            can_split: true,
        },
    }
}

fn build_flow_unit(element: &NormalizedElement) -> FlowUnit {
    FlowUnit {
        element_id: element.element_id.clone(),
        kind: flow_kind(&element.kind),
        text: element.text.clone(),
        inline_text: element.inline_text.clone(),
        render_attributes: element.render_attributes.clone(),
        line_range: None,
        cohesion: match element.kind.as_str() {
            "Scene Heading" => Cohesion {
                keep_together: true,
                keep_with_next: true,
                can_split: false,
            },
            "Transition" | "Cold Opening" | "New Act" | "End of Act" => Cohesion {
                keep_together: true,
                keep_with_next: false,
                can_split: false,
            },
            _ => Cohesion {
                keep_together: false,
                keep_with_next: false,
                can_split: true,
            },
        },
    }
}

fn dialogue_like_cohesion() -> Cohesion {
    Cohesion {
        keep_together: false,
        keep_with_next: false,
        can_split: true,
    }
}

fn flow_kind(kind: &str) -> FlowKind {
    match kind {
        "Scene Heading" => FlowKind::SceneHeading,
        "Transition" => FlowKind::Transition,
        "Section" => FlowKind::Section,
        "Synopsis" => FlowKind::Synopsis,
        "Cold Opening" => FlowKind::ColdOpening,
        "New Act" => FlowKind::NewAct,
        "End of Act" => FlowKind::EndOfAct,
        _ => FlowKind::Action,
    }
}

fn dialogue_part_kind(kind: &str) -> DialoguePartKind {
    match kind {
        "Character" => DialoguePartKind::Character,
        "Parenthetical" => DialoguePartKind::Parenthetical,
        "Lyric" => DialoguePartKind::Lyric,
        _ => DialoguePartKind::Dialogue,
    }
}

fn dialogue_contd_speaker(parts: &[DialoguePart]) -> Option<String> {
    let text = parts
        .iter()
        .find(|part| part.kind == DialoguePartKind::Character)
        .map(|part| part.text.trim())?;
    let stripped = strip_trailing_contd(text);

    if speaker_excludes_contd(stripped) {
        return None;
    }

    Some(strip_trailing_speaker_extensions(stripped).to_string())
}

fn strip_trailing_contd(text: &str) -> &str {
    let trimmed = text.trim_end();
    let upper = trimmed.to_ascii_uppercase();

    if upper.ends_with("(CONT'D)") || upper.ends_with("(CONT’D)") {
        let suffix_start = trimmed.rfind('(').unwrap_or(trimmed.len());
        return trimmed[..suffix_start].trim_end();
    }

    trimmed
}

fn speaker_excludes_contd(text: &str) -> bool {
    let upper = text.trim_end().to_ascii_uppercase();
    upper.contains("(V.O.)") || upper.contains("(V.O)")
}

fn strip_trailing_speaker_extensions(text: &str) -> &str {
    let mut current = text.trim_end();

    while current.ends_with(')') {
        let Some(open_paren) = current.rfind('(') else {
            break;
        };
        current = current[..open_paren].trim_end();
    }

    current
}
