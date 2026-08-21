//! Functional tests for Select and Combobox.

mod common;

use common::{Counter, Harness};
use fyrox::gui::dropdown_list::DropdownListMessage;
use fyrox::gui::message::{MessageDirection, UiMessage};
use raikou_widgets::{Combobox, Select};

fn send_selection(h: &mut Harness, handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>, index: usize) {
    h.ui.send_message(
        UiMessage::with_data(DropdownListMessage::Selection(Some(index)))
            .with_destination(handle)
            .with_direction(MessageDirection::FromWidget),
    );
}

#[test]
fn select_reports_selection() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let sel = h.build(move |cx| {
        Select::new()
            .items(vec!["Red", "Green", "Blue"])
            .on_change(move |_, i| s.set(i))
            .build(cx)
    });

    send_selection(&mut h, sel.handle, 2);
    h.pump();
    assert_eq!(seen.get(), 2, "Selection(Some(2)) must fire on_change(2)");
}

#[test]
fn combobox_reports_selection() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let cb = h.build(move |cx| {
        Combobox::new()
            .items(vec!["Small", "Medium", "Large"])
            .placeholder("Pick size")
            .on_change(move |_, i| s.set(i))
            .build(cx)
    });

    send_selection(&mut h, cb.handle, 0);
    h.pump();
    assert_eq!(seen.get(), 0);

    // Deselection (None) must not fire.
    h.ui.send_message(
        UiMessage::with_data(DropdownListMessage::Selection(None))
            .with_destination(cb.handle)
            .with_direction(MessageDirection::FromWidget),
    );
    h.pump();
    assert_eq!(seen.get(), 0, "Selection(None) must be ignored");
}
