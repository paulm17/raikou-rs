//! Functional tests for Select and Combobox.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::dropdown_list::DropdownListMessage;
use fyrox::gui::message::{MessageDirection, UiMessage};
use raikou_widgets::{Combobox, Select};

fn send_selection(
    h: &mut Harness,
    handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
    index: usize,
) {
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

/// Finds the first Text node under `root.
fn find_text(
    ui: &fyrox::gui::UserInterface,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> Option<fyrox::core::pool::Handle<fyrox::gui::UiNode>> {
    let mut stack = vec![root];
    while let Some(h) = stack.pop() {
        if h.is_none() {
            continue;
        }
        if ui.try_get_of_type::<fyrox::gui::text::Text>(h).is_ok() {
            return Some(h);
        }
        for c in ui.node(h).children() {
            stack.push(*c);
        }
    }
    None
}

#[test]
fn select_and_combobox_flip_placeholder_visibility() {
    let mut h = Harness::new();
    let sel = h.build(|cx| {
        Select::new()
            .items(vec!["Red", "Green"])
            .placeholder("Pick one")
            .build(cx)
    });
    let cb = h.build(|cx| {
        Combobox::new()
            .items(vec!["Small", "Medium"])
            .placeholder("Pick size")
            .build(cx)
    });

    for root in [sel.handle, cb.handle] {
        let text = find_text(&h.ui, root).expect("placeholder text must exist");
        assert!(
            h.ui.try_get(text).unwrap().visibility(),
            "placeholder must start visible"
        );

        send_selection(&mut h, root, 1);
        h.pump();
        assert!(
            !h.ui.try_get(text).unwrap().visibility(),
            "placeholder must hide once an item is selected"
        );

        h.ui.send_message(
            UiMessage::with_data(DropdownListMessage::Selection(None))
                .with_destination(root)
                .with_direction(MessageDirection::FromWidget),
        );
        h.pump();
        assert!(
            h.ui.try_get(text).unwrap().visibility(),
            "placeholder must return when selection clears"
        );
    }
}
