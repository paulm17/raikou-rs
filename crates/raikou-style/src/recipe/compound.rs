//! Compound variant support for conditional style application.

use crate::recipe::VariantMap;
use crate::state::WidgetState;
use crate::style::Style;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub struct CompoundVariant {
    conditions: Vec<CompoundVariantCondition>,
    style: Style,
}

impl CompoundVariant {
    pub fn new(conditions: Vec<CompoundVariantCondition>, style: Style) -> Self {
        Self { conditions, style }
    }

    pub fn conditions(&self) -> &[CompoundVariantCondition] {
        &self.conditions
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn matches(&self, variants: &VariantMap, state: &WidgetState) -> bool {
        for condition in &self.conditions {
            if !condition.matches(variants, state) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub enum CompoundVariantCondition {
    Variant {
        group: SmolStr,
        value: SmolStr,
    },
    State {
        state: WidgetState,
    },
    NotBox {
        inner: Box<CompoundVariantCondition>,
    },
    AndBox {
        left: Box<CompoundVariantCondition>,
        right: Box<CompoundVariantCondition>,
    },
    OrBox {
        left: Box<CompoundVariantCondition>,
        right: Box<CompoundVariantCondition>,
    },
}

impl CompoundVariantCondition {
    pub fn variant(group: &str, value: &str) -> Self {
        Self::Variant {
            group: group.into(),
            value: value.into(),
        }
    }

    pub fn state(state: WidgetState) -> Self {
        Self::State { state }
    }

    pub fn hovered() -> Self {
        Self::State {
            state: WidgetState::new().hovered(),
        }
    }

    pub fn pressed() -> Self {
        Self::State {
            state: WidgetState::new().pressed(),
        }
    }

    pub fn focused() -> Self {
        Self::State {
            state: WidgetState::new().focused(),
        }
    }

    pub fn disabled() -> Self {
        Self::State {
            state: WidgetState::new().disabled(),
        }
    }

    pub fn checked() -> Self {
        Self::State {
            state: WidgetState::new().checked(),
        }
    }

    pub fn selected() -> Self {
        Self::State {
            state: WidgetState::new().selected(),
        }
    }

    pub fn not(inner: CompoundVariantCondition) -> Self {
        Self::NotBox {
            inner: Box::new(inner),
        }
    }

    pub fn and(left: CompoundVariantCondition, right: CompoundVariantCondition) -> Self {
        Self::AndBox {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn or(left: CompoundVariantCondition, right: CompoundVariantCondition) -> Self {
        Self::OrBox {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn matches(&self, variants: &VariantMap, state: &WidgetState) -> bool {
        match self {
            Self::Variant { group, value } => {
                variants.get(group).map(|v| v == value).unwrap_or(false)
            }
            Self::State { state: cond_state } => state.matches_state(cond_state),
            Self::NotBox { inner } => !inner.matches(variants, state),
            Self::AndBox { left, right } => {
                left.matches(variants, state) && right.matches(variants, state)
            }
            Self::OrBox { left, right } => {
                left.matches(variants, state) || right.matches(variants, state)
            }
        }
    }
}

pub struct CompoundVariantBuilder {
    conditions: Vec<CompoundVariantCondition>,
}

impl CompoundVariantBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn when_variant(mut self, group: &str, value: &str) -> Self {
        self.conditions
            .push(CompoundVariantCondition::variant(group, value));
        self
    }

    pub fn when_state(mut self, state: WidgetState) -> Self {
        self.conditions.push(CompoundVariantCondition::state(state));
        self
    }

    pub fn when_hovered(self) -> Self {
        self.when_state(WidgetState::new().hovered())
    }

    pub fn when_pressed(self) -> Self {
        self.when_state(WidgetState::new().pressed())
    }

    pub fn when_focused(self) -> Self {
        self.when_state(WidgetState::new().focused())
    }

    pub fn when_disabled(self) -> Self {
        self.when_state(WidgetState::new().disabled())
    }

    pub fn and(mut self, condition: CompoundVariantCondition) -> Self {
        if let Some(last) = self.conditions.pop() {
            self.conditions
                .push(CompoundVariantCondition::and(last, condition));
        } else {
            self.conditions.push(condition);
        }
        self
    }

    pub fn or(mut self, condition: CompoundVariantCondition) -> Self {
        if let Some(last) = self.conditions.pop() {
            self.conditions
                .push(CompoundVariantCondition::or(last, condition));
        } else {
            self.conditions.push(condition);
        }
        self
    }

    pub fn style(self, style: Style) -> CompoundVariant {
        CompoundVariant::new(self.conditions, style)
    }
}

impl Default for CompoundVariantBuilder {
    fn default() -> Self {
        Self::new()
    }
}
