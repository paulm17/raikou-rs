//! Theme provider infrastructure.
//!
//! Theme providers supply token values and component recipes to the
//! style resolution system. They can be scoped to subtrees.

use crate::property::Shadow;
use crate::recipe::{ComponentRecipe, RecipeKey};
use crate::style_value::TokenScale;
use crate::Theme;
use raikou_core::Color;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum TokenValue {
    Color(Color),
    F32(f32),
    String(String),
    Shadow(Shadow),
}

impl TokenValue {
    pub fn as_color(&self) -> Option<Color> {
        match self {
            TokenValue::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            TokenValue::F32(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            TokenValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

pub trait ThemeProvider: Send + Sync {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue>;

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>>;

    fn variant(&self) -> ThemeVariant;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    Light,
    Dark,
    #[default]
    Default,
}

impl ThemeVariant {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    pub fn is_light(&self) -> bool {
        matches!(self, Self::Light)
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
            Self::Default => write!(f, "default"),
        }
    }
}

pub trait ThemeProviderExt: ThemeProvider {
    fn with_variant(self, variant: ThemeVariant) -> ThemeVariantProvider<Self>
    where
        Self: Sized,
    {
        ThemeVariantProvider {
            inner: self,
            variant,
        }
    }
}

impl<T: ThemeProvider> ThemeProviderExt for T {}

#[derive(Clone, Debug)]
pub struct ThemeVariantProvider<T: ThemeProvider> {
    inner: T,
    variant: ThemeVariant,
}

impl<T: ThemeProvider> ThemeProvider for ThemeVariantProvider<T> {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        self.inner.get_token(scale, name)
    }

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        self.inner.get_component_recipe(key)
    }

    fn variant(&self) -> ThemeVariant {
        self.variant
    }
}

pub struct ScopedThemeProvider<'a> {
    parent: &'a dyn ThemeProvider,
    overrides: &'a Theme,
}

impl<'a> ScopedThemeProvider<'a> {
    pub fn new(parent: &'a dyn ThemeProvider, overrides: &'a Theme) -> Self {
        Self { parent, overrides }
    }
}

impl<'a> ThemeProvider for ScopedThemeProvider<'a> {
    fn get_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        self.overrides
            .get_token(scale, name)
            .or_else(|| self.parent.get_token(scale, name))
    }

    fn get_component_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        self.overrides
            .get_component_recipe(key)
            .or_else(|| self.parent.get_component_recipe(key))
    }

    fn variant(&self) -> ThemeVariant {
        self.overrides.variant()
    }
}

pub struct ThemeResolver<'a> {
    providers: Vec<&'a dyn ThemeProvider>,
}

impl<'a> ThemeResolver<'a> {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: &'a dyn ThemeProvider) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn resolve_token(&self, scale: TokenScale, name: &str) -> Option<TokenValue> {
        for provider in self.providers.iter().rev() {
            if let Some(value) = provider.get_token(scale, name) {
                return Some(value);
            }
        }
        None
    }

    pub fn resolve_recipe(&self, key: &RecipeKey) -> Option<Arc<ComponentRecipe>> {
        for provider in self.providers.iter().rev() {
            if let Some(recipe) = provider.get_component_recipe(key) {
                return Some(recipe);
            }
        }
        None
    }

    pub fn resolve_variant(&self) -> ThemeVariant {
        self.providers
            .last()
            .map(|p| p.variant())
            .unwrap_or(ThemeVariant::Default)
    }
}

impl Default for ThemeResolver<'static> {
    fn default() -> Self {
        Self::new()
    }
}
