//! raikou-style — design tokens and visual style resolution.
//!
//! The theme seam that resolves a component's variant/size/state into concrete
//! fyrox values (colors, corner radius, border). The body is a small hardcoded
//! token set for now; the full raikou recipe/state theme system (tokens,
//! variants, pseudoclasses) will replace these bodies in a later phase without
//! changing the builder API.

pub mod theme;

pub use theme::{ButtonStyle, ButtonVariant, Theme};
