//! The TextArea component (multiline text field).
//!
//! Maps onto fyrox's `TextBoxBuilder` with `with_multiline(true)` and reports
//! text changes through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::text::TextMessage;
use fyrox::gui::text_box::{TextCommitMode, TextBoxBuilder};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, &str);

/// Event handlers of a TextArea component.
#[derive(Clone)]
pub struct TextAreaHandlers {
    /// Invoked with the current text whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl TextAreaHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, text);
            }
        }
    }
}

/// Builder for a [`TextArea`] component.
#[derive(Clone)]
pub struct TextArea {
    text: String,
    rows: usize,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    /// Creates a new text area builder.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            rows: 4,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the initial text value.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Sets the number of visible rows.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
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

    /// Builds the text area, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_text_area")
            .with_margin(to_fyrox_thickness(self.margin))
            .with_height(24.0 * self.rows as f32);

        let handle = {
            let mut ctx = cx.ctx();
            TextBoxBuilder::new(widget_builder)
                .with_text(&self.text)
                .with_multiline(true)
                .with_text_commit_mode(TextCommitMode::Immediate)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::TextArea(TextAreaHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built text area, returned for convenience.
pub type TextAreaHandle = Handle<UiNode>;
