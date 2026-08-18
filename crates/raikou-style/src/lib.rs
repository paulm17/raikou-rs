//! raikou-style — design tokens and visual style resolution.
//!
//! A faithful port of the raikou style system: token scales (colors, spacing,
//! radii, typography, shadows), property-backed styles, recipes with variants,
//! and widget state/pseudoclass resolution.

pub mod inheritance;
pub mod property;
pub mod recipe;
pub mod state;
pub mod style;
pub mod style_resolver;
pub mod style_value;
pub mod theme;

pub use inheritance::{Inheritable, InheritedProperty, InheritedStyles};
pub use property::{
    Property, PropertyId, PropertyRegistry, StyledProperty, TextAlign, TextDecoration,
};
pub use recipe::{ComponentRecipe, RecipeKey, VariantMap};
pub use state::{StatePriority, StateTracker, StateTransition, WidgetState};
pub use style::{ResolvedStyle, Style, StyleBlock, StyleResolver};
pub use style_value::{ColorValue, SizeValue, SpacingValue, StyleValue};
pub use theme::{
    ButtonStyle, ButtonVariant, ColorScale, ComponentThemeRegistry, ControlTheme,
    ControlThemeRegistry, RadiusScale, ShadowScale, SizeScale, SpaceScale, Theme, ThemeBuilder,
    ThemeProvider, ThemeVariant, TypographyScale,
};
