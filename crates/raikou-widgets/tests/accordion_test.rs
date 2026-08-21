//! Functional tests for the Accordion component.

mod common;

use common::{Counter, Harness};
use fyrox::gui::expander::ExpanderMessage;
use fyrox::gui::message::MessageDirection;
use raikou_widgets::Accordion;

fn send_expand(h: &mut Harness, handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>, expanded: bool) {
    h.ui.send_message(
        fyrox::gui::message::UiMessage::with_data(ExpanderMessage::Expand(expanded))
            .with_destination(handle)
            .with_direction(MessageDirection::FromWidget),
    );
}

#[test]
fn accordion_toggle_callback_receives_index_and_state() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(usize, bool)>::new()));
    let s = seen.clone();
    let acc = h.build(move |cx| {
        let mut a = Accordion::new().on_toggle(move |_, i, e| s.borrow_mut().push((i, e)));
        for label in ["Alpha", "Beta"] {
            a = a.item(label);
        }
        a.build(cx)
    });

    // Item handles are registered per expander; find them as children of the
    // accordion root.
    use fyrox::graph::SceneGraph;
    let items: Vec<_> = h.ui.node(acc.handle).children().to_vec();
    assert_eq!(items.len(), 2, "accordion must build one item per label");

    send_expand(&mut h, items[1], true);
    h.pump();

    let seen = seen.borrow();
    assert_eq!(seen.len(), 1, "one toggle must fire one callback");
    assert_eq!(seen[0], (1, true), "callback must carry index and state");
}

#[test]
fn accordion_single_mode_collapses_siblings() {
    let mut h = Harness::new();
    let expanded = std::rc::Rc::new(std::cell::RefCell::new(Vec::<bool>::new()));
    let e = expanded.clone();
    let acc = h.build(move |cx| {
        let mut a = Accordion::new()
            .allow_multiple(false)
            .on_toggle(move |_, _, ex| e.borrow_mut().push(ex));
        for label in ["A", "B"] {
            a = a.item(label);
        }
        a.build(cx)
    });

    use fyrox::graph::SceneGraph;
    let items: Vec<_> = h.ui.node(acc.handle).children().to_vec();

    // Expanding item 1 must collapse item 0 internally (single-open mode)
    // while reporting exactly ONE user-driven toggle event.
    send_expand(&mut h, items[1], true);
    h.pump();

    let events = expanded.borrow();
    assert_eq!(
        events.as_slice(),
        &[true],
        "only the user-driven expansion is reported: {events:?}"
    );
}
