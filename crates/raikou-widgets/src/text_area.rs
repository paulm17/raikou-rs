//! The TextArea component (multiline text field).
//!
//! Maps onto fyrox's `TextBoxBuilder` with `with_multiline(true)` and reports
//! text changes through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::text::TextMessage;
use fyrox::gui::text_box::{TextBoxBuilder, TextCommitMode};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};

type ChangeCallback = dyn Fn(&mut UserInterface, &str);

/// Event handlers of a TextArea component.
#[derive(Clone)]
pub struct TextAreaHandlers {
    /// Invoked with the current text whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
    /// The inner text box that receives programmatic commands.
    pub command_target: Handle<UiNode>,
}

impl TextAreaHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(text_msg) = message.data::<TextMessage>() {
            // Forward ToWidget commands aimed at the outer chrome to the
            // inner text box (skips the forwarded copy itself).
            if message.direction() == MessageDirection::ToWidget
                && message.destination() != self.command_target
            {
                ui.send(self.command_target, text_msg.clone());
                return;
            }
            if message.direction() != MessageDirection::FromWidget {
                return;
            }
            if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
                if let Some(callback) = &self.on_change {
                    callback(ui, text);
                }
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
        let theme = cx.theme().clone();

        let inner = {
            let widget_builder = WidgetBuilder::new().with_name("raikou_text_area_inner");

            let mut ctx = cx.ctx();
            TextBoxBuilder::new(widget_builder)
                .with_text(&self.text)
                .with_multiline(true)
                .with_text_commit_mode(TextCommitMode::Immediate)
                .build(&mut ctx)
                .to_base()
        };

        let handle = {
            let mut ctx = cx.ctx();
            crate::field::field_chrome(
                &mut ctx,
                &theme,
                inner,
                24.0 * self.rows as f32,
                self.margin,
            )
        };

        let component = Component {
            handle,
            kind: ComponentKind::TextArea(TextAreaHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
            }),
        };
        cx.register(&component);
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::TextArea(TextAreaHandlers {
                on_change: self.on_change,
                command_target: inner,
            }),
        });
        component
    }
}

/// A handle to a built text area, returned for convenience.
pub type TextAreaHandle = Handle<UiNode>;
