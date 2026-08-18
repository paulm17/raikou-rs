//! raikou — a component API for fyrox.
//!
//! Facade crate over the layered raikou crates. Re-exports the full public
//! surface so consumers can use `raikou::prelude::*` (or `raikou::*`) without
//! knowing which layer a type lives in.
//!
//! - `raikou-core`: sizing primitives ([`Length`], [`ControlSize`]).
//! - `raikou-style`: theme tokens and style resolution ([`Theme`],
//!   [`ButtonStyle`], [`ButtonVariant`]).
//! - `raikou-widgets`: component builders and the dispatch seam
//!   ([`Button`], [`BuildCx`], [`Component`], [`ComponentRegistry`]).

pub use raikou_core::*;
pub use raikou_style::*;
pub use raikou_widgets::*;

pub mod prelude;
