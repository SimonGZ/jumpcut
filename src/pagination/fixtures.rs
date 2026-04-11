use crate::render_attributes::RenderAttributes;
use crate::styled_text::StyledText;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fragment {
    Whole,
    ContinuedFromPrev,
    ContinuedToNext,
    ContinuedFromPrevAndToNext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange(pub u32, pub u32);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginationScope {
    pub title_page_count: Option<u32>,
    pub body_start_page: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageBreakFixtureSourceRefs {
    pub fountain: Option<String>,
    pub fdx: Option<String>,
    pub pdf: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NormalizedElement {
    pub element_id: String,
    pub kind: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_text: Option<StyledText>,
    #[serde(default, skip_serializing_if = "RenderAttributes::is_default")]
    pub render_attributes: RenderAttributes,
    pub fragment: Option<Fragment>,
    pub block_kind: Option<String>,
    pub block_id: Option<String>,
    pub dual_dialogue_group: Option<String>,
    pub dual_dialogue_side: Option<u8>,
}

impl<'de> Deserialize<'de> for NormalizedElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NormalizedElementRepr {
            element_id: String,
            kind: String,
            text: String,
            #[serde(default)]
            inline_text: Option<StyledText>,
            #[serde(default)]
            render_attributes: Option<RenderAttributes>,
            fragment: Option<Fragment>,
            #[serde(default)]
            centered: bool,
            #[serde(default)]
            starts_new_page: bool,
            #[serde(default)]
            scene_number: Option<String>,
            block_kind: Option<String>,
            block_id: Option<String>,
            dual_dialogue_group: Option<String>,
            dual_dialogue_side: Option<u8>,
        }

        let repr = NormalizedElementRepr::deserialize(deserializer)?;
        let render_attributes = repr.render_attributes.unwrap_or(RenderAttributes {
            centered: repr.centered,
            starts_new_page: repr.starts_new_page,
            scene_number: repr.scene_number,
        });

        Ok(Self {
            element_id: repr.element_id,
            kind: repr.kind,
            text: repr.text,
            inline_text: repr.inline_text,
            render_attributes,
            fragment: repr.fragment,
            block_kind: repr.block_kind,
            block_id: repr.block_id,
            dual_dialogue_group: repr.dual_dialogue_group,
            dual_dialogue_side: repr.dual_dialogue_side,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedScreenplay {
    pub screenplay: String,
    pub starting_page_number: Option<u32>,
    pub elements: Vec<NormalizedElement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageItem {
    pub element_id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_preview: Option<String>,
    pub fragment: Fragment,
    pub line_range: Option<LineRange>,
    pub block_id: Option<String>,
    pub dual_dialogue_group: Option<String>,
    pub dual_dialogue_side: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageBreakFixturePage {
    pub number: u32,
    pub items: Vec<PageItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageBreakFixture {
    pub screenplay: String,
    pub style_profile: String,
    pub source: PageBreakFixtureSourceRefs,
    pub scope: PaginationScope,
    pub pages: Vec<PageBreakFixturePage>,
}
