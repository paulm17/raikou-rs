//! Functional tests for the Switch component.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::message::{MessageDirection, MouseButton, UiMessage};
use fyrox::gui::toggle::ToggleButtonMessage;
use fyrox::gui::widget::WidgetMessage;
use raikou_widgets::Switch;

#[test]
fn switch_toggled_message_invokes_callback() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = seen.clone();
    let sw = h.build(move |cx| {
        Switch::new()
            .text("Wi-Fi")
            .on_change(move |_, v| s.set(v))
            .build(cx)
    });

    h.ui.send(sw.handle, ToggleButtonMessage::Toggled(true));
    h.pump();
    assert!(seen.get(), "Toggled(true) must fire on_change(true)");

    h.ui.send(sw.handle, ToggleButtonMessage::Toggled(false));
    h.pump();
    assert!(!seen.get(), "Toggled(false) must fire on_change(false)");
}

#[test]
fn switch_native_click_toggles() {
    let mut h = Harness::new();
    let values = std::rc::Rc::new(std::cell::RefCell::new(Vec::<bool>::new()));
    let v = values.clone();
    let sw = h.build(move |cx| {
        Switch::new()
            .toggled(false)
            .on_change(move |_, val| v.borrow_mut().push(val))
            .build(cx)
    });

    // The native toggle button only reacts to FromWidget mouse input: press,
    // release (while captured), then let the UI route the messages.
    let track = find_track(&mut h.ui, sw.handle);
    assert!(track.is_some(), "track toggle button must exist");
    for msg in [
        WidgetMessage::MouseDown {
            pos: Default::default(),
            button: MouseButton::Left,
        },
        WidgetMessage::MouseUp {
            pos: Default::default(),
            button: MouseButton::Left,
        },
    ] {
        h.ui.send_message(
            UiMessage::with_data(msg)
                .with_destination(track)
                .with_direction(MessageDirection::FromWidget),
        );
    }
    h.update_and_pump();

    let vals = values.borrow();
    assert_eq!(vals.len(), 1, "click must produce exactly one change");
    assert_eq!(vals[0], true, "click on off switch must turn it on");
}

/// Finds the first ToggleButton below `root` (breadth-first).
fn find_track(
    ui: &mut fyrox::gui::UserInterface,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(h) = queue.pop_front() {
        if h.is_none() {
            continue;
        }
        if ui
            .try_get_of_type::<fyrox::gui::toggle::ToggleButton>(h)
            .is_ok()
        {
            return h;
        }
        for child in ui.node(h).children() {
            queue.push_back(*child);
        }
    }
    Default::default()
}
