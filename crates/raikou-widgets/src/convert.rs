//! Conversions between raikou-core backend-agnostic types and fyrox-native
//! types, applied at the boundary where a builder maps onto a fyrox widget.

use fyrox::core::color::Color as FyroxColor;
use fyrox::gui::Thickness as FyroxThickness;

use raikou_core::{Color, Thickness};

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
