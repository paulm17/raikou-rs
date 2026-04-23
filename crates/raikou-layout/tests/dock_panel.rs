use raikou_core::{Rect, Size};
use raikou_layout::{Dock, DockPanel, LayoutElement, SizedBox, arrange_element, measure_element};

#[test]
fn avalonia_dock_panel_docks_edges_and_fills_last_child() {
    let mut left = SizedBox::new(Size::new(20.0, 20.0));
    left.layout_mut().attached.dock = Dock::Left;
    let mut top = SizedBox::new(Size::new(40.0, 15.0));
    top.layout_mut().attached.dock = Dock::Top;

    let mut panel = DockPanel::new();
    panel.push_child(Box::new(left));
    panel.push_child(Box::new(top));
    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    assert_eq!(
        measure_element(&mut panel, Size::new(100.0, 100.0)),
        Size::new(60.0, 25.0)
    );
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 100.0, 60.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 20.0, 60.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(20.0, 0.0, 80.0, 15.0)
    );
    assert_eq!(
        panel.children()[2].layout().bounds(),
        Rect::from_xywh(20.0, 15.0, 80.0, 45.0)
    );
}

#[test]
fn avalonia_dock_panel_obeys_dock_order_and_last_child_fill_false() {
    let mut top = SizedBox::new(Size::new(100.0, 10.0));
    top.layout_mut().attached.dock = Dock::Top;
    let mut right = SizedBox::new(Size::new(15.0, 100.0));
    right.layout_mut().attached.dock = Dock::Right;
    let mut bottom = SizedBox::new(Size::new(100.0, 12.0));
    bottom.layout_mut().attached.dock = Dock::Bottom;

    let mut panel = DockPanel::new();
    panel.last_child_fill = false;
    panel.push_child(Box::new(top));
    panel.push_child(Box::new(right));
    panel.push_child(Box::new(bottom));

    measure_element(&mut panel, Size::new(100.0, 60.0));
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 100.0, 60.0));

    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 100.0, 10.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(85.0, 10.0, 15.0, 50.0)
    );
    assert_eq!(
        panel.children()[2].layout().bounds(),
        Rect::from_xywh(0.0, 48.0, 85.0, 12.0)
    );
}
