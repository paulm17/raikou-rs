use raikou_core::{PaintLayer, Rect, Size};
use raikou_layout::{
    LayoutElement, OverlayLayer, Panel, SizedBox, arrange_element, measure_element,
};

#[test]
fn avalonia_overlay_layer_measures_to_root_size_and_paints_in_overlay_layer() {
    let mut overlay = OverlayLayer::new();
    overlay.push_child(Box::new(SizedBox::new(Size::new(30.0, 20.0))));

    assert_eq!(
        measure_element(&mut overlay, Size::new(200.0, 120.0)),
        Size::new(200.0, 120.0)
    );
    arrange_element(&mut overlay, Rect::from_xywh(0.0, 0.0, 200.0, 120.0));

    assert_eq!(overlay.available_size(), Size::new(200.0, 120.0));
    assert_eq!(
        overlay.paint_layer(),
        PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent)
    );
}

#[test]
fn avalonia_overlay_layer_remains_stable_during_resize() {
    let mut overlay = OverlayLayer::new();
    overlay.push_child(Box::new(SizedBox::new(Size::new(20.0, 10.0))));

    measure_element(&mut overlay, Size::new(100.0, 80.0));
    arrange_element(&mut overlay, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));
    assert_eq!(overlay.available_size(), Size::new(100.0, 80.0));

    measure_element(&mut overlay, Size::new(160.0, 90.0));
    arrange_element(&mut overlay, Rect::from_xywh(0.0, 0.0, 160.0, 90.0));
    assert_eq!(overlay.available_size(), Size::new(160.0, 90.0));
}

#[test]
fn avalonia_overlay_children_do_not_affect_normal_content_desired_size() {
    let mut content = Panel::new();
    content.push_child(Box::new(SizedBox::new(Size::new(40.0, 20.0))));

    let mut overlay = OverlayLayer::new();
    overlay.push_child(Box::new(SizedBox::new(Size::new(200.0, 100.0))));

    assert_eq!(
        measure_element(&mut content, Size::new(100.0, 80.0)),
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        measure_element(&mut overlay, Size::new(100.0, 80.0)),
        Size::new(100.0, 80.0)
    );
}
