//! Theme variant support for light/dark mode and custom themes.
//!
//! This module provides theme variant infrastructure including
//! `ThemeVariantScope` for subtree-level variant overrides.

use crate::recipe::{ComponentRecipe, RecipeKey};
use crate::style_value::TokenScale;
use crate::theme::ThemeProvider as ThemeProviderTrait;
use crate::theme::provider::{ThemeProvider, ThemeVariant, TokenValue};
use std::sync::Arc;

pub struct ThemeVariantScope {
    variant: ThemeVariant,
    parent: Option<Box<dyn ThemeProvider>>,
}

impl ThemeVariantScope {
    pub fn new(variant: ThemeVariant) -> Self {
        Self {
            variant,
            parent: None,
        }
    }

    pub fn with_parent(mut self, parent: impl ThemeProvider + 'static) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    pub fn variant(&self) -> ThemeVariant {
        self.variant
    }

    pub fn set_variant(&mut self, variant: ThemeVariant) {
        self.variant = variant;
    }
}

impl ThemeProviderTrait for ThemeVariantScope {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        if let Some(ref parent) = self.parent {
            parent.get_token(scale, name)
        } else {
            None
        }
    }

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        self.parent.as_ref()?.get_component_recipe(key)
    }

    fn variant(&self) -> ThemeVariant {
        self.variant
    }
}

pub struct ThemeVariantOverride {
    scope: ThemeVariantScope,
}

impl ThemeVariantOverride {
    pub fn new(variant: ThemeVariant) -> Self {
        Self {
            scope: ThemeVariantScope::new(variant),
        }
    }

    pub fn with_parent(mut self, parent: impl ThemeProvider + 'static) -> Self {
        self.scope = self.scope.with_parent(parent);
        self
    }

    pub fn variant(&self) -> ThemeVariant {
        self.scope.variant()
    }

    pub fn set_variant(&mut self, variant: ThemeVariant) {
        self.scope.set_variant(variant);
    }
}

impl ThemeProviderTrait for ThemeVariantOverride {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        self.scope.get_token(scale, name)
    }

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        self.scope.get_component_recipe(key)
    }

    fn variant(&self) -> ThemeVariant {
        self.scope.variant()
    }
}

pub mod prelude {
    pub use super::{ThemeVariantOverride, ThemeVariantScope};
    pub use crate::theme::provider::ThemeVariant;
}
