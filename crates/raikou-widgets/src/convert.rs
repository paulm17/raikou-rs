//! Conversions between raikou-core backend-agnostic types and fyrox-native
//! types, applied at the boundary where a builder maps onto a fyrox widget.

use fyrox::core::color::Color as FyroxColor;
use fyrox::gui::brush::{Brush, GradientPoint};
use fyrox::gui::Thickness as FyroxThickness;

use raikou_core::{Color, LinearGradient, Point, Thickness};

/// Converts a raikou (f32) color into a fyrox (u8) color.
pub fn to_fyrox_color(color: Color) -> FyroxColor {
    FyroxColor::from_rgba(
        (color.red * 255.0).round() as u8,
        (color.green * 255.0).round() as u8,
        (color.blue * 255.0).round() as u8,
        (color.alpha * 255.0).round() as u8,
    )
}

/// Converts a raikou thickness into a fyrox thickness.
pub fn to_fyrox_thickness(thickness: Thickness) -> FyroxThickness {
    FyroxThickness {
        left: thickness.left,
        top: thickness.top,
        right: thickness.right,
        bottom: thickness.bottom,
    }
}

/// Converts a raikou linear gradient into a fyrox brush.
///
/// fyrox expects `from`/`to` in normalized coordinates, so pass normalized
/// points when constructing the raikou gradient.
pub fn to_fyrox_gradient(gradient: &LinearGradient) -> Brush {
    Brush::LinearGradient {
        from: to_fyrox_point(gradient.start),
        to: to_fyrox_point(gradient.end),
        stops: gradient
            .stops
            .iter()
            .map(|stop| GradientPoint {
                stop: stop.position,
                color: to_fyrox_color(stop.color),
            })
            .collect(),
    }
}

fn to_fyrox_point(point: Point) -> fyrox::core::algebra::Vector2<f32> {
    fyrox::core::algebra::Vector2::new(point.x, point.y)
}
