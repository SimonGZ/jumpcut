pub enum ElementType {
    Action,
    Character,
    Dialogue,
    Parenthetical,
    Transition,
    Lyric,
}

pub struct WrapConfig {
    pub exact_width_chars: usize,
}

impl WrapConfig {
    pub fn new(element_type: ElementType) -> Self {
        let width = match element_type {
            ElementType::Action => 61,
            ElementType::Character => 42, 
            ElementType::Dialogue => 35,
            ElementType::Parenthetical => 25,
            ElementType::Transition => 16,
            ElementType::Lyric => 48,
        };
        Self { exact_width_chars: width }
    }

    pub fn with_exact_width_chars(width: usize) -> Self {
        Self { exact_width_chars: width }
    }
}

pub fn wrap_text_for_element(text: &str, config: &WrapConfig) -> Vec<String> {
    let mut lines = Vec::new();
    let max_width = config.exact_width_chars;

    if text.is_empty() {
        return lines;
    }

    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        // `split_inclusive` keeps the delimiter attached to the previous chunk,
        // allowing us to preserve internal spaces perfectly.
        let words: Vec<&str> = paragraph.split_inclusive(' ').collect();

        for word in words {
            let word_len = word.chars().count();
            let line_len = current_line.chars().count();
            
            // If the word ends with a space, that space doesn't force a wrap 
            // if it falls exactly on the boundary.
            let fits = if word.ends_with(' ') {
                line_len + word_len - 1 <= max_width
            } else {
                line_len + word_len <= max_width
            };

            if current_line.is_empty() {
                // Always push the first word of a line, even if it's too long
                current_line.push_str(word);
            } else if fits {
                current_line.push_str(word);
            } else {
                // Line is full. Trim trailing spaces on the rendered visual line
                lines.push(current_line.trim_end().to_string());
                current_line = String::from(word);
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}
