//! Functional tests for the Slider component.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::scroll_bar::ScrollBarMessage;
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


    h.ui.send(slider.handle, ScrollBarMessage::Value(50.0));
    h.pump();
    assert_eq!(
        seen.get(),
        50.0,
        "ScrollBarMessage::Value must forward to on_change"
    );
}

#[test]
fn slider_ignores_foreign_messages() {
    let mut h = Harness::new();
    let changes = Counter::new();
    let c = changes.clone();
    h.build(move |cx| {
        Slider::new()
            .min(0.0)
            .max(1.0)
            .on_change(move |_, _| c.bump())
            .build(cx)
    });

    // A message for a different destination must not reach the handler.
    h.ui.send(
        h.ui.root(),
        fyrox::gui::scroll_viewer::ScrollViewerMessage::VerticalScroll(0.5),
    );
    // (sent to the UI root, not the slider)
    h.pump();
    assert_eq!(changes.get(), 0);
}

/// Finds the thin track line (first child of the scroll bar).
fn find_body(
    ui: &mut fyrox::gui::UserInterface,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
    use fyrox::graph::SceneGraph;
    *ui.node(root).children().first().unwrap()
}

fn read_value(
    ui: &fyrox::gui::UserInterface,
    slider: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> f32 {
    use fyrox::graph::SceneGraph;
    *ui.try_get_of_type::<fyrox::gui::scroll_bar::ScrollBar>(slider)
        .unwrap()
        .value
}

#[test]
fn slider_track_click_jumps_but_thumb_click_does_not() {
    use fyrox::core::algebra::Vector2;
    use fyrox::gui::message::{MessageDirection, MouseButton, UiMessage};
    use fyrox::gui::scroll_bar::ScrollBar;
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(f32::NAN));
    let s = seen.clone();
    let slider = h.build(move |cx| {
        Slider::new()
            .min(0.0)
            .max(100.0)
            .value(0.0)
            .on_change(move |_, v| s.set(v))
            .build(cx)
    });
    h.update_and_pump();

    let body = find_body(&mut h.ui, slider.handle);
    let indicator = {
        use fyrox::graph::SceneGraph;
        *h.ui
            .try_get_of_type::<ScrollBar>(slider.handle)
            .unwrap()
            .indicator
    };

    // Click at 75% along the track (aimed at the slider root like a real
    // press aimed at whatever deep child sits under the cursor).
    let bounds = h.ui.node(body).screen_bounds();
    let pos = Vector2::new(
        bounds.position.x + bounds.size.x * 0.75,
        bounds.position.y + bounds.size.y * 0.5,
    );
    h.ui.send_message(
        UiMessage::with_data(WidgetMessage::MouseDown {
            pos,
            button: MouseButton::Left,
        })
        .with_destination(slider.handle)
        .with_direction(MessageDirection::ToWidget),
    );
    h.pump();
    assert_eq!(
        read_value(&h.ui, slider.handle),
        75.0,
        "track click must jump the slider"
    );
    assert_eq!(seen.get(), 75.0, "jump must report through on_change");

    // A press on the thumb must keep native drag semantics (no jump).
    h.ui.send_message(
        UiMessage::with_data(WidgetMessage::MouseDown {
            pos,
            button: MouseButton::Left,
        })
        .with_destination(indicator)
        .with_direction(MessageDirection::ToWidget),
    );
    h.pump();
    assert_eq!(
        read_value(&h.ui, slider.handle),
        75.0,
        "thumb press must not jump"
    );
}

#[test]
fn slider_arrow_keys_step_and_home_end_jump() {
    use fyrox::gui::message::KeyCode;
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let slider = h.build(move |cx| Slider::new().min(0.0).max(100.0).value(10.0).build(cx));
    h.update_and_pump();

    for (key, expected) in [
        (KeyCode::ArrowRight, 11.0),
        (KeyCode::ArrowUp, 12.0),
        (KeyCode::ArrowLeft, 11.0),
        (KeyCode::ArrowDown, 10.0),
        (KeyCode::End, 100.0),
        (KeyCode::Home, 0.0),
    ] {
        h.ui.send(slider.handle, WidgetMessage::KeyDown(key));
        h.pump();
        assert_eq!(
            read_value(&h.ui, slider.handle),
            expected,
            "{key:?} must move the slider to {expected}"
        );
    }
}

#[test]
fn slider_snaps_offgrid_values_onto_step() {
    let mut h = Harness::new();
    let fired = Counter::new();
    let f = fired.clone();
    let last = std::rc::Rc::new(std::cell::Cell::new(f32::NAN));
    let l = last.clone();
    let slider = h.build(move |cx| {
        Slider::new()
            .min(0.0)
            .max(100.0)
            .step(1.0)
            .value(0.0)
            .on_change(move |_, v| {
                f.bump();
                l.set(v);
            })
            .build(cx)
    });

    h.ui.send(slider.handle, ScrollBarMessage::Value(52.3));
    h.pump();
    assert_eq!(
        read_value(&h.ui, slider.handle),
        52.0,
        "off-grid commits must snap to the step lattice"
    );
    assert_eq!(fired.get(), 1, "snap correction must not double-report");
    assert_eq!(last.get(), 52.0);

    // Ranges whose max is not a step multiple must settle without looping.
    let mut h2 = Harness::new();
    let slider2 = h2.build(move |cx| Slider::new().min(0.0).max(50.0).step(30.0).build(cx));
    h2.update_and_pump();
    h2.ui.send(slider2.handle, ScrollBarMessage::Value(49.0));
    h2.pump();
    assert_eq!(
        read_value(&h2.ui, slider2.handle),
        50.0,
        "clamped snaps must settle even off-lattice"
    );
}
