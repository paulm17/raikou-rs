use raikou_core::{Rect, Size, Thickness};
use raikou_layout::{
    HorizontalAlignment, LayoutElement, SizedBox, VerticalAlignment, arrange_element,
    measure_element,
};

#[test]
fn arrange_core_aligns_with_margin_and_stretch_rules() {
    let mut node = SizedBox::new(Size::new(30.0, 20.0));
    let layout = node.layout_mut();
    layout.margin = Thickness::uniform(5.0);
    layout.horizontal_alignment = HorizontalAlignment::Center;
    layout.vertical_alignment = VerticalAlignment::Bottom;

    measure_element(&mut node, Size::new(100.0, 100.0));
    arrange_element(&mut node, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

    assert_eq!(
        node.layout().bounds(),
        Rect::from_xywh(35.0, 75.0, 30.0, 20.0)
    );
}

#[test]
fn arrange_core_stretch_fills_slot_when_no_explicit_size_exists() {
    let mut node = SizedBox::new(Size::new(10.0, 10.0));
    measure_element(&mut node, Size::new(100.0, 80.0));
    arrange_element(&mut node, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));

    assert_eq!(
        node.layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 100.0, 80.0)
    );
}

#[test]
fn arrange_core_respects_left_right_top_and_center_alignments() {
    let mut left_top = SizedBox::new(Size::new(20.0, 10.0));
    left_top.layout_mut().horizontal_alignment = HorizontalAlignment::Left;
    left_top.layout_mut().vertical_alignment = VerticalAlignment::Top;
    measure_element(&mut left_top, Size::new(100.0, 80.0));
    arrange_element(&mut left_top, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));
    assert_eq!(
        left_top.layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 20.0, 10.0)
    );

    let mut right_center = SizedBox::new(Size::new(20.0, 10.0));
    right_center.layout_mut().horizontal_alignment = HorizontalAlignment::Right;
    right_center.layout_mut().vertical_alignment = VerticalAlignment::Center;
    measure_element(&mut right_center, Size::new(100.0, 80.0));
    arrange_element(&mut right_center, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));
    assert_eq!(
        right_center.layout().bounds(),
        Rect::from_xywh(80.0, 35.0, 20.0, 10.0)
    );
}
