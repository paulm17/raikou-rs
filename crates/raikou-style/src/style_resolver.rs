//! Style resolution and precedence handling.
//!
//! This module provides the style resolver that determines the final
//! value for any styleable property based on the precedence model:
//!
//! 1. Animation (highest precedence)
//! 2. Local fluent override
//! 3. State style (hover, pressed, focus, disabled)
//! 4. Component variant and compound variant
//! 5. Component base recipe
//! 6. Inherited theme/text defaults
//! 7. Property default (lowest precedence)

pub use crate::style::ResolvedStyle;
