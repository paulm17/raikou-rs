//! Combobox component: a read-only dropdown backed by fyrox's `DropdownList`.
//!
//! The reference combobox supports type-to-filter; this port maps it to the
//! native read-only `DropdownList` (the free-text/search behaviour is a
//! stretch target, not part of this phase).

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::dropdown_list::{DropdownList, DropdownListBuilder, DropdownListMessage};
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_color;

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Event handlers of a Combobox component.
#[derive(Clone)]
pub struct ComboboxHandlers {
    on_change: Option<Rc<ChangeCallback>>,
    /// The inner dropdown list that receives programmatic commands.
    command_target: Handle<UiNode>,
    /// Muted text shown when nothing is selected (if any).
    placeholder: Option<Handle<UiNode>>,
}

impl ComboboxHandlers {
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
            // Flip the placeholder with the selection state.
            if let (Some(placeholder), Some(DropdownListMessage::Selection(selected))) =
                (&self.placeholder, message.data::<DropdownListMessage>())
            {
                ui.send(*placeholder, WidgetMessage::Visibility(selected.is_none()));
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
                DropdownListBuilder::new(WidgetBuilder::new().with_name("raikou_combobox_inner"))
                    .with_items(item_nodes);

            if let Some(selected) = self.selected {
                if selected < self.items.len() {
                    builder = builder.with_selected(selected);
                }
            }

            builder.build(&mut ctx).to_base()
        };

        // Nothing selected: plant a muted placeholder text into the inner
        // dropdown's content grid; handlers flip its visibility with the
        // selection state.
        let placeholder = if self.selected.is_none() && !self.placeholder.is_empty() {
            let main_grid = {
                use fyrox::graph::SceneGraph;
                cx.ui()
                    .try_get_of_type::<DropdownList>(inner)
                    .ok()
                    .filter(|dd| dd.current.is_none())
                    .map(|dd| *dd.main_grid)
            };
            main_grid.map(|grid| {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                let fallback_muted = raikou_core::Color::new(0.45, 0.45, 0.45, 1.0);
                let muted = Brush::Solid(to_fyrox_color(
                    theme.color("text.muted").unwrap_or(fallback_muted),
                ));
                let text: Handle<UiNode> = fyrox::gui::text::TextBuilder::new(
                    WidgetBuilder::new()
                        .on_row(0)
                        .on_column(0)
                        .with_vertical_alignment(fyrox::gui::VerticalAlignment::Center)
                        .with_foreground(muted.into()),
                )
                .with_text(&self.placeholder)
                .with_font(font)
                .build(&mut ctx)
                .to_base();
                ctx.link(text, grid);
                text
            })
        } else {
            None
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
            kind: ComponentKind::Combobox(ComboboxHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
                placeholder,
            }),
        };
        cx.register(&component);
        // The inner dropdown list emits the FromWidget messages; register it
        // too so exact-destination dispatch finds the handlers.
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::Combobox(ComboboxHandlers {
                on_change: self.on_change,
                command_target: inner,
                placeholder,
            }),
        });
        component
    }
}

/// A handle to a built combobox.
pub type ComboboxHandle = Handle<UiNode>;
