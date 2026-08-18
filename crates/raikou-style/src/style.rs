//! Style value storage and manipulation.
//!
//! This module provides the `Style` and `StyleBlock` types that store
//! property values with their precedence metadata.

use std::collections::HashMap;

use crate::property::{PropertyId, PropertyRegistry};
use crate::style_value::Token;
use smol_str::SmolStr;

#[derive(Clone, Debug, Default)]
pub struct Style {
    values: HashMap<PropertyId, StyleValueEntry>,
}

#[derive(Clone, Debug)]
pub struct StyledValue {
    pub value: StyleValueHolder,
    pub precedence: StylePrecedence,
    pub source: StyleSource,
}

#[derive(Clone, Debug)]
pub enum StyleValueEntry {
    Value(StyledValue),
    Token(Token<SmolStr>),
    None,
    Default,
}

#[derive(Clone, Debug)]
pub enum StyleValueHolder {
    Color(raikou_core::Color),
    F32(f32),
    String(String),
    Shadow(crate::property::Shadow),
    Length(raikou_core::Length),
    Padding(raikou_core::Padding),
    Margin(raikou_core::Margin),
    Radius(raikou_core::Radius),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StylePrecedence {
    Animation = 1,
    LocalOverride = 2,
    StateStyle = 3,
    Variant = 4,
    BaseRecipe = 5,
    Inherited = 6,
    PropertyDefault = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleSource {
    Local,
    State,
    Variant,
    Recipe,
    Inheritance,
    Default,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_color(
        &mut self,
        property: PropertyId,
        value: raikou_core::Color,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::Color(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_f32(
        &mut self,
        property: PropertyId,
        value: f32,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::F32(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_string(
        &mut self,
        property: PropertyId,
        value: String,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::String(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_length(
        &mut self,
        property: PropertyId,
        value: raikou_core::Length,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::Length(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_padding(
        &mut self,
        property: PropertyId,
        value: raikou_core::Padding,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::Padding(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_margin(
        &mut self,
        property: PropertyId,
        value: raikou_core::Margin,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::Margin(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_radius(
        &mut self,
        property: PropertyId,
        value: raikou_core::Radius,
        precedence: StylePrecedence,
        source: StyleSource,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Value(StyledValue {
                value: StyleValueHolder::Radius(value),
                precedence,
                source,
            }),
        );
    }

    pub fn set_token(
        &mut self,
        property: PropertyId,
        scale: crate::style_value::TokenScale,
        name: impl Into<smol_str::SmolStr>,
    ) {
        self.values.insert(
            property,
            StyleValueEntry::Token(Token::new(scale, name.into())),
        );
    }

    pub fn get(&self, property: &PropertyId) -> Option<&StyleValueEntry> {
        self.values.get(property)
    }

    pub fn remove(&mut self, property: &PropertyId) -> Option<StyleValueEntry> {
        self.values.remove(property)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PropertyId, &StyleValueEntry)> {
        self.values.iter()
    }

    pub fn extend(&mut self, other: Style) {
        for (property, entry) in other.values {
            self.values.insert(property, entry);
        }
    }

    pub fn merge(&mut self, other: &Style) {
        for (property, entry) in &other.values {
            if let Some(existing) = self.values.get(property) {
                if entry.precedence() <= existing.precedence() {
                    self.values.insert(*property, entry.clone());
                }
            } else {
                self.values.insert(*property, entry.clone());
            }
        }
    }
}

impl StyleValueEntry {
    pub fn precedence(&self) -> StylePrecedence {
        match self {
            Self::Value(v) => v.precedence,
            Self::Token(_) => StylePrecedence::PropertyDefault,
            Self::None => StylePrecedence::PropertyDefault,
            Self::Default => StylePrecedence::PropertyDefault,
        }
    }

    pub fn source(&self) -> StyleSource {
        match self {
            Self::Value(v) => v.source,
            Self::Token(_) => StyleSource::Default,
            Self::None => StyleSource::Default,
            Self::Default => StyleSource::Default,
        }
    }

    pub fn as_value(&self) -> Option<&StyleValueHolder> {
        match self {
            Self::Value(v) => Some(&v.value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleBlock {
    styles: Vec<Style>,
}

impl StyleBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_style(&mut self, style: Style) {
        self.styles.push(style);
    }

    pub fn styles(&self) -> &[Style] {
        &self.styles
    }

    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.styles.len()
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedStyle {
    pub property: PropertyId,
    pub precedence: StylePrecedence,
    pub source: StyleSource,
}

#[derive(Clone, Debug, Default)]
pub struct StyleResolver {
    #[allow(dead_code)]
    registry: PropertyRegistry,
}

impl StyleResolver {
    pub fn new(registry: PropertyRegistry) -> Self {
        Self { registry }
    }
}

pub struct ResolvedValue<T: Clone> {
    pub value: T,
    pub precedence: StylePrecedence,
    pub source: StyleSource,
}
