use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::default::Default;

pub type Metadata = HashMap<String, Vec<ElementText>>;

#[derive(Debug, PartialEq, Serialize)]
pub struct Screenplay {
    pub metadata: Metadata,
    pub imported_layout: Option<ImportedLayoutOverrides>,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedLayoutOverrides {
    pub page: ImportedPageLayoutOverrides,
    pub element_styles: BTreeMap<ImportedElementKind, ImportedElementStyle>,
    pub mores_and_continueds: ImportedMoresAndContinueds,
}

impl ImportedLayoutOverrides {
    pub fn is_empty(&self) -> bool {
        self.page == ImportedPageLayoutOverrides::default()
            && self.element_styles.is_empty()
            && self.mores_and_continueds == ImportedMoresAndContinueds::default()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedPageLayoutOverrides {
    pub page_width: Option<f32>,
    pub page_height: Option<f32>,
    pub top_margin: Option<f32>,
    pub bottom_margin: Option<f32>,
    pub header_margin: Option<f32>,
    pub footer_margin: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum ImportedElementKind {
    Action,
    SceneHeading,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Lyric,
    ColdOpening,
    NewAct,
    EndOfAct,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedElementStyle {
    pub first_indent: Option<f32>,
    pub left_indent: Option<f32>,
    pub right_indent: Option<f32>,
    pub spacing_before: Option<f32>,
    pub line_spacing: Option<f32>,
    pub alignment: Option<ImportedAlignment>,
    pub starts_new_page: Option<bool>,
    pub underline: Option<bool>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ImportedAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedMoresAndContinueds {
    pub dialogue: ImportedDialogueContinueds,
    pub scene: ImportedSceneContinueds,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedDialogueContinueds {
    pub automatic_character_continueds: Option<bool>,
    pub top_of_next: Option<bool>,
    pub bottom_of_page: Option<bool>,
    pub dialogue_top: Option<String>,
    pub dialogue_bottom: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImportedSceneContinueds {
    pub top_of_next: Option<bool>,
    pub bottom_of_page: Option<bool>,
    pub continued_number: Option<bool>,
    pub scene_top: Option<String>,
    pub scene_bottom: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Action(ElementText, Attributes),
    Character(ElementText, Attributes),
    SceneHeading(ElementText, Attributes),
    Lyric(ElementText, Attributes),
    Parenthetical(ElementText, Attributes),
    Dialogue(ElementText, Attributes),
    DialogueBlock(Vec<Element>),
    DualDialogueBlock(Vec<Element>),
    Transition(ElementText, Attributes),
    Section(ElementText, Attributes, u8),
    Synopsis(ElementText),
    ColdOpening(ElementText, Attributes),
    NewAct(ElementText, Attributes),
    EndOfAct(ElementText, Attributes),
    PageBreak,
}

impl Element {
    pub(crate) fn name(&self) -> &str {
        use Element::*;
        match *self {
            Action(_, _) => "Action",
            Character(_, _) => "Character",
            SceneHeading(_, _) => "Scene Heading",
            Lyric(_, _) => "Lyric",
            Parenthetical(_, _) => "Parenthetical",
            Dialogue(_, _) => "Dialogue",
            DialogueBlock(_) => "Dialogue Block",
            DualDialogueBlock(_) => "Dual Dialogue Block",
            Transition(_, _) => "Transition",
            Section(_, _, _) => "Section",
            Synopsis(_) => "Synopsis",
            ColdOpening(_, _) => "Cold Opening",
            NewAct(_, _) => "New Act",
            EndOfAct(_, _) => "End of Act",
            PageBreak => "Page Break",
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct SerializeElementHelper<'a> {
    #[serde(rename = "type")]
    element_type: &'a str,
    text: &'a ElementText,
    attributes: &'a Attributes,
}

impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Element::Action(ref text, ref attributes)
            | Element::Character(ref text, ref attributes)
            | Element::SceneHeading(ref text, ref attributes)
            | Element::Lyric(ref text, ref attributes)
            | Element::Parenthetical(ref text, ref attributes)
            | Element::Dialogue(ref text, ref attributes)
            | Element::Transition(ref text, ref attributes)
            | Element::ColdOpening(ref text, ref attributes)
            | Element::NewAct(ref text, ref attributes)
            | Element::EndOfAct(ref text, ref attributes) => {
                let el = SerializeElementHelper {
                    element_type: self.name(),
                    text,
                    attributes,
                };
                el.serialize(serializer)
            }
            Element::DialogueBlock(ref block) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "DialogueBlock")?;
                map.serialize_entry("block", block)?;
                map.end()
            }
            Element::DualDialogueBlock(ref blocks) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "DualDialogueBlock")?;
                map.serialize_entry("blocks", blocks)?;
                map.end()
            }
            Element::Section(ref text, ref attributes, ref level) => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("type", "Section")?;
                map.serialize_entry("text", text)?;
                map.serialize_entry("attributes", attributes)?;
                map.serialize_entry("level", level)?;
                map.end()
            }
            Element::Synopsis(ref text) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "Synopsis")?;
                map.serialize_entry("text", text)?;
                map.end()
            }
            Element::PageBreak => serializer.serialize_none(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Attributes {
    pub centered: bool,
    pub starts_new_page: bool,
    pub scene_number: Option<String>,
    pub notes: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ElementText {
    Plain(String),
    Styled(Vec<TextRun>),
}

impl ElementText {
    pub fn plain_text(&self) -> String {
        match self {
            ElementText::Plain(text) => text.clone(),
            ElementText::Styled(runs) => runs.iter().map(|run| run.content.as_str()).collect(),
        }
    }
}

impl Default for ElementText {
    fn default() -> Self {
        ElementText::Plain(String::new())
    }
}

impl From<&str> for ElementText {
    fn from(value: &str) -> Self {
        ElementText::Plain(value.to_string())
    }
}

impl From<String> for ElementText {
    fn from(value: String) -> Self {
        ElementText::Plain(value)
    }
}

// Convenience function
pub fn p(p: &str) -> ElementText {
    ElementText::Plain(p.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TextRun {
    pub content: String,
    #[serde(serialize_with = "text_style_serialize")]
    pub text_style: HashSet<String>,
}

// Convenience function
pub fn tr(t: &str, s: Vec<&str>) -> TextRun {
    let mut styles: HashSet<String> = HashSet::new();
    for str in s {
        styles.insert(str.to_string());
    }
    TextRun {
        content: t.to_string(),
        text_style: styles,
    }
}

fn text_style_serialize<S>(x: &HashSet<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut styles: Vec<String> = x.clone().into_iter().collect();
    styles.sort();
    styles.serialize(s)
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            centered: false,
            starts_new_page: false,
            scene_number: None,
            notes: None,
        }
    }
}

pub fn blank_attributes() -> Attributes {
    Attributes {
        ..Attributes::default()
    }
}
