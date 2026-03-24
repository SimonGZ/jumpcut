use std::collections::BTreeMap;

use crate::pagination::semantic::{
    DialoguePartKind, DialogueUnit, DualDialogueUnit, FlowKind, FlowUnit, LyricUnit, SemanticUnit,
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FdxExtractedSettings {
    pub paragraph_styles: BTreeMap<String, FdxParagraphStyle>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FdxParagraphStyle {
    pub left_indent: f32,
    pub right_indent: f32,
    pub space_before: f32,
    pub spacing: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitMeasurement {
    pub content_lines: u32,
    pub top_spacing_lines: u32,
    pub bottom_spacing_lines: u32,
}

impl UnitMeasurement {
    pub fn placement_lines_with_prev(
        &self,
        previous: Option<&UnitMeasurement>,
    ) -> u32 {
        self.content_lines + boundary_spacing_lines(previous, Some(self))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementConfig {
    pub chars_per_inch: f32,
    pub lines_per_inch: f32,
    pub action_left_indent_in: f32,
    pub action_right_indent_in: f32,
    pub scene_heading_left_indent_in: f32,
    pub scene_heading_right_indent_in: f32,
    pub cold_opening_left_indent_in: f32,
    pub cold_opening_right_indent_in: f32,
    pub new_act_left_indent_in: f32,
    pub new_act_right_indent_in: f32,
    pub end_of_act_left_indent_in: f32,
    pub end_of_act_right_indent_in: f32,
    pub dialogue_left_indent_in: f32,
    pub dialogue_right_indent_in: f32,
    pub character_left_indent_in: f32,
    pub character_right_indent_in: f32,
    pub parenthetical_left_indent_in: f32,
    pub parenthetical_right_indent_in: f32,
    pub lyric_left_indent_in: f32,
    pub lyric_right_indent_in: f32,
    pub transition_left_indent_in: f32,
    pub transition_right_indent_in: f32,
    pub action_top_spacing_lines: u32,
    pub action_bottom_spacing_lines: u32,
    pub scene_heading_top_spacing_lines: u32,
    pub scene_heading_bottom_spacing_lines: u32,
    pub cold_opening_top_spacing_lines: u32,
    pub cold_opening_bottom_spacing_lines: u32,
    pub new_act_top_spacing_lines: u32,
    pub new_act_bottom_spacing_lines: u32,
    pub end_of_act_top_spacing_lines: u32,
    pub end_of_act_bottom_spacing_lines: u32,
    pub transition_top_spacing_lines: u32,
    pub transition_bottom_spacing_lines: u32,
    pub dialogue_top_spacing_lines: u32,
    pub dialogue_bottom_spacing_lines: u32,
    pub lyric_top_spacing_lines: u32,
    pub lyric_bottom_spacing_lines: u32,
}

impl MeasurementConfig {
    pub fn screenplay_default() -> Self {
        Self {
            chars_per_inch: 10.0,
            lines_per_inch: 6.0,
            action_left_indent_in: 1.50,
            action_right_indent_in: 7.50,
            scene_heading_left_indent_in: 1.50,
            scene_heading_right_indent_in: 7.50,
            cold_opening_left_indent_in: 1.50,
            cold_opening_right_indent_in: 7.50,
            new_act_left_indent_in: 1.50,
            new_act_right_indent_in: 7.50,
            end_of_act_left_indent_in: 1.50,
            end_of_act_right_indent_in: 7.50,
            dialogue_left_indent_in: 2.50,
            dialogue_right_indent_in: 5.30,
            character_left_indent_in: 3.50,
            character_right_indent_in: 5.50,
            parenthetical_left_indent_in: 3.00,
            parenthetical_right_indent_in: 5.00,
            lyric_left_indent_in: 2.50,
            lyric_right_indent_in: 7.38,
            transition_left_indent_in: 5.50,
            transition_right_indent_in: 7.10,
            action_top_spacing_lines: 0,
            action_bottom_spacing_lines: 0,
            scene_heading_top_spacing_lines: 0,
            scene_heading_bottom_spacing_lines: 0,
            cold_opening_top_spacing_lines: 0,
            cold_opening_bottom_spacing_lines: 0,
            new_act_top_spacing_lines: 0,
            new_act_bottom_spacing_lines: 0,
            end_of_act_top_spacing_lines: 0,
            end_of_act_bottom_spacing_lines: 0,
            transition_top_spacing_lines: 0,
            transition_bottom_spacing_lines: 0,
            dialogue_top_spacing_lines: 0,
            dialogue_bottom_spacing_lines: 0,
            lyric_top_spacing_lines: 0,
            lyric_bottom_spacing_lines: 0,
        }
    }

    pub fn width_chars_for_flow_kind(&self, kind: &FlowKind) -> usize {
        let (left, right) = match kind {
            FlowKind::SceneHeading => (
                self.scene_heading_left_indent_in,
                self.scene_heading_right_indent_in,
            ),
            FlowKind::Transition => (
                self.transition_left_indent_in,
                self.transition_right_indent_in,
            ),
            FlowKind::ColdOpening => (
                self.cold_opening_left_indent_in,
                self.cold_opening_right_indent_in,
            ),
            FlowKind::NewAct => (
                self.new_act_left_indent_in,
                self.new_act_right_indent_in,
            ),
            FlowKind::EndOfAct => (
                self.end_of_act_left_indent_in,
                self.end_of_act_right_indent_in,
            ),
            _ => (self.action_left_indent_in, self.action_right_indent_in),
        };
        width_chars(self.chars_per_inch, left, right)
    }

    pub fn width_chars_for_dialogue_part(&self, kind: &DialoguePartKind) -> usize {
        let (left, right) = match kind {
            DialoguePartKind::Character => (
                self.character_left_indent_in,
                self.character_right_indent_in,
            ),
            DialoguePartKind::Parenthetical => (
                self.parenthetical_left_indent_in,
                self.parenthetical_right_indent_in,
            ),
            DialoguePartKind::Lyric => (self.lyric_left_indent_in, self.lyric_right_indent_in),
            DialoguePartKind::Dialogue => (
                self.dialogue_left_indent_in,
                self.dialogue_right_indent_in,
            ),
        };
        width_chars(self.chars_per_inch, left, right)
    }

    pub fn spacing_for_flow_kind(&self, kind: &FlowKind) -> (u32, u32) {
        match kind {
            FlowKind::SceneHeading => (
                self.scene_heading_top_spacing_lines,
                self.scene_heading_bottom_spacing_lines,
            ),
            FlowKind::ColdOpening => (
                self.cold_opening_top_spacing_lines,
                self.cold_opening_bottom_spacing_lines,
            ),
            FlowKind::NewAct => (
                self.new_act_top_spacing_lines,
                self.new_act_bottom_spacing_lines,
            ),
            FlowKind::EndOfAct => (
                self.end_of_act_top_spacing_lines,
                self.end_of_act_bottom_spacing_lines,
            ),
            FlowKind::Transition => (
                self.transition_top_spacing_lines,
                self.transition_bottom_spacing_lines,
            ),
            _ => (
                self.action_top_spacing_lines,
                self.action_bottom_spacing_lines,
            ),
        }
    }

    pub fn spacing_for_dialogue_unit(&self) -> (u32, u32) {
        (
            self.dialogue_top_spacing_lines,
            self.dialogue_bottom_spacing_lines,
        )
    }

    pub fn spacing_for_lyric_unit(&self) -> (u32, u32) {
        (self.lyric_top_spacing_lines, self.lyric_bottom_spacing_lines)
    }

    pub fn from_fdx_settings(settings: &FdxExtractedSettings) -> Self {
        let mut measurement = Self::screenplay_default();

        if let Some(style) = settings.paragraph_styles.get("Action") {
            measurement.action_left_indent_in = style.left_indent;
            measurement.action_right_indent_in = style.right_indent;
            measurement.action_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.action_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("Scene Heading") {
            measurement.scene_heading_left_indent_in = style.left_indent;
            measurement.scene_heading_right_indent_in = style.right_indent;
            measurement.scene_heading_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.scene_heading_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("Cold Opening") {
            measurement.cold_opening_left_indent_in = style.left_indent;
            measurement.cold_opening_right_indent_in = style.right_indent;
            measurement.cold_opening_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.cold_opening_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("New Act") {
            measurement.new_act_left_indent_in = style.left_indent;
            measurement.new_act_right_indent_in = style.right_indent;
            measurement.new_act_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.new_act_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("End of Act") {
            measurement.end_of_act_left_indent_in = style.left_indent;
            measurement.end_of_act_right_indent_in = style.right_indent;
            measurement.end_of_act_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.end_of_act_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("Dialogue") {
            measurement.dialogue_left_indent_in = style.left_indent;
            measurement.dialogue_right_indent_in = style.right_indent;
        }
        if let Some(style) = settings.paragraph_styles.get("Character") {
            measurement.character_left_indent_in = style.left_indent;
            measurement.character_right_indent_in = style.right_indent;
            measurement.dialogue_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.dialogue_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("Parenthetical") {
            measurement.parenthetical_left_indent_in = style.left_indent;
            measurement.parenthetical_right_indent_in = style.right_indent;
        }
        if let Some(style) = settings.paragraph_styles.get("Lyric") {
            measurement.lyric_left_indent_in = style.left_indent;
            measurement.lyric_right_indent_in = style.right_indent;
            measurement.lyric_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.lyric_bottom_spacing_lines = 0;
        }
        if let Some(style) = settings.paragraph_styles.get("Transition") {
            measurement.transition_left_indent_in = style.left_indent;
            measurement.transition_right_indent_in = style.right_indent;
            measurement.transition_top_spacing_lines =
                spacing_lines_from_points(style.space_before, measurement.lines_per_inch);
            measurement.transition_bottom_spacing_lines = 0;
        }

        measurement
    }
}

fn spacing_lines_from_points(space_before_points: f32, lines_per_inch: f32) -> u32 {
    let points_per_line = 72.0 / lines_per_inch;
    (space_before_points / points_per_line).round() as u32
}

pub fn measure_flow_unit(unit: &FlowUnit, measurement: &MeasurementConfig) -> UnitMeasurement {
    let (top_spacing_lines, bottom_spacing_lines) =
        measurement.spacing_for_flow_kind(&unit.kind);
    UnitMeasurement {
        content_lines: measure_text_lines(
            &unit.text,
            measurement.width_chars_for_flow_kind(&unit.kind),
        ),
        top_spacing_lines,
        bottom_spacing_lines,
    }
}

pub fn measure_flow_unit_lines(unit: &FlowUnit, measurement: &MeasurementConfig) -> u32 {
    measure_flow_unit(unit, measurement).content_lines
}

pub fn measure_lyric_unit(unit: &LyricUnit, measurement: &MeasurementConfig) -> UnitMeasurement {
    let (top_spacing_lines, bottom_spacing_lines) = measurement.spacing_for_lyric_unit();
    UnitMeasurement {
        content_lines: measure_text_lines(
            &unit.text,
            width_chars(
                measurement.chars_per_inch,
                measurement.lyric_left_indent_in,
                measurement.lyric_right_indent_in,
            ),
        ),
        top_spacing_lines,
        bottom_spacing_lines,
    }
}

pub fn measure_lyric_unit_lines(unit: &LyricUnit, measurement: &MeasurementConfig) -> u32 {
    measure_lyric_unit(unit, measurement).content_lines
}

pub fn measure_dialogue_part_lines(
    kind: &DialoguePartKind,
    text: &str,
    measurement: &MeasurementConfig,
) -> u32 {
    wrap_text_lines_with_policy(
        text,
        measurement.width_chars_for_dialogue_part(kind),
        preserves_internal_spaces(kind),
    )
    .len() as u32
}

pub fn measure_dialogue_unit(
    unit: &DialogueUnit,
    measurement: &MeasurementConfig,
) -> UnitMeasurement {
    let (top_spacing_lines, bottom_spacing_lines) = measurement.spacing_for_dialogue_unit();
    UnitMeasurement {
        content_lines: unit
            .parts
            .iter()
            .map(|part| measure_dialogue_part_lines(&part.kind, &part.text, measurement))
            .sum::<u32>()
            .max(1),
        top_spacing_lines,
        bottom_spacing_lines,
    }
}

pub fn measure_dialogue_unit_lines(unit: &DialogueUnit, measurement: &MeasurementConfig) -> u32 {
    measure_dialogue_unit(unit, measurement).content_lines
}

pub fn measure_dual_dialogue_unit(
    unit: &DualDialogueUnit,
    measurement: &MeasurementConfig,
) -> UnitMeasurement {
    let (top_spacing_lines, bottom_spacing_lines) = measurement.spacing_for_dialogue_unit();
    UnitMeasurement {
        content_lines: unit
            .sides
            .iter()
            .map(|side| measure_dialogue_unit_lines(&side.dialogue, measurement))
            .max()
            .unwrap_or(1),
        top_spacing_lines,
        bottom_spacing_lines,
    }
}

pub fn measure_dual_dialogue_unit_lines(
    unit: &DualDialogueUnit,
    measurement: &MeasurementConfig,
) -> u32 {
    measure_dual_dialogue_unit(unit, measurement).content_lines
}

pub fn measure_semantic_unit(
    unit: &SemanticUnit,
    measurement: &MeasurementConfig,
) -> Option<UnitMeasurement> {
    match unit {
        SemanticUnit::PageStart(_) => None,
        SemanticUnit::Flow(unit) => Some(measure_flow_unit(unit, measurement)),
        SemanticUnit::Lyric(unit) => Some(measure_lyric_unit(unit, measurement)),
        SemanticUnit::Dialogue(unit) => Some(measure_dialogue_unit(unit, measurement)),
        SemanticUnit::DualDialogue(unit) => Some(measure_dual_dialogue_unit(unit, measurement)),
    }
}

pub fn boundary_spacing_lines(
    previous: Option<&UnitMeasurement>,
    current: Option<&UnitMeasurement>,
) -> u32 {
    match (previous, current) {
        (Some(previous), Some(current)) => previous
            .bottom_spacing_lines
            .max(current.top_spacing_lines),
        _ => 0,
    }
}

pub fn measure_text_lines(text: &str, width_chars: usize) -> u32 {
    wrap_text_lines_with_policy(text, width_chars, false).len() as u32
}

pub fn wrap_text_lines_with_policy(
    text: &str,
    width_chars: usize,
    preserve_internal_spaces: bool,
) -> Vec<String> {
    let wrapped = text
        .lines()
        .flat_map(|line| {
            if preserve_internal_spaces {
                wrap_explicit_line_preserving_spaces(line, width_chars)
            } else {
                wrap_explicit_line_collapsing_spaces(line, width_chars)
            }
        })
        .collect::<Vec<_>>();

    if wrapped.is_empty() {
        vec![String::new()]
    } else {
        wrapped
    }
}

fn wrap_explicit_line_collapsing_spaces(
    line: &str,
    width_chars: usize,
) -> Vec<String> {
    if width_chars == 0 {
        return vec![line.trim().to_string()];
    }

    if line.trim().is_empty() {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();

    for word in line.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.chars().count() + 1 + word.chars().count() <= width_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            wrapped.push(current);
            current = word.to_string();
        }
    }

    if current.is_empty() {
        wrapped.push(String::new());
    } else {
        wrapped.push(current);
    }

    wrapped
}

fn wrap_explicit_line_preserving_spaces(
    line: &str,
    width_chars: usize,
) -> Vec<String> {
    if width_chars == 0 {
        return vec![line.trim_end().to_string()];
    }

    let line = line.trim_end();
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        let mut token = String::from(ch);
        let is_whitespace = ch.is_whitespace();

        while let Some(next) = chars.peek() {
            if next.is_whitespace() == is_whitespace {
                token.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        if current.is_empty() && is_whitespace {
            continue;
        }

        if current.chars().count() + token.chars().count() <= width_chars {
            current.push_str(&token);
            continue;
        }

        if is_whitespace {
            if !current.is_empty() {
                wrapped.push(current.trim_end().to_string());
                current.clear();
            }
            continue;
        }

        if !current.is_empty() {
            wrapped.push(current.trim_end().to_string());
        }
        current = token;
    }

    if current.is_empty() {
        wrapped.push(String::new());
    } else {
        wrapped.push(current.trim_end().to_string());
    }

    wrapped
}

fn preserves_internal_spaces(kind: &DialoguePartKind) -> bool {
    matches!(kind, DialoguePartKind::Dialogue | DialoguePartKind::Lyric)
}

fn width_chars(chars_per_inch: f32, left_indent_in: f32, right_indent_in: f32) -> usize {
    ((right_indent_in - left_indent_in) * chars_per_inch)
        .floor()
        .max(1.0) as usize
}
