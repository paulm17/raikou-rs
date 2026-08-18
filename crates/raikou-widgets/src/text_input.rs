//! The TextInput component (single-line text field).
//!
//! Maps onto fyrox's `TextBoxBuilder` and reports text changes through an
//! `on_change` handler. Uses `TextCommitMode::Immediate` so the callback fires
//! on every edit.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::text::TextMessage;
use fyrox::gui::text_box::{EmptyTextPlaceholder, TextCommitMode, TextBoxBuilder};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, &str);

/// Event handlers of a TextInput component.
#[derive(Clone)]
pub struct TextInputHandlers {
    /// Invoked with the current text whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl TextInputHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, text);
            }
        }
    }
}

/// Builder for a [`TextInput`] component.
#[derive(Clone)]
pub struct TextInput {
    text: String,
    placeholder: String,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a new text input builder.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the initial text value.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Sets the placeholder text shown when the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the text changes.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, &str) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the text input, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_text_input")
            .with_margin(to_fyrox_thickness(self.margin));

        let placeholder = self.placeholder.clone();
        let handle = {
            let mut ctx = cx.ctx();
            let mut builder = TextBoxBuilder::new(widget_builder)
                .with_text(&self.text)
                .with_text_commit_mode(TextCommitMode::Immediate);
            if !placeholder.is_empty() {
                builder = builder.with_empty_text_placeholder(EmptyTextPlaceholder::Text(&placeholder));
            }
            builder.build(&mut ctx).to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::TextInput(TextInputHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built text input, returned for convenience.
pub type TextInputHandle = Handle<UiNode>;
