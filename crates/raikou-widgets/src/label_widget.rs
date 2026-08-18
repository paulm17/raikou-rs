//! The Label component — static text with color and font size.

use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::text::{TextBuilder, TextMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use raikou_core::{Color, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

/// Builder for a [`crate::Label`] component.
#[derive(Clone)]
pub struct Label {
    text: String,
    color: Option<Color>,
    font_size: f32,
    margin: Thickness,
}

impl Label {
    /// Creates a new label builder with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            font_size: 16.0,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Sets the text color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the font size in logical pixels.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the label and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_label")
            .with_margin(to_fyrox_thickness(self.margin));
        let widget_builder = if let Some(color) = self.color {
            widget_builder.with_foreground(Brush::Solid(to_fyrox_color(color)).into())
        } else {
            widget_builder
        };

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            TextBuilder::new(widget_builder)
                .with_text(self.text)
                .with_font(font)
                .with_font_size(self.font_size.into())
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built label.
pub type LabelHandle = Handle<UiNode>;

/// Helper to update a built label's text at runtime.
pub fn set_label_text(ui: &fyrox::gui::UserInterface, label: Handle<UiNode>, text: impl Into<String>) {
    ui.send(label, TextMessage::Text(text.into()));
}
