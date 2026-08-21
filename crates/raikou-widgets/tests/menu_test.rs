//! Functional tests for MenuBar and ContextMenu.

mod common;

use common::{Counter, Harness};
use fyrox::gui::menu::MenuItemMessage;
use raikou_widgets::{ContextMenu, MenuBar, MenuItem};

#[test]
fn menu_bar_reports_leaf_clicks() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let bar = h.build(move |cx| {
        MenuBar::new()
            .menu(
                "File",
                vec![MenuItem::new("New"), MenuItem::new("Open"), MenuItem::new("Exit")],
            )
            .menu("Help", vec![MenuItem::new("About")])
            .on_item_click(move |_, i| s.set(i))
            .build(cx)
    });

    // Leaf items are collected depth-first across menus: File/New=0,
    // File/Open=1, File/Exit=2, Help/About=3. Popups live outside the bar
    // subtree, so scan the whole node pool in creation order.
    let items = collect_menu_items(&h.ui);
    assert_eq!(items.len(), 4, "expected four leaf menu items");

    h.ui.post(items[2], MenuItemMessage::Click);
    h.pump();
    assert_eq!(seen.get(), 2, "clicking 'Exit' must report index 2");

    h.ui.post(items[3], MenuItemMessage::Click);
    h.pump();
    assert_eq!(seen.get(), 3, "clicking 'About' must report index 3");
}

#[test]
fn context_menu_reports_item_clicks() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let menu = h.build(move |cx| {
        ContextMenu::new()
            .item(MenuItem::new("Cut"))
            .item(MenuItem::new("Copy"))
            .item(MenuItem::new("Paste"))
            .on_item_click(move |_, i| s.set(i))
            .build(cx)
    });

    use fyrox::graph::SceneGraph;
    let items = collect_menu_items(&h.ui);
    assert_eq!(items.len(), 3);

    h.ui.post(items[1], MenuItemMessage::Click);
    h.pump();
    assert_eq!(seen.get(), 1, "clicking 'Copy' must report index 1");
}

/// Collects all menu item widget handles in node-creation order (popups are
/// separate graph nodes, so a subtree walk would miss them).
fn collect_menu_items(
    ui: &fyrox::gui::UserInterface,
) -> Vec<fyrox::core::pool::Handle<fyrox::gui::UiNode>> {
    use fyrox::gui::menu::MenuItem;
    ui.nodes()
        .pair_iter()
        .filter(|(_, n)| n.cast::<MenuItem>().is_some())
        .map(|(h, _)| h)
        .collect()
}
