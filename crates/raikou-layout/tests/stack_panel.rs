use raikou_core::{Rect, Size};
use raikou_layout::{
    LayoutElement, Orientation, SizedBox, StackPanel, Visibility, arrange_element, measure_element,
};

#[test]
fn avalonia_stack_panel_vertical_sums_heights_and_spacing() {
    let mut panel = StackPanel::new();
    panel.spacing = 5.0;
    panel.push_child(Box::new(SizedBox::new(Size::new(20.0, 10.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(15.0, 20.0))));

    assert_eq!(
        measure_element(&mut panel, Size::new(100.0, 100.0)),
        Size::new(20.0, 35.0)
    );
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 100.0, 35.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 100.0, 10.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(0.0, 15.0, 100.0, 20.0)
    );
}

#[test]
fn avalonia_stack_panel_horizontal_preserves_order_and_ignores_collapsed_spacing() {
    let mut panel = StackPanel::new();
    panel.orientation = Orientation::Horizontal;
    panel.spacing = 4.0;

    let mut hidden = SizedBox::new(Size::new(99.0, 99.0));
    hidden.layout_mut().visibility = Visibility::Collapsed;

    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 12.0))));
    panel.push_child(Box::new(hidden));
    panel.push_child(Box::new(SizedBox::new(Size::new(15.0, 8.0))));

    assert_eq!(
        measure_element(&mut panel, Size::new(100.0, 100.0)),
        Size::new(29.0, 12.0)
    );
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 29.0, 20.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 10.0, 20.0)
    );
    assert_eq!(
        panel.children()[2].layout().bounds(),
        Rect::from_xywh(14.0, 0.0, 15.0, 20.0)
    );
}
