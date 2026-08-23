//! Functional tests for Select and Combobox.

mod common;

use common::Harness;
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

/// Walks `root` looking for the first node of type `T`.
fn find_of_type<T: fyrox::core::reflect::Reflect>(
    ui: &fyrox::gui::UserInterface,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> Option<fyrox::core::pool::Handle<fyrox::gui::UiNode>> {
    use fyrox::graph::SceneGraph;

    let mut stack = vec![root];
    while let Some(h) = stack.pop() {
        if h.is_none() {
            continue;
        }
        if ui.try_get_of_type::<T>(h).is_ok() {
            return Some(h);
        }
        for c in ui.node(h).children() {
            stack.push(*c);
        }
    }
    None
}

fn dropdown_of(
    h: &Harness,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> fyrox::core::pool::Handle<fyrox::gui::UiNode> {
    find_of_type::<fyrox::gui::dropdown_list::DropdownList>(&h.ui, root)
        .expect("inner dropdown list")
}

#[test]
fn select_alt_down_opens_popup() {
    let mut h = Harness::new();
    let sel = h.build(|cx| Select::new().items(vec!["Red", "Green", "Blue"]).build(cx));
    let dd = dropdown_of(&h, sel.handle);

    // Alt+Down must open the flyout. Fyrox's ArrowDown arm is
    // modifier-agnostic, so the Alt variant rides along for free.
    h.ui.process_os_event(&fyrox::gui::message::OsEvent::KeyboardModifiers(
        fyrox::gui::message::KeyboardModifiers {
            alt: true,
            shift: false,
            control: false,
            system: false,
        },
    ));
    h.ui.send(
        dd,
        fyrox::gui::widget::WidgetMessage::KeyDown(fyrox::gui::message::KeyCode::ArrowDown),
    );
    h.pump();

    let popup: fyrox::core::pool::Handle<fyrox::gui::UiNode> = {
        use fyrox::graph::SceneGraph;
        (*h.ui.try_get_of_type::<fyrox::gui::dropdown_list::DropdownList>(dd)
            .unwrap()
            .popup)
            .to_base()
    };
    let open = *h
        .ui
        .try_get_of_type::<fyrox::gui::popup::Popup>(popup)
        .unwrap()
        .is_open;
    assert!(open, "Alt+Down must open the dropdown flyout");
}

#[test]
fn select_arrows_cycle_open_list() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let s = seen.clone();
    let sel = h.build(move |cx| {
        Select::new()
            .items(vec!["Red", "Green", "Blue"])
            .on_change(move |_, i| s.borrow_mut().push(i))
            .build(cx)
    });
    use fyrox::gui::dropdown_list::DropdownList;

    let dd = dropdown_of(&h, sel.handle);

    // Open the flyout like a click would.
    h.ui.send(dd, DropdownListMessage::Open);
    h.pump();

    use fyrox::graph::SceneGraph;
    let lv: fyrox::core::pool::Handle<fyrox::gui::UiNode> = {
        let dd_ref = h.ui.try_get_of_type::<DropdownList>(dd).unwrap();
        (*dd_ref.list_view).to_base()
    };

    let send_arrow = |h: &mut Harness, key: fyrox::gui::message::KeyCode| {
        h.ui.send(
            lv,
            fyrox::gui::widget::WidgetMessage::KeyDown(key),
        );
        h.pump();
    };
    let selection = |h: &Harness| {
        *h.ui
            .try_get_of_type::<DropdownList>(dd)
            .unwrap()
            .selection
    };
    let popup_open = |h: &Harness| {
        let popup: fyrox::core::pool::Handle<fyrox::gui::UiNode> =
            (*h.ui.try_get_of_type::<DropdownList>(dd).unwrap().popup).to_base();
        *h.ui.try_get_of_type::<fyrox::gui::popup::Popup>(popup)
            .unwrap()
            .is_open
    };

    assert_eq!(selection(&h), None);
    send_arrow(&mut h, fyrox::gui::message::KeyCode::ArrowDown);
    assert_eq!(selection(&h), Some(0), "ArrowDown highlights the first item");
    send_arrow(&mut h, fyrox::gui::message::KeyCode::ArrowDown);
    assert_eq!(selection(&h), Some(1), "ArrowDown advances the highlight");
    send_arrow(&mut h, fyrox::gui::message::KeyCode::ArrowUp);
    assert_eq!(selection(&h), Some(0), "ArrowUp steps back");
    assert!(
        popup_open(&h),
        "cycling must not dismiss the flyout (close_on_selection stays off)"
    );
    assert_eq!(
        *seen.borrow(),
        vec![0, 1, 0],
        "each committed cycle must fire on_change with the highlighted index"
    );
}
