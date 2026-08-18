//! Layout primitives: sizes, spacing and control size ladder.

use crate::geometry::Thickness;

/// How a widget should resolve its width/height.
///
/// `Auto`/`Shrink` let the widget size itself to its content, `Fixed` pins an
/// exact size, `Fill` stretches to the available space of the parent and
/// `Percent` resolves against a fraction of the parent.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    /// Let the widget size itself to its content.
    #[default]
    Auto,
    /// Shrink to content (same as `Auto` for the fyrox facade).
    Shrink,
    /// A fixed size in logical pixels.
    Fixed(f32),
    /// Stretch to the available space of the parent.
    Fill,
    /// A percentage (0-100) of the available parent space.
    Percent(f32),
}

impl Length {
    pub fn auto() -> Self {
        Self::Auto
    }

    pub fn shrink() -> Self {
        Self::Shrink
    }

    pub fn fill() -> Self {
        Self::Fill
    }

    /// Creates a fixed length in logical pixels.
    pub fn fixed(value: f32) -> Self {
        Self::Fixed(value)
    }

    /// Creates a percentage length, clamped to 0-100.
    pub fn percent(value: f32) -> Self {
        Self::Percent(value.clamp(0.0, 100.0))
    }

    /// Creates a fixed length in logical pixels.
    pub fn px(value: f32) -> Self {
        Self::Fixed(value)
    }

    /// Resolves this length to an explicit pixel value, or `None` when the
    /// layout should keep the default (auto) sizing.
    ///
    /// Percentages resolve to `None` because no parent size is available in a
    /// build-time context.
    pub fn resolve(self) -> Option<f32> {
        match self {
            Length::Fixed(value) => Some(value),
            _ => None,
        }
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Fixed(value)
    }
}

impl From<Length> for f32 {
    fn from(length: Length) -> Self {
        match length {
            Length::Fixed(value) => value,
            _ => 0.0,
        }
    }
}

/// Per-edge padding.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn horizontal(value: f32) -> Self {
        Self {
            top: 0.0,
            right: value,
            bottom: 0.0,
            left: value,
        }
    }

    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            right: 0.0,
            bottom: value,
            left: 0.0,
        }
    }

    pub fn to_thickness(&self) -> Thickness {
        Thickness::new(self.left, self.top, self.right, self.bottom)
    }

    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }
}

pub type Margin = Padding;

/// Four-corner radius set.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Radius {
    pub fn all(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top_left: horizontal,
            top_right: vertical,
            bottom_right: horizontal,
            bottom_left: vertical,
        }
    }

    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn pill() -> Self {
        Self::all(999.0)
    }

    pub fn top(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: 0.0,
            bottom_left: 0.0,
        }
    }

    pub fn bottom(value: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: value,
            bottom_left: value,
        }
    }

    pub fn left(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: value,
        }
    }

    pub fn right(value: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: value,
            bottom_right: value,
            bottom_left: 0.0,
        }
    }
}

/// Standard control size ladder, ported from the raikou `ControlSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ControlSize {
    XSmall,
    Small,
    #[default]
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
        Thickness::new(x, y, x, y)
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

    /// Preferred icon size of the control in logical pixels.
    pub fn icon_size(self) -> f32 {
        match self {
            ControlSize::XSmall => 12.0,
            ControlSize::Small => 14.0,
            ControlSize::Medium => 16.0,
            ControlSize::Large => 20.0,
            ControlSize::XLarge => 24.0,
        }
    }
}
