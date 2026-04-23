use raikou_core::{Rect, Size};
use raikou_layout::{LayoutContext, LayoutElement, Panel, SizedBox, arrange_element, measure_element};

#[test]
fn panel_measure_override_works_with_children() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = Panel::new();
    panel.push_child(Box::new(SizedBox::new(Size::new(20.0, 10.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(12.0, 30.0))));

    let desired = measure_element(&mut panel, &mut ctx, Size::new(100.0, 100.0));
    assert_eq!(desired, Size::new(20.0, 30.0));
}

#[test]
#[should_panic(
    expected = "Panel::arrange_override is unimplemented"
)]
fn panel_arrange_override_panics_with_children() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = Panel::new();
    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    measure_element(&mut panel, &mut ctx, Size::new(100.0, 100.0));
    arrange_element(&mut panel, &mut ctx, Rect::from_xywh(0.0, 0.0, 80.0, 60.0));
}

#[test]
fn panel_arrange_override_does_not_panic_without_children() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = Panel::new();
    measure_element(&mut panel, &mut ctx, Size::new(100.0, 100.0));
    arrange_element(&mut panel, &mut ctx, Rect::from_xywh(0.0, 0.0, 80.0, 60.0));
    assert_eq!(panel.layout().bounds(), Rect::from_xywh(0.0, 0.0, 80.0, 60.0));
}

#[test]
fn avalonia_panel_child_add_remove_invalidates_measure() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = Panel::new();
    panel.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));
    measure_element(&mut panel, &mut ctx, Size::new(100.0, 100.0));
    assert!(panel.layout().is_measure_valid());

    panel.push_child(Box::new(SizedBox::new(Size::new(20.0, 5.0))));
    assert!(!panel.layout().is_measure_valid());

    measure_element(&mut panel, &mut ctx, Size::new(100.0, 100.0));
    panel.remove_child(0);
    assert!(!panel.layout().is_measure_valid());
}

#[test]
fn avalonia_panel_background_does_not_affect_child_layout() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = Panel::new();
    panel.set_background(Some(raikou_core::Color::new(0.2, 0.4, 0.8, 1.0)));
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 12.0))));

    let desired = measure_element(&mut panel, &mut ctx, Size::new(100.0, 80.0));
    assert_eq!(desired, Size::new(30.0, 12.0));
    assert_eq!(panel.background(), Some(raikou_core::Color::new(0.2, 0.4, 0.8, 1.0)));
}
