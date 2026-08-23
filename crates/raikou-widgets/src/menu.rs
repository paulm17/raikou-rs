//! Menu components: `MenuItem` data + `MenuBar` built on fyrox's native
//! `Menu`/`MenuItem` widgets.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::graph::SceneGraph;
use fyrox::gui::border::Border;
use fyrox::gui::brush::Brush;
use fyrox::gui::decorator::{Decorator, DecoratorMessage};
use fyrox::gui::dropdown_menu::DropdownMenu;
use fyrox::gui::menu::{MenuItemBuilder, MenuItemContent, MenuItemMessage};
use fyrox::gui::message::UiMessage;
use fyrox::gui::popup::Popup;
use fyrox::gui::text::Text;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

type ItemClickCallback = dyn Fn(&mut UserInterface, usize);

/// Fluent menu flyout row height (~32px in the reference shell).
const FLUENT_MENU_ROW_HEIGHT: f32 = 32.0;

/// Applies Fluent flyout styling to every popup body and menu-item
/// decorator reachable from `roots` (including nested sub-menu popups,
/// which live outside the normal widget tree). Items listed in `disabled`
/// are additionally disabled and dimmed (Fluent disabled label, no hover
/// chrome).
pub(crate) fn style_menu_chrome(
    ui: &mut UserInterface,
    theme: &raikou_style::Theme,
    roots: &[Handle<UiNode>],
    disabled: &[Handle<UiNode>],
) {
    let fallback_white = raikou_core::Color::new(1.0, 1.0, 1.0, 1.0);
    let elevated = Brush::Solid(to_fyrox_color(
        theme.color("surface.elevated").unwrap_or(fallback_white),
    ));
    let stroke = Brush::Solid(to_fyrox_color(
        theme
            .color("border.subtle")
            .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.14)),
    ));
    let hover = Brush::Solid(to_fyrox_color(
        theme
            .color("fluent.list.low")
            .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.05)),
    ));
    let pressed = Brush::Solid(to_fyrox_color(
        theme
            .color("fluent.list.medium")
            .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.10)),
    ));
    let transparent = Brush::Solid(fyrox::core::color::Color::TRANSPARENT);

    let mut stack: Vec<Handle<UiNode>> = roots.to_vec();
    let mut visited: Vec<Handle<UiNode>> = roots.to_vec();
    while let Some(h) = stack.pop() {
        if h.is_none() {
            continue;
        }

        // ContextMenu wraps a Popup that is not its own node.
        if let Ok(cm) = ui.try_get_of_type::<fyrox::gui::menu::ContextMenu>(h) {
            let popup_node: Handle<UiNode> = cm.popup.widget.handle;
            if !popup_node.is_none() && !visited.contains(&popup_node) {
                visited.push(popup_node);
                stack.push(popup_node);
            }
        }
        // Popup bodies are Borders with a stock dark background.
        if let Ok(popup) = ui.try_get_of_type::<Popup>(h) {
            let body: Handle<UiNode> = *popup.body;
            if !body.is_none() {
                ui.send(
                    body,
                    fyrox::gui::widget::WidgetMessage::Background(elevated.clone().into()),
                );
                ui.send(
                    body,
                    fyrox::gui::widget::WidgetMessage::Foreground(stroke.clone().into()),
                );
                if let Ok(border) = ui.try_get_mut_of_type::<Border>(body) {
                    border
                        .corner_radius
                        .set_value_and_mark_modified(4.0f32.into());
                }
            }
        }
        // Dropdown menus keep their popup outside the widget tree.
        if let Ok(dropdown) = ui.try_get_of_type::<DropdownMenu>(h) {
            let popup: Handle<UiNode> = dropdown.popup.to_base();
            if !popup.is_none() && !visited.contains(&popup) {
                visited.push(popup);
                stack.push(popup);
            }
        }
        // Menu items carry their own sub-menu popups + a Decorator child.
        if let Ok(item) = ui.try_get_of_type::<fyrox::gui::menu::MenuItem>(h) {
            let items_panel = item.items_panel.to_base();
            if !items_panel.is_none() && !visited.contains(&items_panel) {
                visited.push(items_panel);
                stack.push(items_panel);
            }
        }
        if let Some(decorator) = item_decorator(ui, h) {
            ui.send(
                decorator,
                DecoratorMessage::NormalBrush(
                    Brush::Solid(fyrox::core::color::Color::TRANSPARENT).into(),
                ),
            );
            ui.send(
                decorator,
                DecoratorMessage::HoverBrush(hover.clone().into()),
            );
            ui.send(
                decorator,
                DecoratorMessage::PressedBrush(pressed.clone().into()),
            );
            ui.send(
                decorator,
                DecoratorMessage::SelectedBrush(hover.clone().into()),
            );

            // Fluent flyout rows: ~32px tall with the content grid
            // (already center-aligned by fyrox) vertically centered.
            ui.send(h, WidgetMessage::Height(FLUENT_MENU_ROW_HEIGHT));
            ui.send(decorator, WidgetMessage::Height(FLUENT_MENU_ROW_HEIGHT));

            // Fluent flyouts hug their content: the stock item grid uses a
            // Stretch text column which absorbs any leftover width handed to
            // the popup, stretching short flyouts across the full available
            // width. Swap it for Auto so grids measure naturally.
            if let Some(&grid_handle) = ui.node(decorator).children().first() {
                if let Ok(grid) = ui.try_get_mut_of_type::<fyrox::gui::grid::Grid>(grid_handle) {
                    let cols: Vec<fyrox::gui::grid::Column> =
                        grid.columns.borrow().clone();
                    if cols.len() == 5
                        && cols[1].size_mode == fyrox::gui::grid::SizeMode::Stretch
                    {
                        let mut cols = cols;
                        cols[1] = fyrox::gui::grid::Column::auto();
                        grid.columns
                            .set_value_and_mark_modified(std::cell::RefCell::new(cols));
                    }
                }
            }
        }

        for child in ui.node(h).children().to_vec() {
            if child.is_some() && !visited.contains(&child) {
                visited.push(child);
                stack.push(child);
            }
        }
    }

    // Disabled items: block interaction and dim per Fluent (muted label,
    // no hover/pressed chrome). Applied after the walk so it wins.
    let fallback_disabled = raikou_core::Color::new(0.478, 0.478, 0.478, 1.0);
    let dim = Brush::Solid(to_fyrox_color(
        theme
            .color("fluent.chrome.disabled.low")
            .unwrap_or(fallback_disabled),
    ));
    for &item in disabled {
        if item.is_none() {
            continue;
        }
        ui.send(item, WidgetMessage::Enabled(false));
        let Some(decorator) = item_decorator(ui, item) else {
            continue;
        };
        ui.send(
            decorator,
            DecoratorMessage::HoverBrush(transparent.clone().into()),
        );
        ui.send(
            decorator,
            DecoratorMessage::PressedBrush(transparent.clone().into()),
        );
        ui.send(
            decorator,
            DecoratorMessage::SelectedBrush(transparent.clone().into()),
        );
        // Dim every text under the item's decorator.
        let mut stack = vec![decorator];
        while let Some(cur) = stack.pop() {
            for child in ui.node(cur).children().to_vec() {
                stack.push(child);
            }
            if ui.try_get_of_type::<Text>(cur).is_ok() {
                ui.send(cur, WidgetMessage::Foreground(dim.clone().into()));
            }
        }
    }
}

/// Returns the handle of `item`'s content Decorator, if it has exactly one
/// (the stock fyrox MenuItem shape).
fn item_decorator(ui: &UserInterface, item: Handle<UiNode>) -> Option<Handle<UiNode>> {
    let node = ui.node(item);
    if node.children.len() == 1 && ui.try_get_of_type::<Decorator>(node.children[0]).is_ok() {
        Some(node.children[0])
    } else {
        None
    }
}

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
                if let Some(index) = self
                    .item_handles
                    .iter()
                    .position(|h| *h == message.destination())
                {
                    let enabled = ui
                        .try_get(message.destination())
                        .map(|node| node.enabled())
                        .unwrap_or(false);
                    if enabled {
                        on_item_click(ui, index);
                    }
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
        let mut disabled_handles: Vec<Handle<UiNode>> = Vec::new();

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

            let item_nodes = build_items(items, &mut item_handles, &mut disabled_handles, &mut ctx);
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
                    .with_children(dropdown_handles.clone()),
            )
            .with_orientation(fyrox::gui::Orientation::Horizontal)
            .build(&mut ctx);
            panel.to_base()
        };

        // Fluent styling for the bar's dropdown popups + item decorators.
        {
            let mut roots = vec![handle];
            roots.extend(dropdown_handles.iter().copied());
            let theme = cx.theme().clone();
            style_menu_chrome(cx.ui(), &theme, &roots, &disabled_handles);
        }
        let _ = dropdown_handles;

        let kind = ComponentKind::MenuBar(MenuBarHandlers {
            item_handles: item_handles.clone(),
            on_item_click: self.on_item_click,
        });
        // Clicks target individual item handles, so the handlers must be
        // reachable from every item destination, not just the bar root.
        cx.register(&Component {
            handle,
            kind: kind.clone(),
        });
        for item in &item_handles {
            cx.register(&Component {
                handle: *item,
                kind: kind.clone(),
            });
        }
        Component { handle, kind }
    }
}

/// Recursively builds a set of menu items into fyrox `MenuItem` widgets,
/// recording leaf handles and their flat index, plus the handles of any
/// disabled items.
pub(crate) fn build_items(
    items: &[MenuItem],
    item_handles: &mut Vec<Handle<UiNode>>,
    disabled_handles: &mut Vec<Handle<UiNode>>,
    ctx: &mut fyrox::gui::BuildContext,
) -> Vec<Handle<fyrox::gui::menu::MenuItem>> {
    let mut handles = Vec::new();
    for item in items {
        let mut builder = MenuItemBuilder::new(WidgetBuilder::new().with_name("raikou_menu_item"))
            .with_content(MenuItemContent::text(&item.label));

        if !item.children.is_empty() {
            let sub = build_items(&item.children, item_handles, disabled_handles, ctx);
            builder = builder.with_items(sub);
        }

        let handle = builder.build(ctx);
        let base: Handle<UiNode> = handle.to_base();
        item_handles.push(base);
        if !item.enabled {
            disabled_handles.push(base);
        }
        handles.push(handle);
    }
    handles
}

pub type MenuBarHandle = Handle<UiNode>;
