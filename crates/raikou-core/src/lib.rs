//! raikou-core — shared primitives for all raikou components.
//!
//! Sizing and layout concepts that carry no widget or theme logic: the
//! [`Length`] size unit and the [`ControlSize`] size ladder.

pub mod length;

pub use length::{ControlSize, Length};
