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
    let button = h.build(move |cx| {
        Button::new()
            .text("Save")
            .on_click(move |_, _| c.bump())
            .build(cx)
    });

    // Release mode fires on the synthetic Click message the native button
    // emits after a real press/release pair.
    h.ui.send(button.handle, ButtonMessage::Click);
    h.pump();
    assert_eq!(
        clicks.get(),
        1,
        "on_click should fire on ButtonMessage::Click"
    );
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

#[test]
fn button_default_fires_on_enter_when_unfocused() {
    let mut h = Harness::new();
    let defaults = Counter::new();
    let others = Counter::new();
    let (d, o) = (defaults.clone(), others.clone());
    let default_btn = h.build(move |cx| {
        Button::new()
            .text("OK")
            .is_default(true)
            .on_click(move |_, _| d.bump())
            .build(cx)
    });
    let _other = h.build(move |cx| {
        Button::new()
            .text("Nope")
            .on_click(move |_, _| o.bump())
            .build(cx)
    });

    // Enter aimed at the root canvas (nothing focused): the default button
    // must activate.
    let root = h.ui.root();
    h.ui.send(root, WidgetMessage::KeyDown(KeyCode::Enter));
    h.pump();
    assert_eq!(defaults.get(), 1, "Enter must activate the default button");
    assert_eq!(others.get(), 0, "sibling buttons must stay idle");
    let _ = default_btn;

    // NumpadEnter behaves identically.
    h.ui.send(root, WidgetMessage::KeyDown(KeyCode::NumpadEnter));
    h.pump();
    assert_eq!(defaults.get(), 2, "NumpadEnter must activate default too");

    // Space never triggers the default action (Avalonia parity).
    h.ui.send(root, WidgetMessage::KeyDown(KeyCode::Space));
    h.pump();
    assert_eq!(defaults.get(), 2, "Space must not trigger the default");
}

#[test]
fn button_enter_on_focused_button_does_not_double_fire_default() {
    let mut h = Harness::new();
    let defaults = Counter::new();
    let others = Counter::new();
    let (d, o) = (defaults.clone(), others.clone());
    let _default_btn = h.build(move |cx| {
        Button::new()
            .text("OK")
            .is_default(true)
            .on_click(move |_, _| d.bump())
            .build(cx)
    });
    let other = h.build(move |cx| {
        Button::new()
            .text("Cancel")
            .on_click(move |_, _| o.bump())
            .build(cx)
    });

    // Enter aimed AT another button (real focus lands on the inner fyrox
    // Button, not raikou's chrome wrapper): the native focused-button
    // activation fires that button; the default must stay silent.
    fn find_inner_button(ui: &fyrox::gui::UserInterface, root: fyrox::core::pool::Handle<fyrox::gui::UiNode>) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
        if ui.try_get_of_type::<fyrox::gui::button::Button>(root).is_ok() {
            return root;
        }
        for child in ui.node(root).children() {
            let found = find_inner_button(ui, *child);
            if found.is_some() {
                return found;
            }
        }
        fyrox::core::pool::Handle::NONE
    }

    h.update_and_pump();
    let inner_other = find_inner_button(&h.ui, other.handle);
    assert!(inner_other.is_some(), "chrome must wrap an inner Button");
    let before = others.get();
    h.ui.send(inner_other, WidgetMessage::KeyDown(KeyCode::Enter));
    h.pump();
    assert_eq!(
        others.get(),
        before + 1,
        "focused sibling must fire exactly once"
    );
    assert_eq!(defaults.get(), 0, "default must not double-fire");
}

#[test]
fn button_default_ignores_enter_in_text_fields() {
    use fyrox::gui::text_box::TextBox;
    use raikou_widgets::{TextArea, TextInput};

    let mut h = Harness::new();
    let defaults = Counter::new();
    let d = defaults.clone();
    let _default_btn = h.build(move |cx| {
        Button::new()
            .text("OK")
            .is_default(true)
            .on_click(move |_, _| d.bump())
            .build(cx)
    });

    // Multiline text area: Enter types a newline, must not submit dialogs.
    let area = h.build(|cx| TextArea::new().rows(3).build(cx));
    let inner_area = h.ui.node(area.handle).children()[0];
    assert!(
        h.ui
            .try_get_of_type::<TextBox>(inner_area)
            .map(|tb| *tb.multiline)
            .unwrap_or(false),
        "area inner must be a multiline TextBox"
    );
    h.ui.send(inner_area, WidgetMessage::KeyDown(KeyCode::Enter));
    h.pump();
    assert_eq!(defaults.get(), 0, "multiline Enter must not fire default");

    // Single-line input: Enter SHOULD activate the default (commit semantics).
    let input = h.build(|cx| TextInput::new().build(cx));
    let inner_input = h.ui.node(input.handle).children()[0];
    h.ui.send(inner_input, WidgetMessage::KeyDown(KeyCode::Enter));
    h.pump();
    assert_eq!(defaults.get(), 1, "single-line Enter should commit");
}
