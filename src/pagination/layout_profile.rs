use crate::Metadata;

use super::{Alignment, LayoutGeometry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleProfile {
    Screenplay,
    Multicam,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayElementStyle {
    pub left_indent: f32,
    pub right_indent: f32,
    pub spacing_before: f32,
    pub line_spacing: f32,
    pub alignment: Alignment,
    pub starts_new_page: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayElementStyles {
    pub action: ScreenplayElementStyle,
    pub scene_heading: ScreenplayElementStyle,
    pub character: ScreenplayElementStyle,
    pub dialogue: ScreenplayElementStyle,
    pub parenthetical: ScreenplayElementStyle,
    pub transition: ScreenplayElementStyle,
    pub lyric: ScreenplayElementStyle,
    pub cold_opening: ScreenplayElementStyle,
    pub new_act: ScreenplayElementStyle,
    pub end_of_act: ScreenplayElementStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayLayoutProfile {
    pub style_profile: StyleProfile,
    pub styles: ScreenplayElementStyles,
}

impl ScreenplayLayoutProfile {
    pub fn from_metadata(metadata: &Metadata) -> Self {
        let mut profile = Self::default_screenplay();

        if let Some(options) = metadata.get("fmt").and_then(|values| values.first()) {
            let options = options.plain_text();
            let tokens = options.split_whitespace().collect::<Vec<_>>();

            // Apply base templates first, then explicit geometry knobs so
            // author-supplied overrides win regardless of token order.
            for option in &tokens {
                apply_fmt_template_option(&mut profile, option);
            }
            for option in &tokens {
                apply_fmt_geometry_override_option(&mut profile, option);
            }
        }

        profile
    }

    pub fn to_pagination_geometry(&self) -> LayoutGeometry {
        let mut geometry = LayoutGeometry::default();

        geometry.action_left = self.styles.action.left_indent;
        geometry.action_right = self.styles.action.right_indent;
        geometry.action_spacing_before = self.styles.action.spacing_before;
        geometry.action_alignment = self.styles.action.alignment;
        geometry.action_line_height = self.styles.action.line_spacing;

        geometry.cold_opening_left = self.styles.cold_opening.left_indent;
        geometry.cold_opening_right = self.styles.cold_opening.right_indent;
        geometry.cold_opening_spacing_before = self.styles.cold_opening.spacing_before;
        geometry.cold_opening_alignment = self.styles.cold_opening.alignment;
        geometry.cold_opening_line_height = self.styles.cold_opening.line_spacing;

        geometry.new_act_left = self.styles.new_act.left_indent;
        geometry.new_act_right = self.styles.new_act.right_indent;
        geometry.new_act_spacing_before = self.styles.new_act.spacing_before;
        geometry.new_act_alignment = self.styles.new_act.alignment;
        geometry.new_act_line_height = self.styles.new_act.line_spacing;

        geometry.end_of_act_left = self.styles.end_of_act.left_indent;
        geometry.end_of_act_right = self.styles.end_of_act.right_indent;
        geometry.end_of_act_spacing_before = self.styles.end_of_act.spacing_before;
        geometry.end_of_act_alignment = self.styles.end_of_act.alignment;
        geometry.end_of_act_line_height = self.styles.end_of_act.line_spacing;

        geometry.character_left = self.styles.character.left_indent;
        geometry.character_right = self.styles.character.right_indent;
        geometry.character_spacing_before = self.styles.character.spacing_before;
        geometry.character_alignment = self.styles.character.alignment;
        geometry.character_line_height = self.styles.character.line_spacing;

        geometry.dialogue_left = self.styles.dialogue.left_indent;
        geometry.dialogue_right = self.styles.dialogue.right_indent;
        geometry.dialogue_alignment = self.styles.dialogue.alignment;
        geometry.dialogue_line_height = self.styles.dialogue.line_spacing;

        geometry.parenthetical_left = self.styles.parenthetical.left_indent;
        geometry.parenthetical_right = self.styles.parenthetical.right_indent;
        geometry.parenthetical_alignment = self.styles.parenthetical.alignment;
        geometry.parenthetical_line_height = self.styles.parenthetical.line_spacing;

        geometry.transition_left = self.styles.transition.left_indent;
        geometry.transition_right = self.styles.transition.right_indent;
        geometry.transition_spacing_before = self.styles.transition.spacing_before;
        geometry.transition_alignment = self.styles.transition.alignment;
        geometry.transition_line_height = self.styles.transition.line_spacing;

        geometry.lyric_left = self.styles.lyric.left_indent;
        geometry.lyric_right = self.styles.lyric.right_indent;
        geometry.lyric_spacing_before = self.styles.lyric.spacing_before;
        geometry.lyric_alignment = self.styles.lyric.alignment;
        geometry.lyric_line_height = self.styles.lyric.line_spacing;

        geometry.scene_heading_spacing_before = self.styles.scene_heading.spacing_before;
        geometry.scene_heading_alignment = self.styles.scene_heading.alignment;
        geometry.scene_heading_line_height = self.styles.scene_heading.line_spacing;

        geometry.line_height = self.styles.dialogue.line_spacing;

        geometry
    }

    fn default_screenplay() -> Self {
        Self {
            style_profile: StyleProfile::Screenplay,
            styles: ScreenplayElementStyles {
                action: ScreenplayElementStyle {
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                scene_heading: ScreenplayElementStyle {
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 2.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                character: ScreenplayElementStyle {
                    left_indent: 3.5,
                    right_indent: 7.25,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                dialogue: ScreenplayElementStyle {
                    left_indent: 2.5,
                    right_indent: 6.0,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                parenthetical: ScreenplayElementStyle {
                    left_indent: 3.0,
                    right_indent: 5.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                transition: ScreenplayElementStyle {
                    left_indent: 5.5,
                    right_indent: 7.1,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Right,
                    starts_new_page: false,
                },
                lyric: ScreenplayElementStyle {
                    left_indent: 2.5,
                    right_indent: 7.375,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                },
                cold_opening: ScreenplayElementStyle {
                    left_indent: 1.0,
                    right_indent: 7.5,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: false,
                },
                new_act: ScreenplayElementStyle {
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: true,
                },
                end_of_act: ScreenplayElementStyle {
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 2.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: false,
                },
            },
        }
    }
}

fn apply_fmt_template_option(profile: &mut ScreenplayLayoutProfile, option: &str) {
    if option.eq_ignore_ascii_case("multicam") {
        profile.style_profile = StyleProfile::Multicam;
        profile.styles.dialogue.line_spacing = 2.0;
        profile.styles.dialogue.left_indent = 2.25;
        profile.styles.character.right_indent = 6.25;
        profile.styles.parenthetical.left_indent = 2.75;
        profile.styles.transition.right_indent = 7.25;
    }
}

fn apply_fmt_geometry_override_option(profile: &mut ScreenplayLayoutProfile, option: &str) {
    if option.eq_ignore_ascii_case("ssbsh") {
        profile.styles.scene_heading.spacing_before = 1.0;
    } else if option.eq_ignore_ascii_case("dsd") {
        profile.styles.dialogue.line_spacing = 2.0;
    } else if option.eq_ignore_ascii_case("no-auto-act-breaks") {
        profile.styles.new_act.starts_new_page = false;
    } else if let Some(value) = option.strip_prefix("dl-") {
        if let Ok(indent) = value.parse::<f32>() {
            profile.styles.dialogue.left_indent = indent;
        }
    } else if let Some(value) = option.strip_prefix("dr-") {
        if let Ok(indent) = value.parse::<f32>() {
            profile.styles.dialogue.right_indent = indent;
        }
    }
}
