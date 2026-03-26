use crate::pagination::wrapping::ElementType;

/// Calculates the exact character capacity for an element given its physical 
/// margin bounds (in inches) and the characters-per-inch (CPI) of the typeface.
pub fn calculate_element_width(left_indent: f32, right_indent: f32, cpi: f32, element_type: ElementType) -> usize {
    let width_inches = right_indent - left_indent;
    let mut chars = (width_inches * cpi).floor() as usize;

    // Apply the Final Draft specific quirk where the Action and Parenthetical grids explicitly 
    // hold an N+1 amount of characters compared to pure mathematical bounds.
    if matches!(element_type, ElementType::Action | ElementType::Parenthetical) {
        chars += 1;
    }
    
    chars
}
