//! Component recipe implementation.

use crate::recipe::{CompoundVariant, RecipeKey, StateStyleMap, VariantMap};
use crate::state::WidgetState;
use crate::style::Style;
use smol_str::SmolStr;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ComponentRecipe {
    pub key: RecipeKey,
    pub base_style: Style,
    pub variants: HashMap<SmolStr, Style>,
    pub compound_variants: Vec<CompoundVariant>,
    pub default_variants: VariantMap,
    pub state_styles: StateStyleMap,
}

impl ComponentRecipe {
    pub fn key(&self) -> &RecipeKey {
        &self.key
    }

    pub fn base_style(&self) -> &Style {
        &self.base_style
    }

    pub fn variants(&self) -> &HashMap<SmolStr, Style> {
        &self.variants
    }

    pub fn compound_variants(&self) -> &[CompoundVariant] {
        &self.compound_variants
    }

    pub fn default_variants(&self) -> &VariantMap {
        &self.default_variants
    }

    pub fn state_styles(&self) -> &StateStyleMap {
        &self.state_styles
    }

    pub fn resolve_variants(
        &self,
        selected_variants: &VariantMap,
        widget_state: &WidgetState,
    ) -> Style {
        let mut result = Style::new();

        let effective_variants = if selected_variants.is_empty() {
            &self.default_variants
        } else {
            selected_variants
        };

        for (group, value) in effective_variants.iter() {
            let key = SmolStr::from(format!("{group}:{value}"));
            if let Some(style) = self.variants.get(&key) {
                result.extend(style.clone());
            }
        }

        for compound in &self.compound_variants {
            if compound.matches(effective_variants, widget_state) {
                result.merge(&compound.style());
            }
        }

        let state_style = self.state_styles.get_style(widget_state);
        result.merge(&state_style);

        result
    }

    pub fn apply_state(&self, state: &WidgetState) -> Style {
        self.state_styles.get_style(state)
    }

    pub fn merge(&mut self, other: ComponentRecipe) {
        self.base_style.merge(&other.base_style);

        for (key, style) in other.variants {
            if let Some(existing) = self.variants.get(&key) {
                let mut merged = existing.clone();
                merged.merge(&style);
                self.variants.insert(key, merged);
            } else {
                self.variants.insert(key, style);
            }
        }

        self.compound_variants.extend(other.compound_variants);
        self.state_styles = other.state_styles;
    }
}

pub struct ComponentRecipeBuilder {
    component: SmolStr,
    base_style: Style,
    variants: HashMap<SmolStr, Style>,
    compound_variants: Vec<CompoundVariant>,
    default_variants: VariantMap,
    state_styles: StateStyleMap,
}

impl ComponentRecipeBuilder {
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

    pub fn base_style(mut self, style: Style) -> Self {
        self.base_style = style;
        self
    }

    pub fn variant(mut self, group: &str, name: &str, style: Style) -> Self {
        let key = SmolStr::from(format!("{}:{}", group, name));
        self.variants.insert(key, style);
        self
    }

    pub fn compound_variant(mut self, compound: CompoundVariant) -> Self {
        self.compound_variants.push(compound);
        self
    }

    pub fn default_variant(mut self, group: &str, value: &str) -> Self {
        self.default_variants.insert(group, value);
        self
    }

    pub fn state(mut self, widget_state: WidgetState, style: Style) -> Self {
        self.state_styles.insert(widget_state, style);
        self
    }

    pub fn build(self) -> ComponentRecipe {
        ComponentRecipe {
            key: RecipeKey::base(self.component),
            base_style: self.base_style,
            variants: self.variants,
            compound_variants: self.compound_variants,
            default_variants: self.default_variants,
            state_styles: self.state_styles,
        }
    }
}
