use std::collections::BTreeMap;
use serde::Deserialize;
use crate::pagination::wrapping::ElementType;

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

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutGeometry {
    pub action_left: f32,
    pub action_right: f32,
    pub character_left: f32,
    pub character_right: f32,
    pub dialogue_left: f32,
    pub dialogue_right: f32,
    pub parenthetical_left: f32,
    pub parenthetical_right: f32,
    pub transition_left: f32,
    pub transition_right: f32,
    pub lyric_left: f32,
    pub lyric_right: f32,
    pub cpi: f32,

    // Vertical Spacing (Blank lines before element)
    pub action_spacing_before: f32,
    pub scene_heading_spacing_before: f32,
    pub character_spacing_before: f32,
    pub transition_spacing_before: f32,
    pub lyric_spacing_before: f32,

    // Orphan/Widow Limits
    pub orphan_limit: usize,
    pub widow_limit: usize,

    // Spacing
    pub line_height: f32,
}

impl Default for LayoutGeometry {
    fn default() -> Self {
        Self {
            action_left: 1.5,
            action_right: 7.5,
            character_left: 3.5,
            character_right: 7.25,
            dialogue_left: 2.5,
            dialogue_right: 6.0,
            parenthetical_left: 3.0,
            parenthetical_right: 5.5,
            transition_left: 5.5,
            transition_right: 7.1,
            lyric_left: 2.5,
            lyric_right: 7.375,
            cpi: 10.0,

            action_spacing_before: 1.0,
            scene_heading_spacing_before: 2.0,
            character_spacing_before: 1.0,
            transition_spacing_before: 1.0,
            lyric_spacing_before: 1.0,

            orphan_limit: 2,
            widow_limit: 2,
            line_height: 1.0,
        }
    }
}

impl LayoutGeometry {
    pub fn from_fdx_settings(settings: &FdxExtractedSettings) -> Self {
        let mut geometry = Self::default();

        let lpi = 6.0; // Default Final Draft lines per inch

        if let Some(style) = settings.paragraph_styles.get("Action") {
            geometry.action_left = style.left_indent;
            geometry.action_right = style.right_indent;
            geometry.action_spacing_before = spacing_lines_from_points(style.space_before, lpi);
        }
        if let Some(style) = settings.paragraph_styles.get("Scene Heading") {
            geometry.scene_heading_spacing_before = spacing_lines_from_points(style.space_before, lpi);
        }
        if let Some(style) = settings.paragraph_styles.get("Dialogue") {
            geometry.dialogue_left = style.left_indent;
            geometry.dialogue_right = style.right_indent;
        }
        if let Some(style) = settings.paragraph_styles.get("Character") {
            geometry.character_left = style.left_indent;
            geometry.character_right = style.right_indent;
            geometry.character_spacing_before = spacing_lines_from_points(style.space_before, lpi);
        }
        if let Some(style) = settings.paragraph_styles.get("Parenthetical") {
            geometry.parenthetical_left = style.left_indent;
            geometry.parenthetical_right = style.right_indent;
        }
        if let Some(style) = settings.paragraph_styles.get("Lyric") {
            geometry.lyric_left = style.left_indent;
            geometry.lyric_right = style.right_indent;
            geometry.lyric_spacing_before = spacing_lines_from_points(style.space_before, lpi);
        }
        if let Some(style) = settings.paragraph_styles.get("Transition") {
            geometry.transition_left = style.left_indent;
            geometry.transition_right = style.right_indent;
            geometry.transition_spacing_before = spacing_lines_from_points(style.space_before, lpi);
        }

        geometry
    }
}

fn spacing_lines_from_points(space_before_points: f32, lines_per_inch: f32) -> f32 {
    let points_per_line = 72.0 / lines_per_inch;
    space_before_points / points_per_line
}

/// Calculates the exact character capacity for an element given its physical 
/// margin bounds (in inches) and the characters-per-inch (CPI) of the typeface.
pub fn calculate_element_width(geometry: &LayoutGeometry, element_type: ElementType) -> usize {
    let (left_indent, right_indent) = match element_type {
        ElementType::Action => (geometry.action_left, geometry.action_right),
        ElementType::SceneHeading => (geometry.action_left, geometry.action_right), // Standard default
        ElementType::Character => (geometry.character_left, geometry.character_right),
        ElementType::Dialogue => (geometry.dialogue_left, geometry.dialogue_right),
        ElementType::Parenthetical => (geometry.parenthetical_left, geometry.parenthetical_right),
        ElementType::Transition => (geometry.transition_left, geometry.transition_right),
        ElementType::Lyric => (geometry.lyric_left, geometry.lyric_right),
    };

    let width_inches = right_indent - left_indent;
    let mut chars = (width_inches * geometry.cpi).floor() as usize;

    // Apply the Final Draft specific quirk where the Action and Parenthetical grids explicitly 
    // hold an N+1 amount of characters compared to pure mathematical bounds.
    if matches!(element_type, ElementType::Action | ElementType::Parenthetical) {
        chars += 1;
    }
    
    chars
}
