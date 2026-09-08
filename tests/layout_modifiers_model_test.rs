use jumpcut::{Attributes, ElementLayoutOverrides, blank_attributes};

#[test]
fn blank_attributes_and_default_attributes_start_with_empty_layout_overrides() {
    let expected = ElementLayoutOverrides {
        left_indent_delta: None,
        space_before_delta: None,
        right_indent_delta: None,
    };

    assert_eq!(Attributes::default().layout_overrides, expected);
    assert_eq!(blank_attributes().layout_overrides, expected);
}
