use raikou_core::{Rect, Size};
use raikou_layout::{LayoutElement, SizedBox, arrange_element, measure_element};

#[test]
fn layout_rounding_stabilizes_fractional_bounds() {
    let mut node = SizedBox::new(Size::new(10.4, 10.4));
    measure_element(&mut node, Size::new(50.5, 50.5));
    arrange_element(&mut node, Rect::from_xywh(0.4, 0.6, 10.4, 10.4));
    let first = node.layout().bounds();

    arrange_element(&mut node, Rect::from_xywh(0.4, 0.6, 10.4, 10.4));
    let second = node.layout().bounds();

    assert_eq!(first, second);
    assert_eq!(first, Rect::from_xywh(0.0, 1.0, 11.0, 10.0));
}

#[test]
fn layout_rounding_rounds_margins_in_final_arrangement() {
    let mut node = SizedBox::new(Size::new(8.2, 8.2));
    node.layout_mut().margin = raikou_core::Thickness::new(0.6, 0.6, 0.6, 0.6);

    measure_element(&mut node, Size::new(20.0, 20.0));
    arrange_element(&mut node, Rect::from_xywh(0.0, 0.0, 20.0, 20.0));

    assert_eq!(
        node.layout().bounds(),
        Rect::from_xywh(1.0, 1.0, 18.0, 18.0)
    );
}
