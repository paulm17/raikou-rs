//! Inherited property handling for the styling system.
//!
//! Certain properties like text color, font size, and cursor propagate
//! down the widget tree unless explicitly overridden. This module provides
//! the infrastructure for tracking and resolving inherited properties.

use std::collections::HashMap;

use crate::property::PropertyId;

#[derive(Clone, Debug, Default)]
pub struct InheritedStyles {
    values: HashMap<PropertyId, InheritedValue>,
}

#[derive(Clone, Debug)]
pub enum InheritedValue {
    Color(raikou_core::Color),
    F32(f32),
    String(String),
}

impl InheritedStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_color(&mut self, property: PropertyId, value: raikou_core::Color) {
        self.values.insert(property, InheritedValue::Color(value));
    }

    pub fn set_f32(&mut self, property: PropertyId, value: f32) {
        self.values.insert(property, InheritedValue::F32(value));
    }

    pub fn set_string(&mut self, property: PropertyId, value: String) {
        self.values.insert(property, InheritedValue::String(value));
    }

    pub fn get_color(&self, property: &PropertyId) -> Option<raikou_core::Color> {
        self.values.get(property).and_then(|v| match v {
            InheritedValue::Color(c) => Some(*c),
            _ => None,
        })
    }

    pub fn get_f32(&self, property: &PropertyId) -> Option<f32> {
        self.values.get(property).and_then(|v| match v {
            InheritedValue::F32(f) => Some(*f),
            _ => None,
        })
    }

    pub fn get_string(&self, property: &PropertyId) -> Option<String> {
        self.values.get(property).and_then(|v| match v {
            InheritedValue::String(s) => Some(s.clone()),
            _ => None,
        })
    }

    pub fn remove(&mut self, property: &PropertyId) -> Option<InheritedValue> {
        self.values.remove(property)
    }

    pub fn get_all(&self) -> &HashMap<PropertyId, InheritedValue> {
        &self.values
    }

    pub fn extend(&mut self, other: InheritedStyles) {
        for (property, value) in other.values {
            self.values.insert(property, value);
        }
    }

    pub fn clear(&mut self) {
        self.values.clear()
    }
}

pub trait Inheritable: Clone + 'static {
    const PROPERTY_ID: PropertyId;

    fn default_value() -> Self;
    fn merge(parent: &Option<Self>, child: &Option<Self>) -> Self;
}

pub struct InheritedProperty<P: Inheritable> {
    value: Option<P>,
}

impl<P: Inheritable> InheritedProperty<P> {
    pub fn new(value: Option<P>) -> Self {
        Self { value }
    }

    pub fn get(&self) -> Option<&P> {
        self.value.as_ref()
    }

    pub fn resolve(&self, parent_value: &Option<P>) -> P {
        match &self.value {
            Some(v) => v.clone(),
            None => parent_value.clone().unwrap_or_else(P::default_value),
        }
    }

    pub fn set(&mut self, value: P) {
        self.value = Some(value);
    }

    pub fn unset(&mut self) {
        self.value = None;
    }

    pub fn is_set(&self) -> bool {
        self.value.is_some()
    }
}

impl<P: Inheritable> Default for InheritedProperty<P> {
    fn default() -> Self {
        Self::new(None)
    }
}

pub fn is_inheritable(property: PropertyId) -> bool {
    matches!(
        property,
        crate::property::text_style::COLOR
            | crate::property::text_style::FONT_FAMILY
            | crate::property::text_style::FONT_SIZE
            | crate::property::text_style::FONT_WEIGHT
            | crate::property::text_style::LINE_HEIGHT
            | crate::property::text_style::LETTER_SPACING
            | crate::property::interaction_style::CURSOR
    )
}

pub fn get_inheritable_properties() -> Vec<PropertyId> {
    vec![
        crate::property::text_style::COLOR,
        crate::property::text_style::FONT_FAMILY,
        crate::property::text_style::FONT_SIZE,
        crate::property::text_style::FONT_WEIGHT,
        crate::property::text_style::LINE_HEIGHT,
        crate::property::text_style::LETTER_SPACING,
        crate::property::interaction_style::CURSOR,
    ]
}
