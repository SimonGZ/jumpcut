use std::collections::HashMap;

use serde::Serialize;

use super::{Fragment, PageBreakFixture, PaginatedScreenplay};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonIssueKind {
    MissingOccurrence,
    UnexpectedOccurrence,
    WrongPage,
    WrongFragment,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComparisonIssue {
    pub kind: ComparisonIssueKind,
    pub element_id: String,
    pub occurrence: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_preview: Option<String>,
    pub expected_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_page_label: Option<String>,
    pub actual_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_page_label: Option<String>,
    pub expected_fragment: Option<Fragment>,
    pub actual_fragment: Option<Fragment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComparisonReport {
    pub expected_page_count: usize,
    pub actual_page_count: usize,
    pub issues: Vec<ComparisonIssue>,
}

impl ComparisonReport {
    pub fn issue_count(&self, kind: ComparisonIssueKind) -> usize {
        self.issues.iter().filter(|issue| issue.kind == kind).count()
    }

    pub fn total_issues(&self) -> usize {
        self.issues.len()
    }
}

pub fn compare_paginated_to_fixture(
    actual: &PaginatedScreenplay,
    expected: &PageBreakFixture,
) -> ComparisonReport {
    let actual_occurrences = collect_actual_occurrences(actual);
    let expected_occurrences = collect_expected_occurrences(expected);
    let expected_previews = collect_expected_previews(&expected_occurrences);
    let mut issues = Vec::new();
    let max_len = actual_occurrences.len().max(expected_occurrences.len());

    for index in 0..max_len {
        match (expected_occurrences.get(index), actual_occurrences.get(index)) {
            (Some(expected_item), Some(actual_item))
                if expected_item.element_id == actual_item.element_id
                    && expected_item.occurrence == actual_item.occurrence =>
            {
                if expected_item.page != actual_item.page {
                    issues.push(ComparisonIssue {
                        kind: ComparisonIssueKind::WrongPage,
                        element_id: expected_item.element_id.clone(),
                        occurrence: expected_item.occurrence,
                        text_preview: expected_item.text_preview.clone(),
                        expected_page: Some(expected_item.page),
                        expected_page_label: None,
                        actual_page: Some(actual_item.page),
                        actual_page_label: None,
                        expected_fragment: Some(expected_item.fragment.clone()),
                        actual_fragment: Some(actual_item.fragment.clone()),
                    });
                }

                if expected_item.fragment != actual_item.fragment {
                    issues.push(ComparisonIssue {
                        kind: ComparisonIssueKind::WrongFragment,
                        element_id: expected_item.element_id.clone(),
                        occurrence: expected_item.occurrence,
                        text_preview: expected_item.text_preview.clone(),
                        expected_page: Some(expected_item.page),
                        expected_page_label: None,
                        actual_page: Some(actual_item.page),
                        actual_page_label: None,
                        expected_fragment: Some(expected_item.fragment.clone()),
                        actual_fragment: Some(actual_item.fragment.clone()),
                    });
                }
            }
            (Some(expected_item), Some(actual_item)) => {
                issues.push(ComparisonIssue {
                    kind: ComparisonIssueKind::MissingOccurrence,
                    element_id: expected_item.element_id.clone(),
                    occurrence: expected_item.occurrence,
                    text_preview: expected_item.text_preview.clone(),
                    expected_page: Some(expected_item.page),
                    expected_page_label: None,
                    actual_page: None,
                    actual_page_label: None,
                    expected_fragment: Some(expected_item.fragment.clone()),
                    actual_fragment: None,
                });
                issues.push(ComparisonIssue {
                    kind: ComparisonIssueKind::UnexpectedOccurrence,
                    element_id: actual_item.element_id.clone(),
                    occurrence: actual_item.occurrence,
                    text_preview: expected_preview_for(&expected_previews, actual_item),
                    expected_page: None,
                    expected_page_label: None,
                    actual_page: Some(actual_item.page),
                    actual_page_label: None,
                    expected_fragment: None,
                    actual_fragment: Some(actual_item.fragment.clone()),
                });
            }
            (Some(expected_item), None) => issues.push(ComparisonIssue {
                kind: ComparisonIssueKind::MissingOccurrence,
                element_id: expected_item.element_id.clone(),
                occurrence: expected_item.occurrence,
                text_preview: expected_item.text_preview.clone(),
                expected_page: Some(expected_item.page),
                expected_page_label: None,
                actual_page: None,
                actual_page_label: None,
                expected_fragment: Some(expected_item.fragment.clone()),
                actual_fragment: None,
            }),
            (None, Some(actual_item)) => issues.push(ComparisonIssue {
                kind: ComparisonIssueKind::UnexpectedOccurrence,
                element_id: actual_item.element_id.clone(),
                occurrence: actual_item.occurrence,
                text_preview: expected_preview_for(&expected_previews, actual_item),
                expected_page: None,
                expected_page_label: None,
                actual_page: Some(actual_item.page),
                actual_page_label: None,
                expected_fragment: None,
                actual_fragment: Some(actual_item.fragment.clone()),
            }),
            (None, None) => {}
        }
    }

    ComparisonReport {
        expected_page_count: expected.pages.len(),
        actual_page_count: actual.pages.len(),
        issues,
    }
}

#[derive(Clone)]
struct Occurrence {
    element_id: String,
    occurrence: usize,
    page: u32,
    fragment: Fragment,
    text_preview: Option<String>,
}

fn collect_actual_occurrences(actual: &PaginatedScreenplay) -> Vec<Occurrence> {
    let mut counters: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::new();

    for page in &actual.pages {
        for item in &page.items {
            let occurrence = counters
                .entry(item.element_id.as_str())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            out.push(Occurrence {
                element_id: item.element_id.clone(),
                occurrence: *occurrence,
                page: page.metadata.number,
                fragment: item.fragment.clone(),
                text_preview: None,
            });
        }
    }

    out
}

fn collect_expected_occurrences(expected: &PageBreakFixture) -> Vec<Occurrence> {
    let mut counters: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::new();

    for page in &expected.pages {
        for item in &page.items {
            let occurrence = counters
                .entry(item.element_id.as_str())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            out.push(Occurrence {
                element_id: item.element_id.clone(),
                occurrence: *occurrence,
                page: page.number,
                fragment: item.fragment.clone(),
                text_preview: item.text_preview.clone(),
            });
        }
    }

    out
}

fn collect_expected_previews(
    expected_occurrences: &[Occurrence],
) -> HashMap<(String, usize), String> {
    expected_occurrences
        .iter()
        .filter_map(|item| {
            item.text_preview.as_ref().map(|preview| {
                ((item.element_id.clone(), item.occurrence), preview.clone())
            })
        })
        .collect()
}

fn expected_preview_for(
    expected_previews: &HashMap<(String, usize), String>,
    actual_item: &Occurrence,
) -> Option<String> {
    expected_previews
        .get(&(actual_item.element_id.clone(), actual_item.occurrence))
        .cloned()
}
