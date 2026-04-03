#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HtmlRenderOptions {
    pub head: bool,
    pub exact_wraps: bool,
    pub paginated: bool,
}

impl Default for HtmlRenderOptions {
    fn default() -> Self {
        Self {
            head: true,
            exact_wraps: false,
            paginated: false,
        }
    }
}
