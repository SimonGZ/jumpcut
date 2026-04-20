use crate::{
    ElementLayoutOverrides, ImportedAlignment, ImportedDialogueContinueds, ImportedElementKind,
    ImportedElementStyle, ImportedLayoutOverrides, ImportedSceneContinueds, Metadata, Screenplay,
};

use super::wrapping::InterruptionDashWrap;
use super::{Alignment, LayoutGeometry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleProfile {
    Screenplay,
    Multicam,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayElementStyle {
    pub first_indent: f32,
    pub left_indent: f32,
    pub right_indent: f32,
    pub spacing_before: f32,
    pub line_spacing: f32,
    pub alignment: Alignment,
    pub starts_new_page: bool,
    pub underline: bool,
    pub bold: bool,
    pub italic: bool,
}

impl ScreenplayElementStyle {
    pub fn apply_element_layout_overrides(
        &self,
        overrides: &ElementLayoutOverrides,
    ) -> ScreenplayElementStyle {
        let mut effective = self.clone();

        if let Some(space_before_delta) = overrides.space_before_delta {
            effective.spacing_before += space_before_delta;
        }
        if let Some(right_indent_delta) = overrides.right_indent_delta {
            effective.right_indent += right_indent_delta;
        }

        effective
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayElementStyles {
    pub action: ScreenplayElementStyle,
    pub scene_heading: ScreenplayElementStyle,
    pub character: ScreenplayElementStyle,
    pub dialogue: ScreenplayElementStyle,
    pub parenthetical: ScreenplayElementStyle,
    pub dual_dialogue_left_character: ScreenplayElementStyle,
    pub dual_dialogue_left_dialogue: ScreenplayElementStyle,
    pub dual_dialogue_left_parenthetical: ScreenplayElementStyle,
    pub dual_dialogue_right_character: ScreenplayElementStyle,
    pub dual_dialogue_right_dialogue: ScreenplayElementStyle,
    pub dual_dialogue_right_parenthetical: ScreenplayElementStyle,
    pub transition: ScreenplayElementStyle,
    pub lyric: ScreenplayElementStyle,
    pub cold_opening: ScreenplayElementStyle,
    pub new_act: ScreenplayElementStyle,
    pub end_of_act: ScreenplayElementStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenplayLayoutProfile {
    pub style_profile: StyleProfile,
    pub interruption_dash_wrap: InterruptionDashWrap,
    pub dual_dialogue_counts_for_contd: bool,
    pub automatic_character_continueds: bool,
    pub styles: ScreenplayElementStyles,
    pub continueds: ScreenplayContinueds,
    pub page_width: f32,
    pub page_height: f32,
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub header_margin: f32,
    pub footer_margin: f32,
    pub lines_per_page: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenplayContinueds {
    pub dialogue: DialogueContinueds,
    pub scene: SceneContinueds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueContinueds {
    pub top_of_next: bool,
    pub bottom_of_page: bool,
    pub top_text: String,
    pub bottom_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneContinueds {
    pub top_of_next: bool,
    pub bottom_of_page: bool,
    pub continued_number: bool,
    pub top_text: String,
    pub bottom_text: String,
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

    pub fn from_screenplay(screenplay: &Screenplay) -> Self {
        let mut profile = Self::from_metadata(&screenplay.metadata);
        if let Some(imported_layout) = &screenplay.imported_layout {
            profile.overlay_imported_layout(imported_layout);
        }
        profile
    }

    pub fn to_pagination_geometry(&self) -> LayoutGeometry {
        let mut geometry = LayoutGeometry::default();

        geometry.action_left = self.styles.action.left_indent;
        geometry.action_first_indent = self.styles.action.first_indent;
        geometry.action_right = self.styles.action.right_indent;
        geometry.action_spacing_before = self.styles.action.spacing_before;
        geometry.action_alignment = self.styles.action.alignment;
        geometry.action_line_height = self.styles.action.line_spacing;

        geometry.cold_opening_left = self.styles.cold_opening.left_indent;
        geometry.cold_opening_first_indent = self.styles.cold_opening.first_indent;
        geometry.cold_opening_right = self.styles.cold_opening.right_indent;
        geometry.cold_opening_spacing_before = self.styles.cold_opening.spacing_before;
        geometry.cold_opening_alignment = self.styles.cold_opening.alignment;
        geometry.cold_opening_line_height = self.styles.cold_opening.line_spacing;

        geometry.new_act_left = self.styles.new_act.left_indent;
        geometry.new_act_first_indent = self.styles.new_act.first_indent;
        geometry.new_act_right = self.styles.new_act.right_indent;
        geometry.new_act_spacing_before = self.styles.new_act.spacing_before;
        geometry.new_act_alignment = self.styles.new_act.alignment;
        geometry.new_act_line_height = self.styles.new_act.line_spacing;

        geometry.end_of_act_left = self.styles.end_of_act.left_indent;
        geometry.end_of_act_first_indent = self.styles.end_of_act.first_indent;
        geometry.end_of_act_right = self.styles.end_of_act.right_indent;
        geometry.end_of_act_spacing_before = self.styles.end_of_act.spacing_before;
        geometry.end_of_act_alignment = self.styles.end_of_act.alignment;
        geometry.end_of_act_line_height = self.styles.end_of_act.line_spacing;

        geometry.dual_dialogue_left_character_left =
            self.styles.dual_dialogue_left_character.left_indent;
        geometry.dual_dialogue_left_character_first_indent =
            self.styles.dual_dialogue_left_character.first_indent;
        geometry.dual_dialogue_left_character_right =
            self.styles.dual_dialogue_left_character.right_indent;
        geometry.dual_dialogue_left_first_indent =
            self.styles.dual_dialogue_left_dialogue.first_indent;
        geometry.dual_dialogue_left_left = self.styles.dual_dialogue_left_dialogue.left_indent;
        geometry.dual_dialogue_left_right = self.styles.dual_dialogue_left_dialogue.right_indent;
        geometry.dual_dialogue_left_parenthetical_first_indent =
            self.styles.dual_dialogue_left_parenthetical.first_indent;
        geometry.dual_dialogue_left_parenthetical_left =
            self.styles.dual_dialogue_left_parenthetical.left_indent;
        geometry.dual_dialogue_left_parenthetical_right =
            self.styles.dual_dialogue_left_parenthetical.right_indent;

        geometry.dual_dialogue_right_character_left =
            self.styles.dual_dialogue_right_character.left_indent;
        geometry.dual_dialogue_right_character_first_indent =
            self.styles.dual_dialogue_right_character.first_indent;
        geometry.dual_dialogue_right_character_right =
            self.styles.dual_dialogue_right_character.right_indent;
        geometry.dual_dialogue_right_first_indent =
            self.styles.dual_dialogue_right_dialogue.first_indent;
        geometry.dual_dialogue_right_left = self.styles.dual_dialogue_right_dialogue.left_indent;
        geometry.dual_dialogue_right_right = self.styles.dual_dialogue_right_dialogue.right_indent;
        geometry.dual_dialogue_right_parenthetical_first_indent =
            self.styles.dual_dialogue_right_parenthetical.first_indent;
        geometry.dual_dialogue_right_parenthetical_left =
            self.styles.dual_dialogue_right_parenthetical.left_indent;
        geometry.dual_dialogue_right_parenthetical_right =
            self.styles.dual_dialogue_right_parenthetical.right_indent;
        geometry.character_left = self.styles.character.left_indent;
        geometry.character_first_indent = self.styles.character.first_indent;
        geometry.character_right = self.styles.character.right_indent;
        geometry.character_spacing_before = self.styles.character.spacing_before;
        geometry.character_alignment = self.styles.character.alignment;
        geometry.character_line_height = self.styles.character.line_spacing;

        geometry.dialogue_left = self.styles.dialogue.left_indent;
        geometry.dialogue_first_indent = self.styles.dialogue.first_indent;
        geometry.dialogue_right = self.styles.dialogue.right_indent;
        geometry.dialogue_alignment = self.styles.dialogue.alignment;
        geometry.dialogue_line_height = self.styles.dialogue.line_spacing;

        geometry.parenthetical_left = self.styles.parenthetical.left_indent;
        geometry.parenthetical_first_indent = self.styles.parenthetical.first_indent;
        geometry.parenthetical_right = self.styles.parenthetical.right_indent;
        geometry.parenthetical_alignment = self.styles.parenthetical.alignment;
        geometry.parenthetical_line_height = self.styles.parenthetical.line_spacing;

        geometry.transition_left = self.styles.transition.left_indent;
        geometry.transition_first_indent = self.styles.transition.first_indent;
        geometry.transition_right = self.styles.transition.right_indent;
        geometry.transition_spacing_before = self.styles.transition.spacing_before;
        geometry.transition_alignment = self.styles.transition.alignment;
        geometry.transition_line_height = self.styles.transition.line_spacing;

        geometry.lyric_left = self.styles.lyric.left_indent;
        geometry.lyric_first_indent = self.styles.lyric.first_indent;
        geometry.lyric_right = self.styles.lyric.right_indent;
        geometry.lyric_spacing_before = self.styles.lyric.spacing_before;
        geometry.lyric_alignment = self.styles.lyric.alignment;
        geometry.lyric_line_height = self.styles.lyric.line_spacing;

        geometry.scene_heading_spacing_before = self.styles.scene_heading.spacing_before;
        geometry.scene_heading_alignment = self.styles.scene_heading.alignment;
        geometry.scene_heading_line_height = self.styles.scene_heading.line_spacing;

        geometry.line_height = self.styles.dialogue.line_spacing;
        geometry.page_width = self.page_width;
        geometry.page_height = self.page_height;
        geometry.top_margin = self.top_margin;
        geometry.bottom_margin = self.bottom_margin;
        geometry.header_margin = self.header_margin;
        geometry.footer_margin = self.footer_margin;
        geometry.lines_per_page = self.lines_per_page;

        geometry
    }

    fn default_screenplay() -> Self {
        Self {
            style_profile: StyleProfile::Screenplay,
            interruption_dash_wrap: InterruptionDashWrap::FinalDraft,
            dual_dialogue_counts_for_contd: true,
            automatic_character_continueds: true,
            styles: ScreenplayElementStyles {
                action: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                scene_heading: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 2.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                character: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 3.5,
                    right_indent: 7.25,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dialogue: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 2.5,
                    right_indent: 6.0,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                parenthetical: ScreenplayElementStyle {
                    first_indent: -0.1,
                    left_indent: 3.0,
                    right_indent: 5.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_left_character: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 2.5,
                    right_indent: 4.875,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_left_dialogue: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 4.375,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_left_parenthetical: ScreenplayElementStyle {
                    first_indent: -0.1,
                    left_indent: 1.75,
                    right_indent: 4.125,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_right_character: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 5.875,
                    right_indent: 7.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_right_dialogue: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 4.625,
                    right_indent: 7.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                dual_dialogue_right_parenthetical: ScreenplayElementStyle {
                    first_indent: -0.1,
                    left_indent: 4.875,
                    right_indent: 7.25,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                transition: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 5.5,
                    right_indent: 7.1,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Right,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: false,
                },
                lyric: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 2.5,
                    right_indent: 7.375,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Left,
                    starts_new_page: false,
                    underline: false,
                    bold: false,
                    italic: true,
                },
                cold_opening: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 1.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: false,
                    underline: true,
                    bold: false,
                    italic: false,
                },
                new_act: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 0.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: true,
                    underline: true,
                    bold: false,
                    italic: false,
                },
                end_of_act: ScreenplayElementStyle {
                    first_indent: 0.0,
                    left_indent: 1.5,
                    right_indent: 7.5,
                    spacing_before: 2.0,
                    line_spacing: 1.0,
                    alignment: Alignment::Center,
                    starts_new_page: false,
                    underline: true,
                    bold: false,
                    italic: false,
                },
            },
            continueds: ScreenplayContinueds {
                dialogue: DialogueContinueds {
                    top_of_next: true,
                    bottom_of_page: true,
                    top_text: "(CONT'D)".to_string(),
                    bottom_text: "(MORE)".to_string(),
                },
                scene: SceneContinueds {
                    top_of_next: false,
                    bottom_of_page: false,
                    continued_number: false,
                    top_text: "CONTINUED:".to_string(),
                    bottom_text: "(CONTINUED)".to_string(),
                },
            },
            page_width: 8.5,
            page_height: 11.0,
            top_margin: 1.0,
            bottom_margin: 1.0,
            header_margin: 0.5,
            footer_margin: 0.5,
            lines_per_page: 54.0,
        }
    }

    fn overlay_imported_layout(&mut self, imported_layout: &ImportedLayoutOverrides) {
        if let Some(page_width) = imported_layout.page.page_width {
            self.page_width = page_width;
        }
        if let Some(page_height) = imported_layout.page.page_height {
            self.page_height = page_height;
        }
        if let Some(top_margin) = imported_layout.page.top_margin {
            self.top_margin = top_margin;
        }
        if let Some(bottom_margin) = imported_layout.page.bottom_margin {
            self.bottom_margin = bottom_margin;
        }
        if let Some(header_margin) = imported_layout.page.header_margin {
            self.header_margin = header_margin;
        }
        if let Some(footer_margin) = imported_layout.page.footer_margin {
            self.footer_margin = footer_margin;
        }

        for (kind, style) in &imported_layout.element_styles {
            match kind {
                ImportedElementKind::Action => {
                    apply_imported_element_style(&mut self.styles.action, style)
                }
                ImportedElementKind::SceneHeading => {
                    apply_imported_element_style(&mut self.styles.scene_heading, style)
                }
                ImportedElementKind::Character => {
                    apply_imported_element_style(&mut self.styles.character, style)
                }
                ImportedElementKind::Dialogue => {
                    apply_imported_element_style(&mut self.styles.dialogue, style)
                }
                ImportedElementKind::Parenthetical => {
                    apply_imported_element_style(&mut self.styles.parenthetical, style)
                }
                ImportedElementKind::Transition => {
                    apply_imported_element_style(&mut self.styles.transition, style)
                }
                ImportedElementKind::Lyric => {
                    apply_imported_element_style(&mut self.styles.lyric, style)
                }
                ImportedElementKind::ColdOpening => {
                    apply_imported_element_style(&mut self.styles.cold_opening, style)
                }
                ImportedElementKind::NewAct => {
                    apply_imported_element_style(&mut self.styles.new_act, style)
                }
                ImportedElementKind::EndOfAct => {
                    apply_imported_element_style(&mut self.styles.end_of_act, style)
                }
            }
        }

        overlay_dialogue_continueds(
            &mut self.automatic_character_continueds,
            &mut self.continueds.dialogue,
            &imported_layout.mores_and_continueds.dialogue,
        );
        overlay_scene_continueds(
            &mut self.continueds.scene,
            &imported_layout.mores_and_continueds.scene,
        );
    }
}

fn apply_imported_element_style(
    target: &mut ScreenplayElementStyle,
    imported: &ImportedElementStyle,
) {
    if let Some(left_indent) = imported.left_indent {
        target.left_indent = left_indent;
    }
    if let Some(first_indent) = imported.first_indent {
        target.first_indent = first_indent;
    }
    if let Some(right_indent) = imported.right_indent {
        target.right_indent = right_indent;
    }
    if let Some(spacing_before) = imported.spacing_before {
        target.spacing_before = spacing_before;
    }
    if let Some(line_spacing) = imported.line_spacing {
        target.line_spacing = line_spacing;
    }
    if let Some(alignment) = imported.alignment {
        target.alignment = match alignment {
            ImportedAlignment::Left => Alignment::Left,
            ImportedAlignment::Center => Alignment::Center,
            ImportedAlignment::Right => Alignment::Right,
        };
    }
    if let Some(starts_new_page) = imported.starts_new_page {
        target.starts_new_page = starts_new_page;
    }
    if let Some(underline) = imported.underline {
        target.underline = underline;
    }
    if let Some(bold) = imported.bold {
        target.bold = bold;
    }
    if let Some(italic) = imported.italic {
        target.italic = italic;
    }
}

fn overlay_dialogue_continueds(
    automatic_character_continueds: &mut bool,
    target: &mut DialogueContinueds,
    imported: &ImportedDialogueContinueds,
) {
    if let Some(value) = imported.automatic_character_continueds {
        *automatic_character_continueds = value;
    }
    if let Some(value) = imported.top_of_next {
        target.top_of_next = value;
    }
    if let Some(value) = imported.bottom_of_page {
        target.bottom_of_page = value;
    }
    if let Some(value) = &imported.dialogue_top {
        target.top_text = value.clone();
    }
    if let Some(value) = &imported.dialogue_bottom {
        target.bottom_text = value.clone();
    }
}

fn overlay_scene_continueds(target: &mut SceneContinueds, imported: &ImportedSceneContinueds) {
    if let Some(value) = imported.top_of_next {
        target.top_of_next = value;
    }
    if let Some(value) = imported.bottom_of_page {
        target.bottom_of_page = value;
    }
    if let Some(value) = imported.continued_number {
        target.continued_number = value;
    }
    if let Some(value) = &imported.scene_top {
        target.top_text = value.clone();
    }
    if let Some(value) = &imported.scene_bottom {
        target.bottom_text = value.clone();
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
    } else if option.eq_ignore_ascii_case("a4") {
        profile.page_width = 8.26;
        profile.page_height = 11.69;
        profile.lines_per_page = 58.0;
    } else if option.eq_ignore_ascii_case("balanced") {
        profile.interruption_dash_wrap = InterruptionDashWrap::KeepTogether;
        profile.dual_dialogue_counts_for_contd = false;
    } else if option.eq_ignore_ascii_case("clean-dashes") {
        profile.interruption_dash_wrap = InterruptionDashWrap::KeepTogether;
    } else if option.eq_ignore_ascii_case("no-dual-contds") {
        profile.dual_dialogue_counts_for_contd = false;
    }
}

fn apply_fmt_geometry_override_option(profile: &mut ScreenplayLayoutProfile, option: &str) {
    if matches_fmt_option(option, &["ssbsh", "single-space-before-scene-headings"]) {
        profile.styles.scene_heading.spacing_before = 1.0;
    } else if matches_fmt_option(option, &["bsh", "bold-scene-headings"]) {
        profile.styles.scene_heading.bold = true;
    } else if matches_fmt_option(option, &["ush", "underline-scene-headings"]) {
        profile.styles.scene_heading.underline = true;
    } else if matches_fmt_option(option, &["dsd", "double-spaced-dialogue"]) {
        profile.styles.dialogue.line_spacing = 2.0;
    } else if option.eq_ignore_ascii_case("no-auto-act-breaks") {
        profile.styles.new_act.starts_new_page = false;
    } else if option.eq_ignore_ascii_case("no-act-underlines") {
        profile.styles.cold_opening.underline = false;
        profile.styles.new_act.underline = false;
        profile.styles.end_of_act.underline = false;
    } else if let Some(value) = option.strip_prefix("dl-") {
        if let Ok(indent) = value.parse::<f32>() {
            profile.styles.dialogue.left_indent = indent;
        }
    } else if let Some(value) = option.strip_prefix("dr-") {
        if let Ok(indent) = value.parse::<f32>() {
            profile.styles.dialogue.right_indent = indent;
        }
    } else if let Some(value) = option.strip_prefix("tm-") {
        if let Ok(margin) = value.parse::<f32>() {
            profile.top_margin = margin;
        }
    } else if let Some(value) = option.strip_prefix("bm-") {
        if let Ok(margin) = value.parse::<f32>() {
            profile.bottom_margin = margin;
        }
    } else if let Some(value) = option.strip_prefix("hm-") {
        if let Ok(margin) = value.parse::<f32>() {
            profile.header_margin = margin;
        }
    } else if let Some(value) = option.strip_prefix("fm-") {
        if let Ok(margin) = value.parse::<f32>() {
            profile.footer_margin = margin;
        }
    } else if let Some(value) = option.strip_prefix("lpp-") {
        if let Ok(lpp) = value.parse::<f32>() {
            profile.lines_per_page = lpp;
        }
    }
}

fn matches_fmt_option(option: &str, accepted: &[&str]) -> bool {
    accepted
        .iter()
        .any(|candidate| option.eq_ignore_ascii_case(candidate))
}
