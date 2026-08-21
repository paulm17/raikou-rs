//! Functional tests for the Slider component.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::scroll_bar::ScrollBarMessage;
use fyrox::gui::Orientation;
use raikou_widgets::Slider;

#[test]
fn slider_value_message_invokes_callback() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(f32::NAN));
    let s = seen.clone();
    let slider = h.build(move |cx| {
        Slider::new()
            .min(0.0)
            .max(100.0)
            .value(10.0)
            .on_change(move |_, v| s.set(v))
            .build(cx)
    });

    h.ui.send(
        slider.handle,
        ScrollBarMessage::Value(50.0),
    );
    h.pump();
    assert_eq!(
        seen.get(), 50.0,
        "ScrollBarMessage::Value must forward to on_change"
    );
}

#[test]
fn slider_ignores_foreign_messages() {
    let mut h = Harness::new();
    let changes = Counter::new();
    let c = changes.clone();
    let slider = h.build(move |cx| {
        Slider::new().min(0.0).max(1.0).on_change(move |_, _| c.bump()).build(cx)
    });

    // A message for a different destination must not reach the handler.
    h.ui.send(h.ui.root(), fyrox::gui::scroll_viewer::ScrollViewerMessage::VerticalScroll(0.5));
    // (sent to the UI root, not the slider)
    h.pump();
    assert_eq!(changes.get(), 0);
}
