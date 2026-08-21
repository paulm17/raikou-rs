//! Functional tests for the Popover component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use fyrox::gui::popup::Popup;
use raikou_widgets::{show_popover, hide_popover, Button, Popover};

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
