//! Functional tests for the Button component: click modes, keyboard
//! activation, loading suppression and hover callbacks.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::button::ButtonMessage;
use fyrox::gui::message::{KeyCode, MouseButton};
use fyrox::gui::widget::WidgetMessage;
use raikou_widgets::{Button, ClickMode};

#[test]
fn button_click_release_mode() {
    let mut h = Harness::new();
    let clicks = Counter::new();
    let c = clicks.clone();
    let button = h.build(move |cx| Button::new().text("Save").on_click(move |_, _| c.bump()).build(cx));

    // Release mode fires on the synthetic Click message the native button
    // emits after a real press/release pair.
    h.ui.send(button.handle, ButtonMessage::Click);
    h.pump();
    assert_eq!(clicks.get(), 1, "on_click should fire on ButtonMessage::Click");
}

#[test]
fn button_click_press_mode() {
    let mut h = Harness::new();
    let clicks = Counter::new();
    let c = clicks.clone();
    let button = h.build(move |cx| {
        Button::new()
            .text("Save")
            .click_mode(ClickMode::Press)
            .on_click(move |_, _| c.bump())
            .build(cx)
    });

    h.ui.send(
        button.handle,
        WidgetMessage::MouseDown {
                        pos: Default::default(),
            button: MouseButton::Left,
        },
    );
    h.pump();
    assert_eq!(clicks.get(), 1, "press mode should fire on left MouseDown");

    // Right-click must not fire.
    h.ui.send(
        button.handle,
        WidgetMessage::MouseDown {
                        pos: Default::default(),
            button: MouseButton::Right,
        },
    );
    h.pump();
    assert_eq!(clicks.get(), 1, "right MouseDown must not fire on_click");
}

#[test]
fn button_keyboard_activation() {
    // In Press and Hover modes raikou's dispatcher reacts to raw key-up
    // events. In Release mode keyboard activation is handled by the native
    // fyrox button (which emits ButtonMessage::Click, covered by
    // button_click_release_mode), so no direct KeyUp response is expected.
    for mode in [ClickMode::Press, ClickMode::Hover] {
        let mut h = Harness::new();
        let clicks = Counter::new();
        let c = clicks.clone();
        let button = h.build(move |cx| {
            Button::new()
                .text("OK")
                .click_mode(mode)
                .on_click(move |_, _| c.bump())
                .build(cx)
        });

        h.ui.send(button.handle, WidgetMessage::KeyUp(KeyCode::Enter));
        h.pump();
        assert_eq!(
            clicks.get(),
            1,
            "Enter must activate the button in mode {mode:?}"
        );

        h.ui.send(button.handle, WidgetMessage::KeyUp(KeyCode::Space));
        h.pump();
        assert_eq!(
            clicks.get(),
            2,
            "Space must activate the button in mode {mode:?}"
        );
    }
}

#[test]
fn button_hover_mode_and_callbacks() {
    let mut h = Harness::new();
    let clicks = Counter::new();
    let overs = Counter::new();
    let outs = Counter::new();
    let (c, o, u) = (clicks.clone(), overs.clone(), outs.clone());
    let button = h.build(move |cx| {
        Button::new()
            .text("H")
            .click_mode(ClickMode::Hover)
            .on_click(move |_, _| c.bump())
            .on_mouse_over(move |_| o.bump())
            .on_mouse_out(move |_| u.bump())
            .build(cx)
    });

    h.ui.send(button.handle, WidgetMessage::MouseEnter);
    h.pump();
    assert_eq!(overs.get(), 1, "MouseEnter fires on_mouse_over");
    assert_eq!(clicks.get(), 1, "hover mode fires on_click on enter");

    h.ui.send(button.handle, WidgetMessage::MouseLeave);
    h.pump();
    assert_eq!(outs.get(), 1, "MouseLeave fires on_mouse_out");
}

#[test]
fn button_loading_suppresses_interaction() {
    let mut h = Harness::new();
    let clicks = Counter::new();
    let c = clicks.clone();
    let button = h.build(move |cx| {
        Button::new()
            .text("Busy")
            .is_loading(true)
            .on_click(move |_, _| c.bump())
            .build(cx)
    });

    h.ui.send(button.handle, ButtonMessage::Click);
    h.ui.send(
        button.handle,
        WidgetMessage::MouseDown {
                        pos: Default::default(),
            button: MouseButton::Left,
        },
    );
    h.ui.send(button.handle, WidgetMessage::MouseEnter);
    h.pump();
    assert_eq!(clicks.get(), 0, "loading state must suppress all handlers");
}

#[test]
fn button_disabled_state_applies() {
    let mut h = Harness::new();
    let button = h.build(|cx| Button::new().text("D").build(cx));

    button.set_enabled(&mut h.ui, false);
    h.pump();
    assert!(
        !h.ui.node(button.handle).enabled(),
        "widget must report disabled after set_enabled(false)"
    );

    button.set_enabled(&mut h.ui, true);
    h.pump();
    assert!(
        h.ui.node(button.handle).enabled(),
        "widget must report enabled after set_enabled(true)"
    );
}
