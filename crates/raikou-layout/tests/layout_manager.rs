use raikou_core::{PaintLayer, Size};
use raikou_layout::{LayoutElement, LayoutManager, OverlayLayer, SizedBox, StackPanel};

#[test]
fn layout_manager_tracks_dirty_measure_and_arrange_queues() {
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));
    let root_id = root.layout().id();

    let mut manager = LayoutManager::new();
    manager.queue_measure(root_id);
    manager.queue_arrange(root_id);

    let summary = manager.update(&mut root, Size::new(100.0, 100.0));
    assert_eq!(summary.queued_measure_count, 1);
    assert_eq!(summary.queued_arrange_count, 1);
    assert!(summary.measured);
    assert!(summary.arranged);
    assert!(!manager.has_dirty_work());
}

#[test]
fn layout_manager_collects_overlay_after_content_using_window_paint_list() {
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let mut overlay = OverlayLayer::new();
    overlay.push_child(Box::new(SizedBox::new(Size::new(5.0, 5.0))));
    let overlay_id = overlay.layout().id();

    root.push_child(Box::new(overlay));

    let manager = LayoutManager::new();
    let list = manager.collect_window_paint_list(&root);
    let ordered = list.ordered();

    assert_eq!(ordered[0].layer, PaintLayer::Content);
    assert_eq!(
        ordered.last().expect("ordered entries").layer,
        PaintLayer::Overlay(raikou_core::OverlayPaintPhase::AfterContent)
    );
    assert!(
        list.overlay_entries()
            .iter()
            .any(|entry| entry.widget_id == overlay_id)
    );
    assert_eq!(list.overlay_entries().len(), 2);
}
