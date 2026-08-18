//! ContextMenu component: a right-click menu built on fyrox's native
//! `ContextMenu` (a `Popup` hosting `MenuItem`s).

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::menu::{
    ContextMenuBuilder, MenuItemBuilder, MenuItemContent, MenuItemMessage,
};
use fyrox::gui::message::UiMessage;
use fyrox::gui::popup::PopupBuilder;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;
use crate::menu::MenuItem;

type ItemClickCallback = dyn Fn(&mut UserInterface, usize);

/// Handlers for a [`ContextMenu`] component.
#[derive(Clone)]
pub struct ContextMenuHandlers {
    item_handles: Vec<Handle<UiNode>>,
    on_item_click: Option<Rc<ItemClickCallback>>,
}

impl ContextMenuHandlers {
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

/// A floating context menu opened via [`show_context_menu`].
#[derive(Clone)]
pub struct ContextMenu {
    items: Vec<MenuItem>,
    on_item_click: Option<Rc<ItemClickCallback>>,
    margin: Thickness,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenu {
    /// Creates a new context menu builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            on_item_click: None,
            margin: Thickness::ZERO,
        }
    }

    /// Adds an item to the menu.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Attaches a click callback receiving the index of the clicked item.
    pub fn on_item_click(mut self, f: impl Fn(&mut UserInterface, usize) + 'static) -> Self {
        self.on_item_click = Some(Rc::new(f));
        self
    }

    /// Sets the margin around the menu panel.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the context menu, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();
        let mut item_handles: Vec<Handle<UiNode>> = Vec::new();

        let mut items = Vec::new();
        for item in &self.items {
            let mut builder = MenuItemBuilder::new(
                WidgetBuilder::new().with_name("raikou_context_menu_item"),
            )
            .with_content(MenuItemContent::text(&item.label));

            if !item.children.is_empty() {
                let mut sub_handles = Vec::new();
                let sub = crate::menu::build_items(&item.children, &mut sub_handles, &mut ctx);
                builder = builder.with_items(sub);
                item_handles.extend(sub_handles);
            }

            let handle = builder.build(&mut ctx);
            item_handles.push(handle.to_base());
            items.push(handle);
        }

        let popup = PopupBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_context_menu")
                .with_margin(to_fyrox_thickness(self.margin)),
        );
        let context_menu = ContextMenuBuilder::new(popup).build(&mut ctx);
        let handle: Handle<UiNode> = context_menu.to_base();

        // Attach items to the popup's content via the parent menu item wiring.
        if !items.is_empty() {
            ctx.link(items[0], handle);
        }

        let kind = ComponentKind::ContextMenu(ContextMenuHandlers {
            item_handles,
            on_item_click: self.on_item_click,
        });
        let component = Component { handle, kind };
        cx.register(&component);
        component
    }
}

/// Opens the given context menu.
pub fn show_context_menu(ui: &mut UserInterface, handle: Handle<UiNode>) {
    use fyrox::gui::popup::PopupMessage;
    ui.send(handle, PopupMessage::Open);
}

/// Closes the given context menu.
pub fn hide_context_menu(ui: &mut UserInterface, handle: Handle<UiNode>) {
    use fyrox::gui::popup::PopupMessage;
    ui.send(handle, PopupMessage::Close);
}

pub type ContextMenuHandle = Handle<UiNode>;
