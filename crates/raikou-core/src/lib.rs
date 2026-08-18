//! raikou-core — backend-agnostic shared primitives for all raikou components.
//!
//! Geometry, paint and layout concepts that carry no widget or theme logic:
//! colors, sizes, rects, spacing and the control size ladder.

pub mod geometry;
pub mod layout;

pub use geometry::{Color, CornerRadii, CornerRadius, Point, Rect, RoundedRect, Size, Thickness};
pub use layout::{ControlSize, Length, Margin, Padding, Radius};
