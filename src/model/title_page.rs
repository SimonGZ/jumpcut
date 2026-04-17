use crate::{
    ElementText, ImportedTitlePageAlignment, Metadata, Screenplay,
};

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
        build_title_page(metadata, Vec::new())
    }

    pub fn from_screenplay(screenplay: &Screenplay) -> Option<Self> {
        let imported_frontmatter = screenplay
            .imported_title_page
            .as_ref()
            .filter(|title_page| title_page.pages.len() > 1)
            .map(|title_page| {
                title_page.pages[1..]
                    .iter()
                    .map(|page| FrontmatterPage {
                        paragraphs: page
                            .paragraphs
                            .iter()
                            .filter(|paragraph| !paragraph.text.plain_text().trim().is_empty())
                            .map(|paragraph| FrontmatterParagraph {
                                text: paragraph.text.clone(),
                                alignment: match paragraph.alignment {
                                    ImportedTitlePageAlignment::Center => {
                                        FrontmatterAlignment::Center
                                    }
                                    _ => FrontmatterAlignment::Left,
                                },
                            })
                            .collect(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        build_title_page(&screenplay.metadata, imported_frontmatter)
    }
}

fn build_title_page(metadata: &Metadata, imported_frontmatter: Vec<FrontmatterPage>) -> Option<TitlePage> {
    let has_title_keys = TITLE_PAGE_METADATA_KEYS
        .iter()
        .chain(["contact"].iter())
        .any(|key| metadata.contains_key(*key));
    let has_frontmatter = !imported_frontmatter.is_empty();
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

    Some(TitlePage {
        blocks,
        frontmatter: imported_frontmatter,
    })
}

impl TitlePage {
    pub fn block(&self, kind: TitlePageBlockKind) -> Option<&TitlePageBlock> {
        self.blocks.iter().find(|block| block.kind == kind)
    }

    /// Total number of title pages: always 1 for the main page, plus any frontmatter pages.
    pub fn total_page_count(&self) -> u32 {
        1 + self.frontmatter.len() as u32
    }
}

pub fn frontmatter_count(screenplay: &Screenplay) -> Option<u32> {
    screenplay
        .metadata
        .get("frontmatter-page-count")
        .and_then(|values| values.first())
        .and_then(|value| value.plain_text().trim().parse::<u32>().ok())
        .filter(|count| *count > 0)
        .map(|count| count + 1)
        .or_else(|| TitlePage::from_screenplay(screenplay).map(|title_page| title_page.total_page_count()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tr, ElementText::Styled, ImportedTitlePage, ImportedTitlePagePage,
        ImportedTitlePageParagraph,
    };

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
    fn title_page_from_screenplay_prefers_imported_overflow_pages() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: Some(ImportedTitlePage {
                header_footer: Default::default(),
                pages: vec![
                    ImportedTitlePagePage { paragraphs: vec![] },
                    ImportedTitlePagePage {
                        paragraphs: vec![ImportedTitlePageParagraph {
                            text: "THE GUYS".into(),
                            alignment: ImportedTitlePageAlignment::Center,
                            left_indent: Some(0.94),
                            space_before: None,
                            tab_stops: Vec::new(),
                        }],
                    },
                ],
            }),
            elements: vec![],
        };

        let title_page = TitlePage::from_screenplay(&screenplay).expect("expected title page");

        assert_eq!(title_page.frontmatter.len(), 1);
        assert_eq!(
            title_page.frontmatter[0].paragraphs[0].text.plain_text(),
            "THE GUYS"
        );
        assert_eq!(
            title_page.frontmatter[0].paragraphs[0].alignment,
            FrontmatterAlignment::Center
        );
    }

    #[test]
    fn screenplay_frontmatter_count_metadata_overrides_derived_title_page_count() {
        let mut metadata = Metadata::new();
        metadata.insert("title".into(), vec!["MY SCRIPT".into()]);
        metadata.insert("frontmatter-page-count".into(), vec!["2".into()]);

        let screenplay = Screenplay {
            metadata,
            imported_layout: None,
            imported_title_page: None,
            elements: vec![],
        };

        assert_eq!(frontmatter_count(&screenplay), Some(3));
    }
}
