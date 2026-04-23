use raikou_core::{Color, Rect, Size};
use raikou_layout::{LayoutElement, Panel, SizedBox, arrange_element, measure_element};

#[test]
fn panel_measures_and_arranges_children() {
    let mut panel = Panel::new();
    panel.push_child(Box::new(SizedBox::new(Size::new(20.0, 10.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(12.0, 30.0))));

    let desired = measure_element(&mut panel, Size::new(100.0, 100.0));
    assert_eq!(desired, Size::new(20.0, 30.0));

    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 80.0, 60.0));
    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 80.0, 60.0)
    );
    assert_eq!(
        panel.children()[1].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 80.0, 60.0)
    );
}

#[test]
fn avalonia_panel_child_add_remove_invalidates_measure() {
    let mut panel = Panel::new();
    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));
    measure_element(&mut panel, Size::new(100.0, 100.0));
    assert!(panel.layout().is_measure_valid());

    panel.push_child(Box::new(SizedBox::new(Size::new(20.0, 5.0))));
    assert!(!panel.layout().is_measure_valid());

    measure_element(&mut panel, Size::new(100.0, 100.0));
    panel.remove_child(0);
    assert!(!panel.layout().is_measure_valid());
}

#[test]
fn avalonia_panel_background_does_not_affect_child_layout() {
    let mut panel = Panel::new();
    panel.set_background(Some(Color::new(0.2, 0.4, 0.8, 1.0)));
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 12.0))));

    let desired = measure_element(&mut panel, Size::new(100.0, 80.0));
    arrange_element(&mut panel, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));

    assert_eq!(desired, Size::new(30.0, 12.0));
    assert_eq!(
        panel.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 100.0, 80.0)
    );
    assert_eq!(panel.background(), Some(Color::new(0.2, 0.4, 0.8, 1.0)));
}
