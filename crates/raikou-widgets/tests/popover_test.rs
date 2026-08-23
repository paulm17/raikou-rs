//! Functional tests for the Popover component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use fyrox::gui::popup::Popup;
use raikou_widgets::{hide_popover, show_popover, Button, Popover};

#[test]
fn popover_open_close_roundtrip() {
    let mut h = Harness::new();
    let owner = h.build(|cx| Button::new().text("Owner").build(cx));
    let popover = h.build(|cx| {
        let content = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("Popover body")
                .build(&mut ctx)
                .to_base()
        };
        Popover::new()
            .content(content)
            .owner(owner.handle)
            .build(cx)
    });

    assert!(
        !*h.ui.node(popover.handle).cast::<Popup>().unwrap().is_open,
        "popover must start closed"
    );

    show_popover(&h.ui, popover.handle);
    h.update_and_pump();
    assert!(
        *h.ui.node(popover.handle).cast::<Popup>().unwrap().is_open,
        "show_popover must open the popup"
    );

    hide_popover(&h.ui, popover.handle);
    h.update_and_pump();
    assert!(
        !*h.ui.node(popover.handle).cast::<Popup>().unwrap().is_open,
        "hide_popover must close the popup"
    );
}

#[test]
fn popover_light_dismiss_by_default() {
    let mut h = Harness::new();
    let owner = h.build(|cx| Button::new().text("Owner").build(cx));
    let popover = h.build(|cx| {
        let content = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("Popover body")
                .build(&mut ctx)
                .to_base()
        };
        Popover::new()
            .content(content)
            .owner(owner.handle)
            .build(cx)
    });

    // Fluent light-dismiss: an outside mouse press closes the popup.
    assert!(
        !*h.ui
            .node(popover.handle)
            .cast::<Popup>()
            .unwrap()
            .stays_open,
        "popover must default to light-dismiss (stays_open = false)"
    );

    show_popover(&h.ui, popover.handle);
    h.update_and_pump();

    // Simulate the outside press: fyrox posts PopupMessage::Close to the
    // popup when a MouseDown lands outside its bounds while open. Drive the
    // same native path by sending Close directly and verifying it applies.
    h.ui.send_message(
        fyrox::gui::message::UiMessage::with_data(fyrox::gui::popup::PopupMessage::Close)
            .with_destination(popover.handle),
    );
    h.update_and_pump();
    assert!(
        !*h.ui.node(popover.handle).cast::<Popup>().unwrap().is_open,
        "outside interaction must close a light-dismiss popup"
    );
}

#[test]
fn popover_stays_open_opt_in() {
    let mut h = Harness::new();
    let owner = h.build(|cx| Button::new().text("Owner").build(cx));
    let popover = h.build(|cx| {
        let content = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("Sticky body")
                .build(&mut ctx)
                .to_base()
        };
        Popover::new()
            .content(content)
            .owner(owner.handle)
            .stays_open(true)
            .build(cx)
    });

    assert!(
        *h.ui.node(popover.handle).cast::<Popup>().unwrap().stays_open,
        ".stays_open(true) must opt out of light-dismiss"
    );
}
