//! Sizing primitives shared by all raikou components.

use fyrox::gui::Thickness;

/// How a widget should resolve its width/height.
///
/// Mirrors the raikou `Length` concept: `Auto`/`Shrink` let the widget size
/// itself to its content, `Fixed` pins an exact size and `Fill` stretches to
/// the available space of the parent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// Let the widget size itself to its content.
    Auto,
    /// Shrink to content (same as `Auto` for the fyrox facade).
    Shrink,
    /// A fixed size in logical pixels.
    Fixed(f32),
    /// Stretch to the available space of the parent.
    Fill,
}

impl Length {
    /// Creates a fixed length in logical pixels.
    pub fn px(value: f32) -> Self {
        Self::Fixed(value)
    }

    /// Resolves this length to an explicit pixel value, or `None` when the
    /// layout should keep the default (auto) sizing.
    pub fn resolve(self) -> Option<f32> {
        match self {
            Length::Fixed(value) => Some(value),
            _ => None,
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::Auto
    }
}

/// Standard control size ladder, ported from the raikou `ControlSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlSize {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}

impl ControlSize {
    /// Minimum clickable height of the control in logical pixels.
    pub fn min_height(self) -> f32 {
        match self {
            ControlSize::XSmall => 22.0,
            ControlSize::Small => 26.0,
            ControlSize::Medium => 30.0,
            ControlSize::Large => 36.0,
            ControlSize::XLarge => 42.0,
        }
    }

    /// Content padding of the control.
    pub fn padding(self) -> Thickness {
        let (x, y) = match self {
            ControlSize::XSmall => (8.0, 4.0),
            ControlSize::Small => (10.0, 6.0),
            ControlSize::Medium => (12.0, 8.0),
            ControlSize::Large => (14.0, 10.0),
            ControlSize::XLarge => (16.0, 12.0),
        };
        Thickness {
            left: x,
            right: x,
            top: y,
            bottom: y,
        }
    }

    /// Font size of the control label in logical pixels.
    pub fn font_size(self) -> f32 {
        match self {
            ControlSize::XSmall => 11.0,
            ControlSize::Small => 12.0,
            ControlSize::Medium => 14.0,
            ControlSize::Large => 16.0,
            ControlSize::XLarge => 18.0,
        }
    }
}

impl Default for ControlSize {
    fn default() -> Self {
        Self::Medium
    }
}
