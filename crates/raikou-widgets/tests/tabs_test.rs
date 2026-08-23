//! Functional tests for the Tabs component.

mod common;

use common::Harness;
use fyrox::gui::button::ButtonMessage;
use fyrox::gui::tab_control::TabControlMessage;
use raikou_widgets::Tabs;

#[test]
fn tabs_active_tab_message_reports_index() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let tabs = h.build(move |cx| {
        let mut t = Tabs::new().on_change(move |_, i| s.set(i));
        for label in ["One", "Two", "Three"] {
            t = t.tab(label, Default::default());
        }
        t.build(cx)
    });

    // The dispatcher maps the tab uuid to its position. We don't know the
    // uuids from outside, so verify via the native header buttons instead
    // (covered in tabs_header_click_activates_tab); here we assert that an
    // ActiveTab message with an unknown uuid is ignored.
    h.ui.send(
        tabs.handle,
        TabControlMessage::ActiveTab(Some(fyrox::core::uuid::uuid!(
            "00000000-0000-0000-0000-000000000000"
        ))),
    );
    h.pump();
    assert_eq!(seen.get(), usize::MAX, "unknown uuid must be ignored");
}

#[test]
fn tabs_header_click_activates_tab() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let tabs = h.build(move |cx| {
        let mut t = Tabs::new().initial_tab(0).on_change(move |_, i| s.set(i));
        for label in ["One", "Two", "Three"] {
            t = t.tab(label, Default::default());
        }
        t.build(cx)
    });

    // Clicking a header button makes the native TabControl emit
    // ActiveTab(Some(uuid)); raikou maps it to the tab index.
    let buttons = collect_buttons(&mut h.ui, tabs.handle);
    assert!(buttons.len() >= 3, "expected 3 header buttons");
    h.ui.send(buttons[2], ButtonMessage::Click);
    h.update_and_pump();

    assert_eq!(seen.get(), 2, "clicking header 2 must select tab index 2");

    h.ui.send(buttons[1], ButtonMessage::Click);
    h.update_and_pump();
    assert_eq!(seen.get(), 1);
}

/// Collects all Button nodes below `root` breadth-first, i.e. in
/// declaration/visual order.
fn collect_buttons(
    ui: &mut fyrox::gui::UserInterface,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> Vec<fyrox::core::pool::Handle<fyrox::gui::UiNode>> {
    use fyrox::graph::SceneGraph;
    let mut out = Vec::new();
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(h) = queue.pop_front() {
        if h.is_none() {
            continue;
        }
        if ui.try_get_of_type::<fyrox::gui::button::Button>(h).is_ok() {
            out.push(h);
        }
        for child in ui.node(h).children() {
            queue.push_back(*child);
        }
    }
    out
}

#[test]
fn tabs_arrow_keys_switch_tabs() {
    use fyrox::gui::message::KeyCode;
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let tabs = h.build(move |cx| {
        let mut t = Tabs::new().on_change(move |_, i| s.set(i));
        for label in ["One", "Two", "Three"] {
            t = t.tab(label, Default::default());
        }
        t.build(cx)
    });

    // Left/Right switch the active tab with wraparound; fyrox's TabControl
    // has no keyboard handling, so this is driven by the raikou watcher.
    h.ui.send(tabs.handle, WidgetMessage::KeyDown(KeyCode::ArrowRight));
    h.pump();
    assert_eq!(seen.get(), 1, "ArrowRight activates the next tab");

    h.ui.send(tabs.handle, WidgetMessage::KeyDown(KeyCode::ArrowLeft));
    h.pump();
    assert_eq!(seen.get(), 0, "ArrowLeft returns to the first tab");

    h.ui.send(tabs.handle, WidgetMessage::KeyDown(KeyCode::ArrowLeft));
    h.pump();
    assert_eq!(seen.get(), 2, "ArrowLeft wraps to the last tab");
}
