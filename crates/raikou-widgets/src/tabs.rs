//! The Tabs component.
//!
//! Maps onto fyrox's `TabControl`. Each tab is assigned a fresh UUID; a change
//! of the active tab is reported through a per-component `on_change` handler
//! with the tab index.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tab_control::{TabControlBuilder, TabControlMessage, TabDefinition};
use fyrox::gui::text::TextBuilder;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use uuid::Uuid;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Handlers of a Tabs component.
#[derive(Clone)]
pub struct TabsHandlers {
    /// UUIDs of the tabs in build order, used to map active-tab events to indexes.
    pub uuids: Vec<Uuid>,
    /// Invoked with the newly active tab index on change.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl TabsHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(TabControlMessage::ActiveTab(Some(uuid))) = message.data::<TabControlMessage>()
        {
            if let Some(index) = self.uuids.iter().position(|u| u == uuid) {
                if let Some(callback) = &self.on_change {
                    callback(ui, index);
                }
            }
        }
    }
}

/// Builder for a [`Tabs`] component.
#[derive(Clone)]
pub struct Tabs {
    headers: Vec<String>,
    contents: Vec<Handle<UiNode>>,
    initial_tab: usize,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    /// Creates a new tabs builder.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            contents: Vec::new(),
            initial_tab: 0,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Appends a tab with the given header label and content.
    pub fn tab(mut self, label: impl Into<String>, content: Handle<UiNode>) -> Self {
        self.headers.push(label.into());
        self.contents.push(content);
        self
    }

    /// Sets the index of the initially active tab (default 0).
    pub fn initial_tab(mut self, index: usize) -> Self {
        self.initial_tab = index;
        self
    }

    /// Sets the callback invoked with the active tab index on change.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the tabs, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut uuids: Vec<Uuid> = Vec::new();
        let mut tab_builder = TabControlBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_tabs")
                .with_margin(to_fyrox_thickness(self.margin)),
        )
        .with_initial_tab(self.initial_tab);

        for (label, content) in self.headers.iter().zip(self.contents.iter()) {
            let header: Handle<UiNode> = {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                TextBuilder::new(WidgetBuilder::new())
                    .with_text(label)
                    .with_font(font)
                    .build(&mut ctx)
                    .to_base()
            };
            let uuid = Uuid::new_v4();
            uuids.push(uuid);
            tab_builder = tab_builder.with_tab(TabDefinition {
                uuid,
                header,
                content: *content,
                can_be_closed: false,
                user_data: None,
            });
        }

        let handle = {
            let mut ctx = cx.ctx();
            tab_builder.build(&mut ctx).to_base()
        };

        // Fluent chrome: the stock TabControl paints a dark backdrop over
        // headers + content and gives header decorators heavy gray fills.
        // Strip those, give headers transparent/hover/selected brushes and pad
        // the header labels for a comfortable hit target.
        {
            use fyrox::graph::SceneGraph;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::DecoratorMessage;

            let theme = cx.theme().clone();
            let token_brush =
                |name: &str| Brush::Solid(to_fyrox_color(theme.color(name).unwrap())).into();
            let transparent = Brush::Solid(fyrox::core::color::Color::TRANSPARENT);
            let transparent_prop: fyrox::gui::style::StyledProperty<Brush> =
                transparent.clone().into();
            let ui = cx.ui();

            let root_node = ui.node(handle);
            let border_h = *root_node.children().first().expect("tab control border");
            let grid_h = *ui
                .node(border_h)
                .children()
                .first()
                .expect("tab control grid");
            let headers_h = *ui
                .node(grid_h)
                .children()
                .first()
                .expect("headers container");
            ui.send(
                border_h,
                fyrox::gui::widget::WidgetMessage::Background(transparent_prop.clone()),
            );
            ui.send(
                border_h,
                fyrox::gui::widget::WidgetMessage::Foreground(transparent_prop.clone()),
            );

            for header_button in ui.node(headers_h).children().to_vec() {
                // Decorators sit between the button and its content grid;
                // decorator messages must target them directly.
                let Some(decorator) = ui.node(header_button).children().first().copied() else {
                    continue;
                };
                ui.send(
                    decorator,
                    DecoratorMessage::NormalBrush(transparent_prop.clone()),
                );
                ui.send(
                    decorator,
                    DecoratorMessage::HoverBrush(token_brush("fluent.list.low")),
                );
                ui.send(
                    decorator,
                    DecoratorMessage::PressedBrush(token_brush("fluent.list.medium")),
                );
                // Pad the header label so the tab is comfortably clickable.
                if let Some(label) = ui.node(header_button).children().first().copied() {
                    if ui.try_get_of_type::<fyrox::gui::text::Text>(label).is_ok() {
                        ui.send(
                            label,
                            fyrox::gui::widget::WidgetMessage::Margin(fyrox::gui::Thickness {
                                left: 12.0,
                                top: 6.0,
                                right: 12.0,
                                bottom: 6.0,
                            }),
                        );
                    }
                }
            }
        }

        let component = Component {
            handle,
            kind: ComponentKind::Tabs(TabsHandlers {
                uuids,
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built tabs container.
pub type TabsHandle = Handle<UiNode>;
