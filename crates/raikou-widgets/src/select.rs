//! Select component: a read-only dropdown backed by fyrox's `DropdownList`.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::dropdown_list::{DropdownListBuilder, DropdownListMessage};
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Event handlers of a Select component.
#[derive(Clone)]
pub struct SelectHandlers {
    on_change: Option<Rc<ChangeCallback>>,
    /// The inner dropdown list that receives programmatic commands.
    command_target: Handle<UiNode>,
}

impl SelectHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(selection) = message.data::<DropdownListMessage>() {
            // Forward ToWidget commands aimed at the outer chrome to the
            // inner dropdown list (skips the forwarded copy itself).
            if message.direction() == MessageDirection::ToWidget
                && message.destination() != self.command_target
            {
                ui.send(self.command_target, selection.clone());
                return;
            }
            if message.direction() != MessageDirection::FromWidget {
                return;
            }
            if let Some(on_change) = &self.on_change {
                if let Some(DropdownListMessage::Selection(Some(index))) =
                    message.data::<DropdownListMessage>()
                {
                    on_change(ui, *index);
                }
            }
        }
    }
}

/// Builder for a [`Select`] component (read-only dropdown).
#[derive(Clone)]
pub struct Select {
    items: Vec<String>,
    selected: Option<usize>,
    placeholder: String,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}

impl Select {
    /// Creates a new select builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            placeholder: String::new(),
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the selectable items.
    pub fn items(mut self, items: Vec<impl Into<String>>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the initially selected item index (clamped to the item count).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = (index < self.items.len()).then_some(index);
        self
    }

    /// Sets the placeholder text shown when nothing is selected.
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

    /// Builds the select, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();

        let inner = {
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

            let mut builder =
                DropdownListBuilder::new(WidgetBuilder::new().with_name("raikou_select_inner"))
                    .with_items(item_nodes);

            if let Some(selected) = self.selected {
                builder = builder.with_selected(selected);
            }

            builder.build(&mut ctx).to_base()
        };

        let handle = {
            let mut ctx = cx.ctx();
            crate::field::field_chrome(
                &mut ctx,
                &theme,
                inner,
                crate::field::FIELD_MIN_HEIGHT,
                self.margin,
            )
        };

        let component = Component {
            handle,
            kind: ComponentKind::Select(SelectHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
            }),
        };
        cx.register(&component);
        // The inner dropdown list emits the FromWidget messages; register it
        // too so exact-destination dispatch finds the handlers.
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::Select(SelectHandlers {
                on_change: self.on_change,
                command_target: inner,
            }),
        });
        component
    }
}

/// A handle to a built select.
pub type SelectHandle = Handle<UiNode>;
