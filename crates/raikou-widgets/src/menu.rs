//! Menu components: `MenuItem` data + `MenuBar` built on fyrox's native
//! `Menu`/`MenuItem` widgets.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::menu::{MenuItemBuilder, MenuItemContent, MenuItemMessage};
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ItemClickCallback = dyn Fn(&mut UserInterface, usize);

/// A single menu item. Leaf items fire `on_click` (indexed); items with
/// `children` render as a sub-menu.
#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub enabled: bool,
    pub on_click: Option<Rc<ItemClickCallback>>,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    /// Creates a new menu item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            on_click: None,
            children: Vec::new(),
        }
    }

    /// Attaches a click callback that receives this item's index.
    pub fn on_click(mut self, f: impl Fn(&mut UserInterface, usize) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// Marks the item as disabled.
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Attaches sub-menu items (rendered as a nested sub-menu).
    pub fn submenu(mut self, children: Vec<MenuItem>) -> Self {
        self.children = children;
        self
    }
}

/// Handlers for a [`MenuBar`] component. Tracks leaf item handles so a click
/// can be mapped back to the item's index.
#[derive(Clone)]
pub struct MenuBarHandlers {
    item_handles: Vec<Handle<UiNode>>,
    on_item_click: Option<Rc<ItemClickCallback>>,
}

impl MenuBarHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(on_item_click) = &self.on_item_click {
            if message.data::<MenuItemMessage>() == Some(&MenuItemMessage::Click) {
                if let Some(index) = self.item_handles.iter().position(|h| *h == message.destination())
                {
                    on_item_click(ui, index);
                }
            }
        }
    }
}

/// A horizontal menu bar built from top-level menus, each containing items.
#[derive(Clone)]
pub struct MenuBar {
    menus: Vec<(String, Vec<MenuItem>)>,
    on_item_click: Option<Rc<ItemClickCallback>>,
    margin: Thickness,
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBar {
    /// Creates a new menu bar builder.
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            on_item_click: None,
            margin: Thickness::ZERO,
        }
    }

    /// Adds a top-level menu with the given label and items.
    pub fn menu(mut self, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        self.menus.push((label.into(), items));
        self
    }

    /// Attaches a click callback receiving the index of the clicked leaf item.
    pub fn on_item_click(mut self, f: impl Fn(&mut UserInterface, usize) + 'static) -> Self {
        self.on_item_click = Some(Rc::new(f));
        self
    }

    /// Sets the margin around the bar.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the menu bar, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();
        let mut item_handles: Vec<Handle<UiNode>> = Vec::new();

        let mut dropdown_handles = Vec::new();
        for (label, items) in &self.menus {
            let font = ctx.default_font();
            let header: Handle<UiNode> = fyrox::gui::text::TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(to_fyrox_thickness(Thickness::new(8.0, 4.0, 8.0, 4.0))),
            )
            .with_text(label)
            .with_font(font)
            .build(&mut ctx)
            .to_base();

            let item_nodes = build_items(items, &mut item_handles, &mut ctx);
            let item_nodes: Vec<Handle<UiNode>> =
                item_nodes.into_iter().map(|h| h.to_base()).collect();

            let content: Handle<UiNode> = fyrox::gui::stack_panel::StackPanelBuilder::new(
                WidgetBuilder::new().with_children(item_nodes),
            )
            .build(&mut ctx)
            .to_base();

            let dropdown = fyrox::gui::dropdown_menu::DropdownMenuBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_menu")
                    .with_margin(to_fyrox_thickness(self.margin)),
            )
            .with_header(header)
            .with_content(content)
            .build(&mut ctx);
            dropdown_handles.push(dropdown.to_base());
        }

        let handle = if dropdown_handles.is_empty() {
            let panel = fyrox::gui::stack_panel::StackPanelBuilder::new(
                WidgetBuilder::new().with_name("raikou_menu_bar"),
            )
            .build(&mut ctx);
            panel.to_base()
        } else {
            let panel = fyrox::gui::stack_panel::StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_menu_bar")
                    .with_children(dropdown_handles),
            )
            .with_orientation(fyrox::gui::Orientation::Horizontal)
            .build(&mut ctx);
            panel.to_base()
        };

        let kind = ComponentKind::MenuBar(MenuBarHandlers {
            item_handles,
            on_item_click: self.on_item_click,
        });
        let component = Component { handle, kind };
        cx.register(&component);
        component
    }
}

/// Recursively builds a set of menu items into fyrox `MenuItem` widgets,
/// recording leaf handles and their flat index.
pub(crate) fn build_items(
    items: &[MenuItem],
    item_handles: &mut Vec<Handle<UiNode>>,
    ctx: &mut fyrox::gui::BuildContext,
) -> Vec<Handle<fyrox::gui::menu::MenuItem>> {
    let mut handles = Vec::new();
    for item in items {
        let mut builder = MenuItemBuilder::new(
            WidgetBuilder::new().with_name("raikou_menu_item"),
        )
        .with_content(MenuItemContent::text(&item.label));

        if !item.children.is_empty() {
            let sub = build_items(&item.children, item_handles, ctx);
            builder = builder.with_items(sub);
        }

        let handle = builder.build(ctx);
        let base: Handle<UiNode> = handle.to_base();
        item_handles.push(base);
        handles.push(handle);
    }
    handles
}

pub type MenuBarHandle = Handle<UiNode>;
