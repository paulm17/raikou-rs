//! The ProgressBar component — an indeterminate/determinate horizontal bar.

use fyrox::core::pool::Handle;
use fyrox::gui::progress_bar::{ProgressBarBuilder, ProgressBarMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use raikou_core::{Color, Length, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

/// Builder for a [`crate::ProgressBar`] component.
#[derive(Clone)]
pub struct ProgressBar {
    value: f32,
    width: Length,
    height: f32,
    track_color: Option<Color>,
    fill_color: Option<Color>,
    margin: Thickness,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    /// Creates a new progress bar builder.
    pub fn new() -> Self {
        Self {
            value: 0.0,
            width: Length::Fixed(200.0),
            height: 8.0,
            track_color: None,
            fill_color: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the fill progress (clamped to 0..=1).
    pub fn value(mut self, v: f32) -> Self {
        self.value = v.clamp(0.0, 1.0);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height in logical pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    /// Sets the track (background) color.
    pub fn track_color(mut self, color: impl Into<Color>) -> Self {
        self.track_color = Some(color.into());
        self
    }

    /// Sets the fill (indicator) color.
    pub fn fill_color(mut self, color: impl Into<Color>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the progress bar and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let width = self.width.resolve().unwrap_or(200.0);
        let track_color = self.track_color.unwrap_or_else(|| {
            cx.theme()
                .color("surface.muted")
                .unwrap_or(Color::new(0.88, 0.89, 0.91, 1.0))
        });
        let fill_color = self.fill_color.unwrap_or_else(|| {
            cx.theme()
                .color("accent.solid")
                .unwrap_or(Color::new(0.13, 0.39, 0.94, 1.0))
        });

        // fyrox paints its default indicator with Style::BRUSH_BRIGHTEST
        // (near-black ink here), so supply an explicit fill-colored indicator.
        let indicator = {
            use fyrox::gui::border::BorderBuilder;
            let mut ctx = cx.ctx();
            BorderBuilder::new(
                WidgetBuilder::new().with_background(
                    fyrox::gui::brush::Brush::Solid(to_fyrox_color(fill_color)).into(),
                ),
            )
            .build(&mut ctx)
            .to_base()
        };

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            ProgressBarBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_progress_bar")
                    .with_width(width)
                    .with_height(self.height)
                    .with_margin(to_fyrox_thickness(self.margin))
                    .with_background(
                        fyrox::gui::brush::Brush::Solid(to_fyrox_color(track_color)).into(),
                    ),
            )
            .with_indicator(indicator)
            .with_progress(self.value)
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

/// A handle to a built progress bar.
pub type ProgressBarHandle = Handle<UiNode>;

/// Helper to update a built progress bar's value at runtime.
pub fn set_progress(ui: &fyrox::gui::UserInterface, bar: Handle<UiNode>, value: f32) {
    ui.send(bar, ProgressBarMessage::Progress(value.clamp(0.0, 1.0)));
}
