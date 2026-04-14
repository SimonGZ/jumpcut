use crate::{ElementText, Metadata};

const TITLE_PAGE_METADATA_KEYS: [&str; 7] = [
    "title",
    "credit",
    "author",
    "authors",
    "source",
    "draft",
    "draft date",
];
const ALLOW_LOWERCASE_TITLE_FMT_OPTION: &str = "allow-lowercase-title";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitlePageRegion {
    CenterTitle,
    CenterMeta,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitlePageBlockKind {
    Title,
    Credit,
    Author,
    Source,
    Contact,
    Draft,
    DraftDate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TitlePageBlock {
    pub kind: TitlePageBlockKind,
    pub region: TitlePageRegion,
    pub lines: Vec<ElementText>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontmatterAlignment {
    Left,
    Center,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontmatterParagraph {
    pub text: ElementText,
    pub alignment: FrontmatterAlignment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontmatterPage {
    pub paragraphs: Vec<FrontmatterParagraph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TitlePage {
    pub blocks: Vec<TitlePageBlock>,
    pub frontmatter: Vec<FrontmatterPage>,
}

impl TitlePage {
    pub fn from_metadata(metadata: &Metadata) -> Option<Self> {
        let has_title_keys = TITLE_PAGE_METADATA_KEYS
            .iter()
            .chain(["contact"].iter())
            .any(|key| metadata.contains_key(*key));
        let has_frontmatter = metadata.contains_key("frontmatter");
        if !has_title_keys && !has_frontmatter {
            return None;
        }

        let mut blocks = Vec::new();
        push_block(
            &mut blocks,
            metadata,
            "title",
            TitlePageBlockKind::Title,
            TitlePageRegion::CenterTitle,
        );
        push_block(
            &mut blocks,
            metadata,
            "credit",
            TitlePageBlockKind::Credit,
            TitlePageRegion::CenterMeta,
        );
        let author_lines = metadata
            .get("author")
            .into_iter()
            .flatten()
            .chain(metadata.get("authors").into_iter().flatten())
            .cloned()
            .collect::<Vec<_>>();
        if !author_lines.is_empty() {
            blocks.push(TitlePageBlock {
                kind: TitlePageBlockKind::Author,
                region: TitlePageRegion::CenterMeta,
                lines: author_lines,
            });
        }
        push_block(
            &mut blocks,
            metadata,
            "source",
            TitlePageBlockKind::Source,
            TitlePageRegion::CenterMeta,
        );
        push_block(
            &mut blocks,
            metadata,
            "contact",
            TitlePageBlockKind::Contact,
            TitlePageRegion::BottomLeft,
        );
        push_block(
            &mut blocks,
            metadata,
            "draft",
            TitlePageBlockKind::Draft,
            TitlePageRegion::BottomRight,
        );
        push_block(
            &mut blocks,
            metadata,
            "draft date",
            TitlePageBlockKind::DraftDate,
            TitlePageRegion::BottomRight,
        );

        let frontmatter = metadata
            .get("frontmatter")
            .map(|lines| parse_frontmatter_pages(lines))
            .unwrap_or_default();

        Some(Self {
            blocks,
            frontmatter,
        })
    }

    pub fn block(&self, kind: TitlePageBlockKind) -> Option<&TitlePageBlock> {
        self.blocks.iter().find(|block| block.kind == kind)
    }

    /// Total number of title pages: always 1 for the main page, plus any frontmatter pages.
    pub fn total_page_count(&self) -> u32 {
        1 + self.frontmatter.len() as u32
    }
}

pub fn plain_title_uses_all_caps(metadata: &Metadata) -> bool {
    !metadata.get("fmt").into_iter().flatten().any(|value| {
        value
            .plain_text()
            .split_whitespace()
            .any(|option| option.eq_ignore_ascii_case(ALLOW_LOWERCASE_TITLE_FMT_OPTION))
    })
}

fn push_block(
    blocks: &mut Vec<TitlePageBlock>,
    metadata: &Metadata,
    key: &str,
    kind: TitlePageBlockKind,
    region: TitlePageRegion,
) {
    if let Some(lines) = metadata.get(key).filter(|lines| !lines.is_empty()) {
        blocks.push(TitlePageBlock {
            kind,
            region,
            lines: lines.clone(),
        });
    }
}

fn parse_frontmatter_pages(lines: &[ElementText]) -> Vec<FrontmatterPage> {
    let mut pages: Vec<FrontmatterPage> = Vec::new();
    let mut current_paragraphs: Vec<FrontmatterParagraph> = Vec::new();

    for line in lines {
        let plain = line.plain_text();
        let trimmed = plain.trim();

        // Page break separator
        if trimmed == "===" {
            if !current_paragraphs.is_empty() {
                pages.push(FrontmatterPage {
                    paragraphs: std::mem::take(&mut current_paragraphs),
                });
            }
            continue;
        }

        // Blank line = paragraph separator
        if trimmed.is_empty() {
            continue;
        }

        // Detect centered text: > text <
        if trimmed.starts_with('>') && trimmed.ends_with('<') {
            let centered_text = trimmed
                .trim_start_matches('>')
                .trim_end_matches('<')
                .trim();
            current_paragraphs.push(FrontmatterParagraph {
                text: ElementText::Plain(centered_text.to_string()),
                alignment: FrontmatterAlignment::Center,
            });
            continue;
        }

        // Normal paragraph line
        current_paragraphs.push(FrontmatterParagraph {
            text: line.clone(),
            alignment: FrontmatterAlignment::Left,
        });
    }

    if !current_paragraphs.is_empty() {
        pages.push(FrontmatterPage {
            paragraphs: current_paragraphs,
        });
    }

    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tr, ElementText::Styled};

    #[test]
    fn title_page_is_built_from_metadata_regions() {
        let mut metadata = Metadata::new();
        metadata.insert(
            "title".into(),
            vec![Styled(vec![tr("BIG FISH", vec!["Bold"])])],
        );
        metadata.insert("credit".into(), vec!["Written by".into()]);
        metadata.insert("author".into(), vec!["John August".into()]);
        metadata.insert("contact".into(), vec!["UTA".into()]);
        metadata.insert("draft".into(), vec!["Blue Draft".into()]);
        metadata.insert("draft date".into(), vec!["1/1/2000".into()]);

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert_eq!(
            title_page.block(TitlePageBlockKind::Title).unwrap().region,
            TitlePageRegion::CenterTitle
        );
        assert_eq!(
            title_page
                .block(TitlePageBlockKind::Contact)
                .unwrap()
                .region,
            TitlePageRegion::BottomLeft
        );
        assert_eq!(
            title_page.block(TitlePageBlockKind::Draft).unwrap().region,
            TitlePageRegion::BottomRight
        );
        assert_eq!(
            title_page
                .block(TitlePageBlockKind::DraftDate)
                .unwrap()
                .region,
            TitlePageRegion::BottomRight
        );
    }

    #[test]
    fn plain_title_uses_all_caps_by_default() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["Sample Script".into()]);

        assert!(plain_title_uses_all_caps(&metadata));
    }

    #[test]
    fn fmt_can_allow_lowercase_plain_titles() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["Sample Script".into()]);
        metadata.insert("fmt".into(), vec!["bsh allow-lowercase-title".into()]);

        assert!(!plain_title_uses_all_caps(&metadata));
    }

    #[test]
    fn title_page_without_frontmatter_has_empty_frontmatter() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert!(title_page.frontmatter.is_empty());
    }

    #[test]
    fn frontmatter_single_page_with_paragraphs() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);
        metadata.insert(
            "frontmatter".into(),
            vec![
                "WRITERS' NOTE".into(),
                "".into(),
                "First paragraph of the note.".into(),
                "".into(),
                "Second paragraph of the note.".into(),
            ],
        );

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert_eq!(title_page.frontmatter.len(), 1);
        let page = &title_page.frontmatter[0];
        assert_eq!(page.paragraphs.len(), 3);
        assert_eq!(page.paragraphs[0].text.plain_text(), "WRITERS' NOTE");
        assert_eq!(page.paragraphs[0].alignment, FrontmatterAlignment::Left);
        assert_eq!(
            page.paragraphs[1].text.plain_text(),
            "First paragraph of the note."
        );
        assert_eq!(
            page.paragraphs[2].text.plain_text(),
            "Second paragraph of the note."
        );
    }

    #[test]
    fn frontmatter_multi_page_split_on_page_break() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);
        metadata.insert(
            "frontmatter".into(),
            vec![
                "Page one content.".into(),
                "===".into(),
                "Page two content.".into(),
            ],
        );

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert_eq!(title_page.frontmatter.len(), 2);
        assert_eq!(title_page.frontmatter[0].paragraphs.len(), 1);
        assert_eq!(
            title_page.frontmatter[0].paragraphs[0].text.plain_text(),
            "Page one content."
        );
        assert_eq!(title_page.frontmatter[1].paragraphs.len(), 1);
        assert_eq!(
            title_page.frontmatter[1].paragraphs[0].text.plain_text(),
            "Page two content."
        );
    }

    #[test]
    fn frontmatter_detects_centered_text() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);
        metadata.insert(
            "frontmatter".into(),
            vec![
                "> A centered quote <".into(),
                "".into(),
                "A left-aligned paragraph.".into(),
            ],
        );

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert_eq!(title_page.frontmatter.len(), 1);
        let page = &title_page.frontmatter[0];
        assert_eq!(page.paragraphs.len(), 2);
        assert_eq!(
            page.paragraphs[0].text.plain_text(),
            "A centered quote"
        );
        assert_eq!(page.paragraphs[0].alignment, FrontmatterAlignment::Center);
        assert_eq!(page.paragraphs[1].alignment, FrontmatterAlignment::Left);
    }

    #[test]
    fn frontmatter_preserves_styled_text() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);
        metadata.insert(
            "frontmatter".into(),
            vec![Styled(vec![tr("WRITERS' NOTE", vec!["Underline"])])],
        );

        let title_page = TitlePage::from_metadata(&metadata).expect("expected title page");

        assert_eq!(title_page.frontmatter.len(), 1);
        assert_eq!(title_page.frontmatter[0].paragraphs.len(), 1);
        assert!(matches!(
            &title_page.frontmatter[0].paragraphs[0].text,
            Styled(_)
        ));
    }
}
