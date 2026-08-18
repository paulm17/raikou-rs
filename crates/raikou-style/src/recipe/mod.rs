//! Component recipe system for theming.
//!
//! This module provides the infrastructure for component-level theming
//! including base styles, named variants, compound variants, and state overlays.

pub mod component_recipe;
pub mod compound;
pub mod variant_map;

pub use component_recipe::{ComponentRecipe, ComponentRecipeBuilder};
pub use compound::{CompoundVariant, CompoundVariantCondition};
pub use variant_map::VariantMap;

use crate::state::WidgetState;
use crate::style::Style;
use smol_str::SmolStr;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecipeKey {
    component: SmolStr,
    variant: Option<SmolStr>,
}

impl RecipeKey {
    pub fn new(component: impl Into<SmolStr>, variant: Option<impl Into<SmolStr>>) -> Self {
        Self {
            component: component.into(),
            variant: variant.map(|v| v.into()),
        }
    }

    pub fn base(component: impl Into<SmolStr>) -> Self {
        Self::new(component, Option::<SmolStr>::None)
    }

    pub fn component(&self) -> &SmolStr {
        &self.component
    }
}

#[derive(Clone, Debug)]
pub struct VariantDefinition {
    name: SmolStr,
    style: Style,
}

impl VariantDefinition {
    pub fn new(name: impl Into<SmolStr>, style: Style) -> Self {
        Self {
            name: name.into(),
            style,
        }
    }

    pub fn name(&self) -> &SmolStr {
        &self.name
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
}

pub struct RecipeBuilder {
    component: SmolStr,
    base_style: Style,
    variants: HashMap<SmolStr, Style>,
    compound_variants: Vec<CompoundVariant>,
    default_variants: VariantMap,
    state_styles: StateStyleMap,
}

impl RecipeBuilder {
    pub fn new(component: impl Into<SmolStr>) -> Self {
        Self {
            component: component.into(),
            base_style: Style::new(),
            variants: HashMap::new(),
            compound_variants: Vec::new(),
            default_variants: VariantMap::new(),
            state_styles: StateStyleMap::new(),
        }
    }

    pub fn base(mut self, style: Style) -> Self {
        self.base_style = style;
        self
    }

    pub fn variant(mut self, variant_group: &str, variant_name: &str, style: Style) -> Self {
        let key = SmolStr::from(format!("{}:{}", variant_group, variant_name));
        self.variants.insert(key, style);
        self
    }

    pub fn compound(mut self, compound: CompoundVariant) -> Self {
        self.compound_variants.push(compound);
        self
    }

    pub fn defaults(mut self, defaults: VariantMap) -> Self {
        self.default_variants = defaults;
        self
    }

    pub fn state(mut self, state: WidgetState, style: Style) -> Self {
        self.state_styles.insert(state, style);
        self
    }

    pub fn build(self) -> ComponentRecipe {
        ComponentRecipe {
            key: RecipeKey::base(self.component.clone()),
            base_style: self.base_style,
            variants: self.variants,
            compound_variants: self.compound_variants,
            default_variants: self.default_variants,
            state_styles: self.state_styles,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StateStyleMap {
    states: Vec<(WidgetState, Style)>,
}

impl StateStyleMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, state: WidgetState, style: Style) {
        self.states.push((state, style));
    }

    pub fn get_style(&self, current_state: &WidgetState) -> Style {
        let mut result = Style::new();
        for (state, style) in &self.states {
            if current_state.matches_state(state) {
                result.merge(style);
            }
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

pub fn recipe_for<T: HasRecipe>() -> ComponentRecipe {
    T::recipe()
}

pub trait HasRecipe {
    fn recipe() -> ComponentRecipe;
}
