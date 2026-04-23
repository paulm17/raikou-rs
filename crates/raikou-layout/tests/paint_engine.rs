use raikou_core::{PaintLayer, Point, Rect, Size};
use raikou_layout::{
    LayoutContext, LayoutElement, SizedBox, StackPanel, arrange_element, collect_paint_commands, measure_element,
};

#[test]
fn paint_commands_order_content_before_overlay() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    let content = SizedBox::new(Size::new(40.0, 20.0));
    let mut overlay = SizedBox::new(Size::new(30.0, 30.0));
    overlay
        .layout_mut()
        .set_overlay_layer(PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent));

    root.push_child(Box::new(content));
    root.push_child(Box::new(overlay));

    measure_element(&mut root, &mut ctx, Size::new(100.0, 100.0));
    arrange_element(&mut root, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

    let commands = collect_paint_commands(&root);
    let layers: Vec<_> = commands.iter().map(|c| c.layer).collect();

    // Root is content, first child is content, second child is overlay
    assert_eq!(layers[0], PaintLayer::Content);
    assert_eq!(layers[1], PaintLayer::Content);
    assert_eq!(
        layers[2],
        PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent)
    );
}

#[test]
fn paint_commands_accumulate_absolute_positions() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    let mut child = SizedBox::new(Size::new(40.0, 20.0));
    child.layout_mut().margin = raikou_core::Thickness::uniform(10.0);
    root.push_child(Box::new(child));

    measure_element(&mut root, &mut ctx, Size::new(100.0, 100.0));
    arrange_element(&mut root, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

    let commands = collect_paint_commands(&root);
    // Root is at (0,0), child is at (10,10) due to margin
    assert_eq!(commands[0].absolute_position, Point::new(0.0, 0.0));
    assert_eq!(commands[1].absolute_position, Point::new(10.0, 10.0));
}

#[test]
fn multiple_overlays_preserve_insertion_order() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    let mut overlay_a = SizedBox::new(Size::new(40.0, 20.0));
    let mut overlay_b = SizedBox::new(Size::new(40.0, 20.0));
    overlay_a
        .layout_mut()
        .set_overlay_layer(PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent));
    overlay_b
        .layout_mut()
        .set_overlay_layer(PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent));

    let id_a = overlay_a.layout().id();
    let id_b = overlay_b.layout().id();

    root.push_child(Box::new(overlay_a));
    root.push_child(Box::new(overlay_b));

    measure_element(&mut root, &mut ctx, Size::new(100.0, 100.0));
    arrange_element(&mut root, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

    let commands = collect_paint_commands(&root);
    let overlay_commands: Vec<_> = commands
        .iter()
        .filter(|c| matches!(c.layer, PaintLayer::Overlay(_)))
        .collect();
    assert_eq!(overlay_commands.len(), 2);
    // First overlay should appear before second overlay
    assert!(
        commands.iter().position(|c| c.element.layout().id() == id_a)
            < commands.iter().position(|c| c.element.layout().id() == id_b)
    );
}

#[test]
fn set_overlay_layer_on_layoutable_changes_paint_layer() {
    let mut element = SizedBox::new(Size::new(10.0, 10.0));
    assert_eq!(element.layout().paint_layer(), PaintLayer::Content);

    element
        .layout_mut()
        .set_overlay_layer(PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent));
    assert_eq!(
        element.layout().paint_layer(),
        PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent)
    );
}
