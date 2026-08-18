//! Token scales for the theme system.
//!
//! This module defines the typed token scales including colors, space,
//! sizes, radii, typography, and shadows.

use crate::property::Shadow;
use crate::recipe::{ComponentRecipe, RecipeKey};
use crate::style_value::TokenScale;
use raikou_core::Color;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ColorScale {
    raw: HashMap<SmolStr, Color>,
    aliases: HashMap<SmolStr, SmolStr>,
}

impl ColorScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raw(&mut self, name: impl Into<SmolStr>, color: Color) -> &mut Self {
        self.raw.insert(name.into(), color);
        self
    }

    pub fn alias(&mut self, alias: impl Into<SmolStr>, target: impl Into<SmolStr>) -> &mut Self {
        self.aliases.insert(alias.into(), target.into());
        self
    }

    pub fn alias_color(&mut self, alias: impl Into<SmolStr>, color: Color) -> &mut Self {
        self.raw.insert(alias.into(), color);
        self
    }

    pub fn resolve(&self, name: &str) -> Option<Color> {
        let name: SmolStr = name.into();
        if let Some(&color) = self.raw.get(&name) {
            return Some(color);
        }
        if let Some(target) = self.aliases.get(&name) {
            return self.resolve(target);
        }
        None
    }

    pub fn get(&self, name: &str) -> Option<Color> {
        self.resolve(name)
    }

    pub fn insert(&mut self, name: impl Into<SmolStr>, color: Color) {
        self.raw.insert(name.into(), color);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpaceScale {
    values: HashMap<SmolStr, f32>,
}

impl SpaceScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<SmolStr>, value: f32) -> &mut Self {
        self.values.insert(name.into(), value);
        self
    }

    pub fn resolve(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.values.get(&key).copied()
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.resolve(name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SizeScale {
    values: HashMap<SmolStr, f32>,
}

impl SizeScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<SmolStr>, value: f32) -> &mut Self {
        self.values.insert(name.into(), value);
        self
    }

    pub fn resolve(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.values.get(&key).copied()
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.resolve(name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RadiusScale {
    values: HashMap<SmolStr, f32>,
}

impl RadiusScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<SmolStr>, value: f32) -> &mut Self {
        self.values.insert(name.into(), value);
        self
    }

    pub fn resolve(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.values.get(&key).copied()
    }

    pub fn get(&self, name: &str) -> Option<f32> {
        self.resolve(name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TypographyScale {
    font_families: HashMap<SmolStr, SmolStr>,
    font_sizes: HashMap<SmolStr, f32>,
    font_weights: HashMap<SmolStr, f32>,
    line_heights: HashMap<SmolStr, f32>,
    letter_spacings: HashMap<SmolStr, f32>,
}

impl TypographyScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font_family(
        &mut self,
        name: impl Into<SmolStr>,
        family: impl Into<SmolStr>,
    ) -> &mut Self {
        self.font_families.insert(name.into(), family.into());
        self
    }

    pub fn font_size(&mut self, name: impl Into<SmolStr>, size: f32) -> &mut Self {
        self.font_sizes.insert(name.into(), size);
        self
    }

    pub fn font_weight(&mut self, name: impl Into<SmolStr>, weight: f32) -> &mut Self {
        self.font_weights.insert(name.into(), weight);
        self
    }

    pub fn line_height(&mut self, name: impl Into<SmolStr>, height: f32) -> &mut Self {
        self.line_heights.insert(name.into(), height);
        self
    }

    pub fn letter_spacing(&mut self, name: impl Into<SmolStr>, spacing: f32) -> &mut Self {
        self.letter_spacings.insert(name.into(), spacing);
        self
    }

    pub fn get_font_family(&self, name: &str) -> Option<&SmolStr> {
        let key: SmolStr = name.into();
        self.font_families.get(&key)
    }

    pub fn get_font_size(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.font_sizes.get(&key).copied()
    }

    pub fn get_font_weight(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.font_weights.get(&key).copied()
    }

    pub fn get_line_height(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.line_heights.get(&key).copied()
    }

    pub fn get_letter_spacing(&self, name: &str) -> Option<f32> {
        let key: SmolStr = name.into();
        self.letter_spacings.get(&key).copied()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShadowScale {
    values: HashMap<SmolStr, Shadow>,
}

impl ShadowScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<SmolStr>, shadow: Shadow) -> &mut Self {
        self.values.insert(name.into(), shadow);
        self
    }

    pub fn resolve(&self, name: &str) -> Option<Shadow> {
        let key: SmolStr = name.into();
        self.values.get(&key).cloned()
    }

    pub fn get(&self, name: &str) -> Option<Shadow> {
        self.resolve(name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ComponentThemeRegistry {
    recipes: HashMap<RecipeKey, Arc<ComponentRecipe>>,
}

impl ComponentThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, key: RecipeKey, recipe: ComponentRecipe) -> &mut Self {
        self.recipes.insert(key, Arc::new(recipe));
        self
    }

    pub fn get(&self, key: &RecipeKey) -> Option<&Arc<ComponentRecipe>> {
        self.recipes.get(key)
    }

    pub fn get_recipe(
        &self,
        type_name: &str,
        variant: Option<&str>,
    ) -> Option<&Arc<ComponentRecipe>> {
        let key = RecipeKey::new(type_name, variant);
        self.get(&key)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TokenRegistry {
    colors: ColorScale,
    space: SpaceScale,
    sizes: SizeScale,
    radii: RadiusScale,
    typography: TypographyScale,
    shadows: ShadowScale,
}

impl TokenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn colors(&self) -> &ColorScale {
        &self.colors
    }

    pub fn space(&self) -> &SpaceScale {
        &self.space
    }

    pub fn sizes(&self) -> &SizeScale {
        &self.sizes
    }

    pub fn radii(&self) -> &RadiusScale {
        &self.radii
    }

    pub fn typography(&self) -> &TypographyScale {
        &self.typography
    }

    pub fn shadows(&self) -> &ShadowScale {
        &self.shadows
    }

    pub fn resolve(
        &self,
        scale: TokenScale,
        name: &str,
    ) -> Option<crate::theme::provider::TokenValue> {
        match scale {
            TokenScale::Color => self
                .colors
                .get(name)
                .map(|c| crate::theme::provider::TokenValue::Color(c)),
            TokenScale::Space => self
                .space
                .get(name)
                .map(|s| crate::theme::provider::TokenValue::F32(s)),
            TokenScale::Size => self
                .sizes
                .get(name)
                .map(|s| crate::theme::provider::TokenValue::F32(s)),
            TokenScale::Radius => self
                .radii
                .get(name)
                .map(|r| crate::theme::provider::TokenValue::F32(r)),
            TokenScale::FontSize => self
                .typography
                .get_font_size(name)
                .map(|s| crate::theme::provider::TokenValue::F32(s)),
            TokenScale::FontWeight => self
                .typography
                .get_font_weight(name)
                .map(|w| crate::theme::provider::TokenValue::F32(w)),
            TokenScale::FontFamily => self
                .typography
                .get_font_family(name)
                .map(|f| crate::theme::provider::TokenValue::String(f.to_string())),
            TokenScale::LineHeight => self
                .typography
                .get_line_height(name)
                .map(|h| crate::theme::provider::TokenValue::F32(h)),
            TokenScale::LetterSpacing => self
                .typography
                .get_letter_spacing(name)
                .map(|s| crate::theme::provider::TokenValue::F32(s)),
            TokenScale::Shadow => self
                .shadows
                .get(name)
                .map(|s| crate::theme::provider::TokenValue::Shadow(s)),
            TokenScale::Duration => None,
        }
    }
}
