use raikou_core::{Size, Thickness};
use raikou_layout::{LayoutElement, SizedBox, measure_element};

#[test]
fn measure_core_applies_width_height_min_max_and_margin() {
    let mut node = SizedBox::new(Size::new(50.0, 20.0));
    let layout = node.layout_mut();
    layout.width = Some(80.0);
    layout.constraints.min.height = 30.0;
    layout.constraints.max.width = 100.0;
    layout.margin = Thickness::new(5.0, 3.0, 5.0, 7.0);

    let desired = measure_element(&mut node, Size::new(200.0, 200.0));
    assert_eq!(desired, Size::new(90.0, 40.0));
}

#[test]
fn measure_core_collapsed_node_returns_zero() {
    let mut node = SizedBox::new(Size::new(50.0, 20.0));
    node.layout_mut().visibility = raikou_layout::Visibility::Collapsed;

    let desired = measure_element(&mut node, Size::new(200.0, 200.0));
    assert_eq!(desired, Size::ZERO);
}

#[test]
fn measure_core_sanitizes_invalid_available_sizes() {
    let mut node = SizedBox::new(Size::new(50.0, 20.0));
    let desired = measure_element(&mut node, Size::new(f32::NAN, -5.0));
    assert_eq!(desired, Size::ZERO);
}
