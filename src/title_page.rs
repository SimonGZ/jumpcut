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

#[derive(Clone, Debug, PartialEq)]
pub struct TitlePage {
    pub blocks: Vec<TitlePageBlock>,
}

impl TitlePage {
    pub fn from_metadata(metadata: &Metadata) -> Option<Self> {
        if !TITLE_PAGE_METADATA_KEYS
            .iter()
            .chain(["contact"].iter())
            .any(|key| metadata.contains_key(*key))
        {
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

        Some(Self { blocks })
    }

    pub fn block(&self, kind: TitlePageBlockKind) -> Option<&TitlePageBlock> {
        self.blocks.iter().find(|block| block.kind == kind)
    }
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
            title_page.block(TitlePageBlockKind::Contact).unwrap().region,
            TitlePageRegion::BottomLeft
        );
        assert_eq!(
            title_page.block(TitlePageBlockKind::Draft).unwrap().region,
            TitlePageRegion::BottomRight
        );
        assert_eq!(
            title_page.block(TitlePageBlockKind::DraftDate).unwrap().region,
            TitlePageRegion::BottomRight
        );
    }
}
