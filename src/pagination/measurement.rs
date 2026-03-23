use crate::pagination::semantic::{
    DialoguePartKind, DialogueUnit, DualDialogueUnit, FlowKind, FlowUnit, LyricUnit,
};

#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementConfig {
    pub chars_per_inch: f32,
    pub lines_per_inch: f32,
    pub action_left_indent_in: f32,
    pub action_right_indent_in: f32,
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
}

impl MeasurementConfig {
    pub fn screenplay_default() -> Self {
        Self {
            chars_per_inch: 10.0,
            lines_per_inch: 6.0,
            action_left_indent_in: 1.50,
            action_right_indent_in: 7.50,
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
        }
    }

    pub fn width_chars_for_flow_kind(&self, kind: &FlowKind) -> usize {
        let (left, right) = match kind {
            FlowKind::Transition => (
                self.transition_left_indent_in,
                self.transition_right_indent_in,
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
}

pub fn measure_flow_unit_lines(unit: &FlowUnit, measurement: &MeasurementConfig) -> u32 {
    measure_text_lines(
        &unit.text,
        measurement.width_chars_for_flow_kind(&unit.kind),
    )
}

pub fn measure_lyric_unit_lines(unit: &LyricUnit, measurement: &MeasurementConfig) -> u32 {
    measure_text_lines(
        &unit.text,
        width_chars(
            measurement.chars_per_inch,
            measurement.lyric_left_indent_in,
            measurement.lyric_right_indent_in,
        ),
    )
}

pub fn measure_dialogue_part_lines(
    kind: &DialoguePartKind,
    text: &str,
    measurement: &MeasurementConfig,
) -> u32 {
    measure_text_lines(text, measurement.width_chars_for_dialogue_part(kind))
}

pub fn measure_dialogue_unit_lines(unit: &DialogueUnit, measurement: &MeasurementConfig) -> u32 {
    unit.parts
        .iter()
        .map(|part| measure_dialogue_part_lines(&part.kind, &part.text, measurement))
        .sum::<u32>()
        .max(1)
}

pub fn measure_dual_dialogue_unit_lines(
    unit: &DualDialogueUnit,
    measurement: &MeasurementConfig,
) -> u32 {
    unit.sides
        .iter()
        .map(|side| measure_dialogue_unit_lines(&side.dialogue, measurement))
        .max()
        .unwrap_or(1)
}

pub fn measure_text_lines(text: &str, width_chars: usize) -> u32 {
    text.lines()
        .map(|line| measure_explicit_line(line, width_chars))
        .sum::<u32>()
        .max(1)
}

fn measure_explicit_line(line: &str, width_chars: usize) -> u32 {
    if width_chars == 0 {
        return 1;
    }

    if line.trim().is_empty() {
        return 1;
    }

    let mut count = 0;
    let mut current_len = 0;

    for word in line.split_whitespace() {
        let word_len = word.chars().count();
        if current_len == 0 {
            current_len = word_len;
            count += 1;
            continue;
        }

        if current_len + 1 + word_len <= width_chars {
            current_len += 1 + word_len;
        } else {
            count += 1;
            current_len = word_len;
        }
    }

    count.max(1)
}

fn width_chars(chars_per_inch: f32, left_indent_in: f32, right_indent_in: f32) -> usize {
    ((right_indent_in - left_indent_in) * chars_per_inch)
        .floor()
        .max(1.0) as usize
}
