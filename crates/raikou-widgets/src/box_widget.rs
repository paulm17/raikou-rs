//! The Box component — a styled rectangle (background + border + corner radius).

use fyrox::core::pool::Handle;
use fyrox::gui::border::BorderBuilder;
use fyrox::gui::brush::Brush;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use raikou_core::{Color, Length, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

/// Builder for a [`crate::BoxWidget`] component.
#[derive(Clone)]
pub struct BoxWidget {
    width: Length,
    height: Length,
    background: Color,
    border_color: Color,
    border_width: f32,
    corner_radius: f32,
    margin: Thickness,
}

impl Default for BoxWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxWidget {
    /// Creates a new box builder.
    pub fn new() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            corner_radius: 0.0,
            margin: Thickness::ZERO,
        }
    }

    /// Sets an explicit width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets an explicit height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the fill color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.background = color.into();
        self
    }

    /// Sets the border stroke color.
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = color.into();
        self
    }

    /// Sets the border stroke width.
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width.max(0.0);
        self
    }

    /// Sets the corner radius in logical pixels.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the box and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut widget_builder = WidgetBuilder::new()
            .with_name("raikou_box")
            .with_margin(to_fyrox_thickness(self.margin))
            .with_background(Brush::Solid(to_fyrox_color(self.background)).into())
            .with_foreground(Brush::Solid(to_fyrox_color(self.border_color)).into());
        if let Some(width) = self.width.resolve() {
            widget_builder = widget_builder.with_width(width);
        }
        if let Some(height) = self.height.resolve() {
            widget_builder = widget_builder.with_height(height);
        }

        let mut border = BorderBuilder::new(widget_builder)
            .with_corner_radius(self.corner_radius.into())
            .with_pad_by_corner_radius(false);
        if self.border_width > 0.0 {
            border = border.with_stroke_thickness(
                to_fyrox_thickness(Thickness::uniform(self.border_width)).into(),
            );
        }

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            border.build(&mut ctx).to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built box.
pub type BoxHandle = Handle<UiNode>;
