//! The Tabs component.
//!
//! Maps onto fyrox's `TabControl`. Each tab is assigned a fresh UUID; a change
//! of the active tab is reported through a per-component `on_change` handler
//! with the tab index.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::tab_control::{TabControlBuilder, TabControlMessage, TabDefinition};
use fyrox::gui::text::TextBuilder;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use uuid::Uuid;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

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
        if let Some(TabControlMessage::ActiveTab(Some(uuid))) = message.data::<TabControlMessage>() {
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
