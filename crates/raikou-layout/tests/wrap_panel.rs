use raikou_core::{Rect, Size};
use raikou_layout::{Orientation, SizedBox, WrapPanel, arrange_element, measure_element};

#[test]
fn avalonia_wrap_panel_wraps_items_and_tracks_line_height() {
    let mut panel = WrapPanel::new();
    panel.item_spacing = 5.0;
    panel.line_spacing = 2.0;
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 10.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 20.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 15.0))));

    assert_eq!(
        measure_element(&mut panel, Size::new(70.0, 100.0)),
        Size::new(65.0, 37.0)
    );
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 70.0, 37.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 30.0, 20.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(35.0, 0.0, 30.0, 20.0)
    );
    assert_eq!(
        panel.children()[2].layout().bounds(),
        Rect::from_xywh(0.0, 22.0, 30.0, 15.0)
    );
}

#[test]
fn avalonia_wrap_panel_vertical_orientation_wraps_by_height() {
    let mut panel = WrapPanel::new();
    panel.orientation = Orientation::Vertical;
    panel.item_spacing = 3.0;
    panel.line_spacing = 4.0;
    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 20.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(15.0, 20.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(12.0, 20.0))));

    assert_eq!(
        measure_element(&mut panel, Size::new(100.0, 45.0)),
        Size::new(31.0, 43.0)
    );
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 31.0, 45.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 15.0, 20.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(0.0, 23.0, 15.0, 20.0)
    );
    assert_eq!(
        panel.children()[2].layout().bounds(),
        Rect::from_xywh(19.0, 0.0, 12.0, 20.0)
    );
}
