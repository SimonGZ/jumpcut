use crate::pagination::wrapping::ElementType;

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
        }
    }
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
