use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use smol_str::SmolStr;

use crate::Style;
use crate::recipe::{ComponentRecipe, VariantMap};
use crate::state::WidgetState;

#[derive(Clone, Debug)]
pub struct ControlTheme {
    id: SmolStr,
    target_type: SmolStr,
    based_on: Option<Arc<ControlTheme>>,
    recipe: Arc<ComponentRecipe>,
}

impl ControlTheme {
    pub fn new(
        id: impl Into<SmolStr>,
        target_type: impl Into<SmolStr>,
        recipe: ComponentRecipe,
    ) -> Self {
        Self {
            id: id.into(),
            target_type: target_type.into(),
            based_on: None,
            recipe: Arc::new(recipe),
        }
    }

    pub fn based_on(mut self, parent: Arc<ControlTheme>) -> Self {
        self.based_on = Some(parent);
        self
    }

    pub fn id(&self) -> &SmolStr {
        &self.id
    }

    pub fn target_type(&self) -> &SmolStr {
        &self.target_type
    }

    pub fn recipe(&self) -> &Arc<ComponentRecipe> {
        &self.recipe
    }

    pub fn resolve_style(&self, variants: &VariantMap, state: &WidgetState) -> Style {
        let mut style = if let Some(parent) = &self.based_on {
            parent.resolve_style(variants, state)
        } else {
            Style::new()
        };
        style.merge(self.recipe.base_style());
        style.merge(&self.recipe.resolve_variants(variants, state));
        style
    }
}

#[derive(Default)]
pub struct ControlThemeRegistry {
    themes: RwLock<HashMap<SmolStr, Arc<ControlTheme>>>,
}

impl ControlThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, theme: ControlTheme) -> Arc<ControlTheme> {
        let theme = Arc::new(theme);
        self.themes
            .write()
            .expect("control theme registry poisoned")
            .insert(theme.id().clone(), theme.clone());
        theme
    }

    pub fn get(&self, id: &str) -> Option<Arc<ControlTheme>> {
        self.themes
            .read()
            .expect("control theme registry poisoned")
            .get(id)
            .cloned()
    }

    pub fn get_by_target_type(&self, target_type: &str) -> Vec<Arc<ControlTheme>> {
        self.themes
            .read()
            .expect("control theme registry poisoned")
            .values()
            .filter(|theme| theme.target_type() == target_type)
            .cloned()
            .collect()
    }
}
