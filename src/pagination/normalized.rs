use crate::{render_attributes::RenderAttributes, styled_text::StyledText, Element, ElementText, Screenplay};

use super::fixtures::{NormalizedElement, NormalizedScreenplay};

pub fn normalize_screenplay(
    screenplay_id: impl Into<String>,
    screenplay: &Screenplay,
) -> NormalizedScreenplay {
    let mut collector = NormalizedCollector::default();
    for element in &screenplay.elements {
        collector.expand_element(element, None, None, None, None);
    }

    NormalizedScreenplay {
        screenplay: screenplay_id.into(),
        starting_page_number: None,
        elements: collector.elements,
    }
}

#[derive(Default)]
struct NormalizedCollector {
    next_element_id: usize,
    next_block_id: usize,
    next_dual_group_id: usize,
    elements: Vec<NormalizedElement>,
}

impl NormalizedCollector {
    fn expand_element(
        &mut self,
        element: &Element,
        block_kind: Option<&str>,
        block_id: Option<&str>,
        dual_dialogue_group: Option<&str>,
        dual_dialogue_side: Option<u8>,
    ) {
        match element {
            Element::DialogueBlock(children) => {
                self.next_block_id += 1;
                let block_id = format!("block-{:05}", self.next_block_id);
                for child in children {
                    self.expand_element(
                        child,
                        Some("DialogueBlock"),
                        Some(block_id.as_str()),
                        dual_dialogue_group,
                        dual_dialogue_side,
                    );
                }
            }
            Element::DualDialogueBlock(blocks) => {
                self.next_dual_group_id += 1;
                let group_id = format!("dual-{:05}", self.next_dual_group_id);
                for (idx, child) in blocks.iter().enumerate() {
                    self.expand_element(
                        child,
                        None,
                        None,
                        Some(group_id.as_str()),
                        Some((idx + 1) as u8),
                    );
                }
            }
            _ => self.push_leaf(
                element,
                block_kind,
                block_id,
                dual_dialogue_group,
                dual_dialogue_side,
            ),
        }
    }

    fn push_leaf(
        &mut self,
        element: &Element,
        block_kind: Option<&str>,
        block_id: Option<&str>,
        dual_dialogue_group: Option<&str>,
        dual_dialogue_side: Option<u8>,
    ) {
        let (kind, text, inline_text, centered, starts_new_page, scene_number) = match element {
            Element::Action(text, attributes)
            | Element::Character(text, attributes)
            | Element::SceneHeading(text, attributes)
            | Element::Lyric(text, attributes)
            | Element::Parenthetical(text, attributes)
            | Element::Dialogue(text, attributes)
            | Element::Transition(text, attributes)
            | Element::ColdOpening(text, attributes)
            | Element::NewAct(text, attributes)
            | Element::EndOfAct(text, attributes) => (
                element.name().to_string(),
                flatten_text(text),
                StyledText::from_element_text(text),
                attributes.centered,
                attributes.starts_new_page,
                attributes.scene_number.clone(),
            ),
            Element::Section(text, attributes, _) => (
                "Section".to_string(),
                flatten_text(text),
                StyledText::from_element_text(text),
                attributes.centered,
                attributes.starts_new_page,
                attributes.scene_number.clone(),
            ),
            Element::Synopsis(text) => (
                "Synopsis".to_string(),
                flatten_text(text),
                StyledText::from_element_text(text),
                false,
                false,
                None,
            ),
            Element::DialogueBlock(_) | Element::DualDialogueBlock(_) | Element::PageBreak => {
                return
            }
        };

        self.next_element_id += 1;
        self.elements.push(NormalizedElement {
            element_id: format!("el-{:05}", self.next_element_id),
            kind,
            text,
            inline_text,
            render_attributes: RenderAttributes {
                centered,
                starts_new_page,
                scene_number: scene_number.clone(),
            },
            fragment: None,
            block_kind: block_kind.map(str::to_string),
            block_id: block_id.map(str::to_string),
            dual_dialogue_group: dual_dialogue_group.map(str::to_string),
            dual_dialogue_side,
        });
    }
}

fn flatten_text(text: &ElementText) -> String {
    match text {
        ElementText::Plain(value) => value.clone(),
        ElementText::Styled(runs) => runs.iter().map(|run| run.content.as_str()).collect(),
    }
}
