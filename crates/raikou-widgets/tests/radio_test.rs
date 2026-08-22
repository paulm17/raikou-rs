//! Functional tests for Radio and RadioGroup.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::check_box::CheckBoxMessage;
use raikou_widgets::{Radio, RadioGroup};

#[test]
fn radio_check_message_invokes_callback() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = seen.clone();
    let r = h.build(move |cx| {
        Radio::new()
            .text("Option A")
            .on_change(move |_, v| s.set(v))
            .build(cx)
    });

    h.ui.send(r.handle, CheckBoxMessage::Check(Some(true)));
    h.pump();
    assert!(seen.get(), "Check(Some(true)) must fire on_change(true)");
}

#[test]
fn radio_group_selects_and_deselects_siblings() {
    let mut h = Harness::new();
    let picked = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let p = picked.clone();
    let group = h.build(move |cx| {
        RadioGroup::new()
            .options(["Red", "Green", "Blue"])
            .selected(0)
            .on_change(move |_, i| p.set(i))
            .build(cx)
    });

    // Options are native check boxes linked as direct children of the group
    // root, in declaration order.
    let options: Vec<_> = h.ui.node(group.handle).children().to_vec();
    assert_eq!(options.len(), 3, "group must build one option per label");

    // Selecting item 2 must report index 2 ...
    h.ui.send(options[2], CheckBoxMessage::Check(Some(true)));
    h.pump();
    assert_eq!(picked.get(), 2, "group callback must receive the new index");

    // ... and the previously selected sibling (item 0) must have been sent
    // Check(Some(false)). Deselecting must NOT re-fire the group callback.
    let before = picked.get();
    h.ui.send(options[0], CheckBoxMessage::Check(Some(false)));
    h.pump();
    assert_eq!(
        picked.get(),
        before,
        "deselect messages must not invoke the group callback"
    );

    // Selecting another option moves the selection.
    h.ui.send(options[1], CheckBoxMessage::Check(Some(true)));
    h.pump();
    assert_eq!(picked.get(), 1);
}
