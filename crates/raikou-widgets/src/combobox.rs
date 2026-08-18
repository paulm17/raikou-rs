//! Combobox component: a read-only dropdown backed by fyrox's `DropdownList`.
//!
//! The reference combobox supports type-to-filter; this port maps it to the
//! native read-only `DropdownList` (the free-text/search behaviour is a
//! stretch target, not part of this phase).

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::dropdown_list::{DropdownListBuilder, DropdownListMessage};
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Event handlers of a Combobox component.
#[derive(Clone)]
pub struct ComboboxHandlers {
    on_change: Option<Rc<ChangeCallback>>,
}

impl ComboboxHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(on_change) = &self.on_change {
            if let Some(DropdownListMessage::Selection(Some(index))) =
                message.data::<DropdownListMessage>()
            {
                on_change(ui, *index);
            }
        }
    }
}

/// Builder for a [`Combobox`] component (read-only dropdown).
#[derive(Clone)]
pub struct Combobox {
    items: Vec<String>,
    selected: Option<usize>,
    placeholder: String,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Combobox {
    fn default() -> Self {
        Self::new()
    }
}

impl Combobox {
    /// Creates a new combobox builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            placeholder: "Search...".to_string(),
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the selectable items.
    pub fn items(mut self, items: Vec<impl Into<String>>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the initially selected item index (not clamped).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// Sets the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when a selection is made (passes the index).
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the combobox, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();

        let mut item_nodes = Vec::new();
        for item in &self.items {
            let font = ctx.default_font();
            let text = fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
                .with_text(item)
                .with_font(font)
                .build(&mut ctx);
            item_nodes.push(text.to_base());
        }

        let handle = {
            let mut builder = DropdownListBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_combobox")
                    .with_margin(to_fyrox_thickness(self.margin)),
            )
            .with_items(item_nodes);

            if let Some(selected) = self.selected {
                if selected < self.items.len() {
                    builder = builder.with_selected(selected);
                }
            }

            builder.build(&mut ctx).to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Combobox(ComboboxHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built combobox.
pub type ComboboxHandle = Handle<UiNode>;
