use crate::{ElementText, TextRun};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyledRun {
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub styles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyledText {
    pub plain_text: String,
    pub runs: Vec<StyledRun>,
}

impl StyledText {
    pub fn from_element_text(text: &ElementText) -> Option<Self> {
        match text {
            ElementText::Plain(_) => None,
            ElementText::Styled(runs) => Some(Self {
                plain_text: runs.iter().map(|run| run.content.as_str()).collect(),
                runs: runs.iter().map(styled_run_from_text_run).collect(),
            }),
        }
    }

    pub fn slice(&self, start_offset: usize, end_offset: usize) -> Self {
        if start_offset >= end_offset {
            return Self {
                plain_text: String::new(),
                runs: Vec::new(),
            };
        }

        let plain_text = self.plain_text[start_offset..end_offset].to_string();
        let mut runs = Vec::new();
        let mut run_start = 0usize;

        for run in &self.runs {
            let run_end = run_start + run.text.len();
            let slice_start = run_start.max(start_offset);
            let slice_end = run_end.min(end_offset);

            if slice_start < slice_end {
                runs.push(StyledRun {
                    text: run.text[(slice_start - run_start)..(slice_end - run_start)].to_string(),
                    styles: run.styles.clone(),
                });
            }

            run_start = run_end;
        }

        Self { plain_text, runs }
    }
}

fn styled_run_from_text_run(run: &TextRun) -> StyledRun {
    let mut styles: Vec<String> = run.text_style.iter().cloned().collect();
    styles.sort();

    StyledRun {
        text: run.content.clone(),
        styles,
    }
}
