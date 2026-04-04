#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlRenderOptions {
    pub head: bool,
    pub exact_wraps: bool,
    pub paginated: bool,
    pub render_continueds: bool,
    pub embed_courier_prime: bool,
    pub embedded_courier_prime_css: Option<String>,
}

impl Default for HtmlRenderOptions {
    fn default() -> Self {
        Self {
            head: true,
            exact_wraps: false,
            paginated: false,
            render_continueds: true,
            embed_courier_prime: false,
            embedded_courier_prime_css: None,
        }
    }
}

pub fn embedded_courier_prime_css_from_base64(
    regular_ttf_base64: &str,
    italic_ttf_base64: &str,
    bold_ttf_base64: &str,
    bold_italic_ttf_base64: &str,
) -> String {
    [
        embedded_font_face_from_base64("Courier Prime", 400, "normal", regular_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 400, "italic", italic_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 700, "normal", bold_ttf_base64),
        embedded_font_face_from_base64("Courier Prime", 700, "italic", bold_italic_ttf_base64),
    ]
    .join("\n")
}

fn embedded_font_face_from_base64(
    font_family: &str,
    font_weight: u16,
    font_style: &str,
    encoded: &str,
) -> String {
    format!(
        "@font-face {{\n  font-family: \"{font_family}\";\n  src: url(data:font/ttf;base64,{encoded}) format(\"truetype\");\n  font-weight: {font_weight};\n  font-style: {font_style};\n}}\n"
    )
}
