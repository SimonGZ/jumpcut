use std::collections::BTreeMap;

use super::fixtures::{NormalizedElement, NormalizedScreenplay};

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
    pub line_range: Option<(u32, u32)>,
    pub scene_number: Option<String>,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueUnit {
    pub block_id: String,
    pub parts: Vec<DialoguePart>,
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
    pub cohesion: Cohesion,
}

pub fn build_semantic_screenplay(normalized: NormalizedScreenplay) -> SemanticScreenplay {
    let mut units = Vec::new();
    let mut index = 0;

    while index < normalized.elements.len() {
        let element = &normalized.elements[index];
        if element.starts_new_page {
            units.push(SemanticUnit::PageStart(PageStartUnit {
                source_element_id: element.element_id.clone(),
            }));
        }

        if let Some(group_id) = &element.dual_dialogue_group {
            let start = index;
            let mut end = index + 1;
            while end < normalized.elements.len()
                && normalized.elements[end].dual_dialogue_group.as_deref() == Some(group_id.as_str())
            {
                end += 1;
            }
            units.push(SemanticUnit::DualDialogue(build_dual_dialogue_unit(
                group_id,
                &normalized.elements[start..end],
            )));
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
            units.push(SemanticUnit::Dialogue(build_dialogue_unit(
                block_id,
                &normalized.elements[start..end],
            )));
            index = end;
            continue;
        }

        if element.kind == "Lyric" {
            units.push(SemanticUnit::Lyric(build_lyric_unit(element)));
            index += 1;
            continue;
        }

        units.push(SemanticUnit::Flow(build_flow_unit(element)));
        index += 1;
    }

    SemanticScreenplay {
        screenplay: normalized.screenplay,
        starting_page_number: normalized.starting_page_number,
        units,
    }
}

fn build_dual_dialogue_unit(group_id: &str, elements: &[NormalizedElement]) -> DualDialogueUnit {
    let mut by_side_and_block: BTreeMap<(u8, String), Vec<NormalizedElement>> = BTreeMap::new();

    for element in elements {
        let side = element.dual_dialogue_side.unwrap_or(1);
        let block_id = element
            .block_id
            .clone()
            .unwrap_or_else(|| format!("dual-{}-side-{}", group_id, side));
        by_side_and_block
            .entry((side, block_id))
            .or_default()
            .push(element.clone());
    }

    let sides = by_side_and_block
        .into_iter()
        .map(|((side, block_id), elements)| DualDialogueSide {
            side,
            dialogue: build_dialogue_unit(&block_id, &elements),
        })
        .collect();

    DualDialogueUnit {
        group_id: group_id.to_string(),
        sides,
        cohesion: dialogue_like_cohesion(),
    }
}

fn build_dialogue_unit(block_id: &str, elements: &[NormalizedElement]) -> DialogueUnit {
    DialogueUnit {
        block_id: block_id.to_string(),
        parts: elements
            .iter()
            .map(|element| DialoguePart {
                element_id: element.element_id.clone(),
                kind: dialogue_part_kind(&element.kind),
                text: element.text.clone(),
            })
            .collect(),
        cohesion: dialogue_like_cohesion(),
    }
}

fn build_lyric_unit(element: &NormalizedElement) -> LyricUnit {
    LyricUnit {
        element_id: element.element_id.clone(),
        text: element.text.clone(),
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
        line_range: None,
        scene_number: element.scene_number.clone(),
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
