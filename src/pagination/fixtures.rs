use serde::{Deserialize, Serialize};
use crate::styled_text::StyledText;

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedElement {
    pub element_id: String,
    pub kind: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_text: Option<StyledText>,
    pub fragment: Option<Fragment>,
    #[serde(default)]
    pub centered: bool,
    pub starts_new_page: bool,
    pub scene_number: Option<String>,
    pub block_kind: Option<String>,
    pub block_id: Option<String>,
    pub dual_dialogue_group: Option<String>,
    pub dual_dialogue_side: Option<u8>,
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
