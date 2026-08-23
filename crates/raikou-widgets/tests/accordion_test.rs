//! Functional tests for the Accordion component.

mod common;

use common::Harness;
use fyrox::gui::expander::ExpanderMessage;
use fyrox::gui::message::MessageDirection;
use raikou_widgets::Accordion;

fn send_expand(
    h: &mut Harness,
    handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
    expanded: bool,
) {
    h.ui.send_message(
        fyrox::gui::message::UiMessage::with_data(ExpanderMessage::Expand(expanded))
            .with_destination(handle)
            .with_direction(MessageDirection::FromWidget),
    );
}

#[test]
fn accordion_toggle_callback_receives_index_and_state() {
    use fyrox::graph::SceneGraph;
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
    // accordion root (skipping the hairline dividers).
        let items: Vec<_> = h
        .ui
        .node(acc.handle)
        .children()
        .iter()
        .copied()
        .filter(|c| h.ui.try_get_of_type::<fyrox::gui::expander::Expander>(*c).is_ok())
        .collect();
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
    let items: Vec<_> = h
        .ui
        .node(acc.handle)
        .children()
        .iter()
        .copied()
        .filter(|c| h.ui.try_get_of_type::<fyrox::gui::expander::Expander>(*c).is_ok())
        .collect();

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

/// Walks an expander's fixed layout: outer grid > inner grid > [checkbox,
/// header row].
fn header_row_of(
    ui: &mut fyrox::gui::UserInterface,
    expander: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
    use fyrox::graph::SceneGraph;
    let outer = *ui.node(expander).children().first().unwrap();
    let inner = *ui.node(outer).children().first().unwrap();
    *ui.node(inner).children().get(1).unwrap()
}

fn checkbox_base_of(
    ui: &fyrox::gui::UserInterface,
    expander: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
    use fyrox::graph::SceneGraph;
    (*ui
        .try_get_of_type::<fyrox::gui::expander::Expander>(expander)
        .unwrap()
        .expander)
        .to_base()
}

fn is_expanded(
    ui: &fyrox::gui::UserInterface,
    expander: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> bool {
    use fyrox::graph::SceneGraph;
    *ui.try_get_of_type::<fyrox::gui::expander::Expander>(expander)
        .unwrap()
        .is_expanded
}

#[test]
fn accordion_header_click_toggles_expansion() {
    use fyrox::gui::message::{MessageDirection, MouseButton, UiMessage};
    use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
    use fyrox::graph::SceneGraph;

    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(usize, bool)>::new()));
    let s = seen.clone();
    let acc = h.build(move |cx| {
        let mut ctx = cx.ctx();
        let content = fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
            .with_text("details")
            .build(&mut ctx)
            .to_base();
        Accordion::new()
            .on_toggle(move |_, i, e| s.borrow_mut().push((i, e)))
            .push_item(raikou_widgets::AccordionItem {
                label: "Alpha".into(),
                expanded: false,
                content: Some(content),
                accent: None,
            })
            .build(cx)
    });
    h.update_and_pump();

    let items: Vec<_> = h
        .ui
        .node(acc.handle)
        .children()
        .iter()
        .copied()
        .filter(|c| h.ui.try_get_of_type::<fyrox::gui::expander::Expander>(*c).is_ok())
        .collect();
    assert_eq!(items.len(), 1);
    let header = header_row_of(&mut h.ui, items[0]);

    // A press/release on the header label flips the item.
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
                .with_destination(header)
                .with_direction(MessageDirection::ToWidget),
        );
    }
    h.pump();

    assert!(
        is_expanded(&h.ui, items[0]),
        "header click must expand the item"
    );
    assert_eq!(
        seen.borrow().as_slice(),
        &[(0, true)],
        "header click must report exactly one toggle"
    );

    // Clicking again collapses it.
    h.ui.send_message(
        UiMessage::with_data(WidgetMessage::MouseUp {
            pos: Default::default(),
            button: MouseButton::Left,
        })
        .with_destination(header)
        .with_direction(MessageDirection::ToWidget),
    );
    h.pump();
    assert!(!is_expanded(&h.ui, items[0]), "second click collapses");
}

#[test]
fn accordion_checkbox_click_does_not_double_toggle() {
    use fyrox::gui::message::{MessageDirection, MouseButton, UiMessage};
    use fyrox::gui::widget::WidgetMessage;
    use fyrox::graph::SceneGraph;

    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(usize, bool)>::new()));
    let s = seen.clone();
    let acc = h.build(move |cx| {
        Accordion::new()
            .on_toggle(move |_, i, e| s.borrow_mut().push((i, e)))
            .item("Alpha")
            .build(cx)
    });
    h.update_and_pump();

    let items: Vec<_> = h
        .ui
        .node(acc.handle)
        .children()
        .iter()
        .copied()
        .filter(|c| h.ui.try_get_of_type::<fyrox::gui::expander::Expander>(*c).is_ok())
        .collect();
    let checkbox = checkbox_base_of(&h.ui, items[0]);

    // The chevron checkbox toggles natively; the header-hit watcher must
    // stay out of the way so exactly one flip happens.
    h.ui.send_message(
        UiMessage::with_data(WidgetMessage::MouseUp {
            pos: Default::default(),
            button: MouseButton::Left,
        })
        .with_destination(checkbox)
        .with_direction(MessageDirection::ToWidget),
    );
    h.pump();

    assert!(is_expanded(&h.ui, items[0]), "native chevron click expands");
    assert_eq!(
        seen.borrow().as_slice(),
        &[(0, true)],
        "chevron click must report exactly one toggle"
    );
}
