//! raikou-widgets — the raikou component builders.
//!
//! Each component is a builder (`Button::new()...build(cx)`) that constructs
//! the equivalent native fyrox widget, wraps it in a [`Component`], and
//! registers its handlers into a [`ComponentRegistry`] for dispatch from the
//! app's message poll loop.

pub mod build_cx;
pub mod button;
pub mod component;
pub mod registry;

pub use build_cx::BuildCx;
pub use button::{Button, ButtonHandle};
pub use component::{ClickEvent, Component, ComponentKind};
pub use registry::ComponentRegistry;
