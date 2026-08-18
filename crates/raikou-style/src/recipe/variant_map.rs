//! Variant map for tracking selected component variants.

use smol_str::SmolStr;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct VariantMap {
    variants: HashMap<SmolStr, SmolStr>,
}

impl VariantMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, group: impl Into<SmolStr>, value: impl Into<SmolStr>) -> &mut Self {
        self.variants.insert(group.into(), value.into());
        self
    }

    pub fn get(&self, group: &str) -> Option<&SmolStr> {
        self.variants.get(group)
    }

    pub fn remove(&mut self, group: &str) -> Option<SmolStr> {
        self.variants.remove(group)
    }

    pub fn contains(&self, group: &str) -> bool {
        self.variants.contains_key(group)
    }

    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }

    pub fn len(&self) -> usize {
        self.variants.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SmolStr, &SmolStr)> {
        self.variants.iter()
    }

    pub fn matches(&self, other: &VariantMap) -> bool {
        for (group, value) in &self.variants {
            if let Some(other_value) = other.get(group) {
                if value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn extend(&mut self, other: VariantMap) {
        self.variants.extend(other.variants);
    }

    pub fn clear(&mut self) {
        self.variants.clear();
    }
}

impl From<VariantMap> for HashMap<SmolStr, SmolStr> {
    fn from(map: VariantMap) -> Self {
        map.variants
    }
}

impl From<HashMap<SmolStr, SmolStr>> for VariantMap {
    fn from(map: HashMap<SmolStr, SmolStr>) -> Self {
        Self { variants: map }
    }
}
