//! The Switch component.
//!
//! Maps onto fyrox's `ToggleButtonBuilder` and reports state through an
//! `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::toggle::{ToggleButtonBuilder, ToggleButtonMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, bool);

/// Event handlers of a Switch component.
#[derive(Clone)]
pub struct SwitchHandlers {
    /// Invoked with the new toggled state whenever the switch flips.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl SwitchHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(ToggleButtonMessage::Toggled(state)) = message.data::<ToggleButtonMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, *state);
            }
        }
    }
}

/// Builder for a [`Switch`] component.
#[derive(Clone)]
pub struct Switch {
    label: String,
    toggled: bool,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Switch {
    /// Creates a new switch builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            toggled: false,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the switch label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Sets the initial toggled state.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the switch flips.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, bool) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the switch, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let label_handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
                .with_text(&self.label)
                .with_font(font)
                .build(&mut ctx)
                .to_base()
        };

        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_switch")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            ToggleButtonBuilder::new(widget_builder)
                .with_toggled(self.toggled)
                .with_content(label_handle)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Switch(SwitchHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built switch, returned for convenience.
pub type SwitchHandle = Handle<UiNode>;
