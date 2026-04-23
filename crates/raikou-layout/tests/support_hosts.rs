use raikou_core::{Rect, Size};
use raikou_layout::{
    ItemsHost, LayoutContext, LayoutElement, LayoutManager, ScrollContentPresenter, SizedBox, StackPanel,
    VirtualizationHost, arrange_element, measure_element,
};

#[test]
fn items_host_exposes_realized_range_contract() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut host = ItemsHost::new();
    host.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));
    host.push_child(Box::new(SizedBox::new(Size::new(20.0, 20.0))));

    assert_eq!(host.realized_range(), 0..2);
    assert_eq!(
        measure_element(&mut host, &mut ctx, Size::new(50.0, 50.0)),
        Size::new(20.0, 20.0)
    );
}

#[test]
fn scroll_content_presenter_tracks_viewport_and_extent() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut presenter = ScrollContentPresenter::new();
    presenter.set_child(Box::new(SizedBox::new(Size::new(200.0, 150.0))));

    assert_eq!(
        measure_element(&mut presenter, &mut ctx, Size::new(100.0, 80.0)),
        Size::new(100.0, 80.0)
    );
    arrange_element(&mut presenter, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));

    assert_eq!(presenter.viewport(), Size::new(100.0, 80.0));
    assert_eq!(presenter.extent(), Size::new(200.0, 150.0));
}

#[test]
fn scroll_content_presenter_viewport_not_set_during_measure() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut presenter = ScrollContentPresenter::new();
    presenter.set_child(Box::new(SizedBox::new(Size::new(200.0, 150.0))));

    // Viewport should not be updated during measure — it remains zero until arrange.
    assert_eq!(presenter.viewport(), Size::ZERO);

    measure_element(&mut presenter, &mut ctx, Size::new(100.0, 80.0));

    // After measure, viewport is still zero (not set to available).
    assert_eq!(presenter.viewport(), Size::ZERO);

    arrange_element(&mut presenter, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));

    // Only after arrange does viewport reflect the final arranged size.
    assert_eq!(presenter.viewport(), Size::new(100.0, 80.0));
}

#[test]
fn scroll_content_presenter_no_infinities_inside_stack_panel() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    let mut presenter = ScrollContentPresenter::new();
    presenter.set_child(Box::new(SizedBox::new(Size::new(200.0, 150.0))));
    root.push_child(Box::new(presenter));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(100.0, f32::INFINITY));

    let presenter_ref = root.children()[0]
        .as_any()
        .downcast_ref::<ScrollContentPresenter>()
        .expect("expected ScrollContentPresenter");

    let (sx, sy) = presenter_ref.scroll_offset();
    assert!(!sx.is_nan() && !sx.is_infinite(), "scroll_offset_x must be finite");
    assert!(!sy.is_nan() && !sy.is_infinite(), "scroll_offset_y must be finite");
}

#[test]
fn scroll_content_presenter_clamps_scroll_when_viewport_larger_than_extent() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut presenter = ScrollContentPresenter::new();
    presenter.set_child(Box::new(SizedBox::new(Size::new(50.0, 50.0))));

    arrange_element(&mut presenter, &mut ctx, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

    // Viewport (100x100) is larger than extent (50x50); offset should clamp to (0, 0).
    let scp = &mut presenter as &mut dyn LayoutElement;
    scp.as_any_mut()
        .downcast_mut::<ScrollContentPresenter>()
        .unwrap()
        .set_scroll_offset(9999.0, 9999.0);

    let (sx, sy) = presenter.scroll_offset();
    assert_eq!(sx, 0.0);
    assert_eq!(sy, 0.0);
}
