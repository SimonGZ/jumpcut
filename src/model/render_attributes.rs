use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderAttributes {
    #[serde(default)]
    pub centered: bool,
    #[serde(default)]
    pub starts_new_page: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_number: Option<String>,
}

impl RenderAttributes {
    pub fn is_default(attrs: &Self) -> bool {
        !attrs.centered && !attrs.starts_new_page && attrs.scene_number.is_none()
    }
}
