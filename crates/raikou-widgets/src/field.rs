//! Shared Avalonia/Fluent "field" chrome for input-like controls.
//!
//! Wraps an already-built inner widget (text box, dropdown, numeric field...)
//! in a rounded 1px border so every field-type control shares the same
//! Fluent look: radius 3, stroke `border.default`, min height 32.

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::border::BorderBuilder;
use fyrox::gui::brush::Brush;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{BuildContext, UiNode};

use raikou_core::{Color, Thickness};
use raikou_style::Theme;

use crate::convert::{to_fyrox_color, to_fyrox_thickness};

/// Standard Avalonia control minimum height.
pub const FIELD_MIN_HEIGHT: f32 = 32.0;

/// Builds a Fluent-styled chrome border around an already-built inner widget
/// and returns the outer handle. The caller keeps the margin on its own outer
/// widget builder; the inner widget fills the chrome.
pub fn field_chrome(
    ctx: &mut BuildContext,
    theme: &Theme,
    inner: Handle<UiNode>,
    min_height: f32,
    margin: Thickness,
) -> Handle<UiNode> {
    let fill = theme
        .color("fluent.control.solid")
        .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0));
    let stroke = theme
        .color("border.default")
        .unwrap_or(Color::new(0.0, 0.0, 0.0, 0.4));

    BorderBuilder::new(
        WidgetBuilder::new()
            .with_name("raikou_field_chrome")
            .with_min_size(Vector2::new(0.0, min_height))
            .with_margin(to_fyrox_thickness(margin))
            .with_foreground(Brush::Solid(to_fyrox_color(stroke)).into())
            .with_background(Brush::Solid(to_fyrox_color(fill)).into())
            .with_child(inner),
    )
    .with_corner_radius(3.0.into())
    .with_stroke_thickness(fyrox::gui::Thickness::uniform(1.0).into())
    .build(ctx)
    .to_base()
}
