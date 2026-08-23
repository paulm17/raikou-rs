//! Functional tests for the Checkbox component.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::check_box::CheckBoxMessage;
use fyrox::gui::message::MouseButton;
use fyrox::gui::widget::WidgetMessage;
use raikou_widgets::Checkbox;

#[test]
fn checkbox_check_message_invokes_callback() {
    let mut h = Harness::new();
    let changes = Counter::new();
    let c = changes.clone();
    let cb = h.build(move |cx| {
        Checkbox::new()
            .text("Accept")
            .on_change(move |_, v| {
                let _ = v;
                c.bump();
            })
            .build(cx)
    });

    // The dispatcher forwards Check(Some(true)) as on_change(true).
    h.ui.send(cb.handle, CheckBoxMessage::Check(Some(true)));
    h.pump();
    assert_eq!(changes.get(), 1, "Check(Some(true)) must fire on_change");

    h.ui.send(cb.handle, CheckBoxMessage::Check(Some(false)));
    h.pump();
    assert_eq!(changes.get(), 2, "Check(Some(false)) must fire on_change");
}

#[test]
fn checkbox_none_state_maps_to_false() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = seen.clone();
    let cb = h.build(move |cx| Checkbox::new().on_change(move |_, v| s.set(v)).build(cx));

    h.ui.send(cb.handle, CheckBoxMessage::Check(None));
    h.pump();
    assert!(
        !seen.get(),
        "Check(None) (indeterminate) must map to on_change(false)"
    );
}

#[test]
fn checkbox_native_click_toggles() {
    let mut h = Harness::new();
    let values = std::rc::Rc::new(std::cell::RefCell::new(Vec::<bool>::new()));
    let v = values.clone();
    let cb = h.build(move |cx| {
        Checkbox::new()
            .text("Click")
            .checked(false)
            .on_change(move |_, val| v.borrow_mut().push(val))
            .build(cx)
    });

    // The native check box toggles on a left MouseUp over it.
    h.ui.send(
        cb.handle,
        WidgetMessage::MouseUp {
            pos: Default::default(),
            button: MouseButton::Left,
        },
    );
    h.update_and_pump();

    let vals = values.borrow();
    assert_eq!(vals.len(), 1, "click must produce exactly one change");
    assert_eq!(vals[0], true, "click on unchecked box must check it");
}

#[test]
fn checkbox_three_state_cycles_like_avalonia() {
    use fyrox::gui::check_box::CheckBox;

    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<Option<bool>>::new()));
    let s = seen.clone();
    let cb = h.build(move |cx| {
        Checkbox::new()
            .text("Tri")
            .three_state(true)
            .state(Some(false))
            .on_change_state(move |_, state| s.borrow_mut().push(state))
            .build(cx)
    });

    assert_eq!(
        *h.ui.try_get_of_type::<CheckBox>(cb.handle).unwrap().checked,
        Some(false),
        "initial indeterminate=false state"
    );

    // Avalonia cycle from false: null -> true -> false. Each click is a
    // MouseUp over the box (native toggles on release).
    for _ in 0..3 {
        h.ui.send(
            cb.handle,
            WidgetMessage::MouseUp {
                pos: Default::default(),
                button: MouseButton::Left,
            },
        );
        h.update_and_pump();
    }

    assert_eq!(
        *seen.borrow(),
        vec![None, Some(true), Some(false)],
        "cycle must match Avalonia CheckBox.Toggle"
    );
    assert_eq!(
        *h.ui.try_get_of_type::<CheckBox>(cb.handle).unwrap().checked,
        Some(false),
        "full cycle returns to false"
    );
}

#[test]
fn checkbox_three_state_programmatic_initial() {
    use fyrox::gui::check_box::CheckBox;

    let mut h = Harness::new();
    let cb = h.build(|cx| Checkbox::new().three_state(true).state(None).build(cx));

    h.update_and_pump();
    assert_eq!(
        *h.ui.try_get_of_type::<CheckBox>(cb.handle).unwrap().checked,
        None,
        "state(Some(None)) must build an indeterminate checkbox"
    );
}
