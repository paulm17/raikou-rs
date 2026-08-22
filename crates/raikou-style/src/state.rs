//! Widget state definitions for state-driven styling.
//!
//! This module provides the `WidgetState` type that tracks interactive states
//! like hover, pressed, focused, and disabled. These states drive state-style
//! resolution in the styling system.

use std::collections::HashSet;

use crate::property::{box_style, text_style};
use crate::style::{Style, StylePrecedence, StyleSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pseudoclass {
    Idle,
    Hovered,
    Pressed,
    Focused,
    FocusVisible,
    Disabled,
    Loading,
    Checked,
    Selected,
    Expanded,
    Collapsed,
}

impl Pseudoclass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => ":idle",
            Self::Hovered => ":hovered",
            Self::Pressed => ":pressed",
            Self::Focused => ":focused",
            Self::FocusVisible => ":focus-visible",
            Self::Disabled => ":disabled",
            Self::Loading => ":loading",
            Self::Checked => ":checked",
            Self::Selected => ":selected",
            Self::Expanded => ":expanded",
            Self::Collapsed => ":collapsed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatePriority {
    Idle = 0,
    Focused = 1,
    Hovered = 2,
    Pressed = 3,
    Loading = 4,
    Disabled = 5,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateTransition {
    pub entered: Vec<Pseudoclass>,
    pub exited: Vec<Pseudoclass>,
}

#[derive(Clone, Debug, Default)]
pub struct StateTracker {
    current: WidgetState,
}

impl StateTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> WidgetState {
        self.current
    }

    pub fn update(&mut self, next: WidgetState) -> StateTransition {
        let previous = self.current.pseudoclasses();
        let current = next.pseudoclasses();
        self.current = next;

        let prev: HashSet<_> = previous.iter().copied().collect();
        let curr: HashSet<_> = current.iter().copied().collect();

        let entered = current
            .into_iter()
            .filter(|class| !prev.contains(class))
            .collect();
        let exited = previous
            .into_iter()
            .filter(|class| !curr.contains(class))
            .collect();

        StateTransition { entered, exited }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WidgetState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focused_ring: bool,
    pub disabled: bool,
    pub selected: bool,
    pub checked: bool,
    pub expanded: bool,
    pub collapsed: bool,
    pub loading: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hovered(mut self) -> Self {
        self.hovered = true;
        self
    }

    pub fn pressed(mut self) -> Self {
        self.pressed = true;
        self
    }

    pub fn focused(mut self) -> Self {
        self.focused = true;
        self
    }

    pub fn focused_ring(mut self) -> Self {
        self.focused_ring = true;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    pub fn expanded(mut self) -> Self {
        self.expanded = true;
        self
    }

    pub fn collapsed(mut self) -> Self {
        self.collapsed = true;
        self
    }

    pub fn loading(mut self) -> Self {
        self.loading = true;
        self
    }

    pub fn is_interactive(&self) -> bool {
        !self.disabled && !self.loading
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered && !self.disabled && !self.loading
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed && !self.disabled && !self.loading
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn dominant_state(&self) -> StatePriority {
        if self.disabled {
            StatePriority::Disabled
        } else if self.loading {
            StatePriority::Loading
        } else if self.pressed {
            StatePriority::Pressed
        } else if self.hovered {
            StatePriority::Hovered
        } else if self.focused {
            StatePriority::Focused
        } else {
            StatePriority::Idle
        }
    }

    pub fn is_state_active(&self, state: StatePriority) -> bool {
        match state {
            StatePriority::Disabled => self.disabled,
            StatePriority::Loading => self.loading && !self.disabled,
            StatePriority::Pressed => self.pressed && !self.disabled && !self.loading,
            StatePriority::Hovered => {
                self.hovered && !self.disabled && !self.loading && !self.pressed
            }
            StatePriority::Focused => self.focused && !self.disabled && !self.loading,
            StatePriority::Idle => {
                !self.disabled && !self.loading && !self.pressed && !self.hovered && !self.focused
            }
        }
    }

    pub fn pseudoclasses(&self) -> Vec<Pseudoclass> {
        let mut classes = Vec::new();

        if self.disabled {
            classes.push(Pseudoclass::Disabled);
        } else if self.loading {
            classes.push(Pseudoclass::Loading);
        } else if self.pressed {
            classes.push(Pseudoclass::Pressed);
        } else if self.hovered {
            classes.push(Pseudoclass::Hovered);
        } else {
            classes.push(Pseudoclass::Idle);
        }

        if self.focused {
            classes.push(Pseudoclass::Focused);
            if self.focused_ring {
                classes.push(Pseudoclass::FocusVisible);
            }
        }
        if self.checked {
            classes.push(Pseudoclass::Checked);
        }
        if self.selected {
            classes.push(Pseudoclass::Selected);
        }
        if self.expanded {
            classes.push(Pseudoclass::Expanded);
        }
        if self.collapsed {
            classes.push(Pseudoclass::Collapsed);
        }

        classes
    }

    pub fn active_pseudoclasses(&self) -> Vec<&'static str> {
        self.pseudoclasses()
            .into_iter()
            .map(Pseudoclass::as_str)
            .collect()
    }

    pub fn to_style(&self) -> Style {
        let mut style = Style::new();

        if self.hovered && !self.disabled {
            style.set_f32(
                box_style::OPACITY,
                0.9,
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
        }

        if self.pressed && !self.disabled {
            style.set_f32(
                box_style::OPACITY,
                0.8,
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
        }

        if self.focused {
            style.set_color(
                box_style::BORDER_COLOR,
                raikou_core::Color::new(0.0, 0.4, 0.8, 1.0),
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
            style.set_f32(
                box_style::BORDER_WIDTH,
                2.0,
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
        }

        if self.disabled {
            style.set_f32(
                box_style::OPACITY,
                0.5,
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
            style.set_color(
                text_style::COLOR,
                raikou_core::Color::new(0.5, 0.5, 0.5, 1.0),
                StylePrecedence::StateStyle,
                StyleSource::State,
            );
        }

        style
    }

    pub fn matches_state(&self, other: &WidgetState) -> bool {
        if self.hovered != other.hovered && (self.hovered || other.hovered) {
            return false;
        }
        if self.pressed != other.pressed && (self.pressed || other.pressed) {
            return false;
        }
        if self.focused != other.focused && (self.focused || other.focused) {
            return false;
        }
        if self.focused_ring != other.focused_ring && (self.focused_ring || other.focused_ring) {
            return false;
        }
        if self.disabled != other.disabled && (self.disabled || other.disabled) {
            return false;
        }
        if self.selected != other.selected && (self.selected || other.selected) {
            return false;
        }
        if self.checked != other.checked && (self.checked || other.checked) {
            return false;
        }
        if self.expanded != other.expanded && (self.expanded || other.expanded) {
            return false;
        }
        if self.collapsed != other.collapsed && (self.collapsed || other.collapsed) {
            return false;
        }
        if self.loading != other.loading && (self.loading || other.loading) {
            return false;
        }
        true
    }
}

#[derive(Clone, Debug, Default)]
pub struct StateStyles {
    hover: Option<Style>,
    pressed: Option<Style>,
    focused: Option<Style>,
    focused_ring: Option<Style>,
    disabled: Option<Style>,
    selected: Option<Style>,
    checked: Option<Style>,
    expanded: Option<Style>,
    loading: Option<Style>,
}

impl StateStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hover(mut self, style: Style) -> Self {
        self.hover = Some(style);
        self
    }

    pub fn pressed(mut self, style: Style) -> Self {
        self.pressed = Some(style);
        self
    }

    pub fn focused(mut self, style: Style) -> Self {
        self.focused = Some(style);
        self
    }

    pub fn focused_ring(mut self, style: Style) -> Self {
        self.focused_ring = Some(style);
        self
    }

    pub fn disabled(mut self, style: Style) -> Self {
        self.disabled = Some(style);
        self
    }

    pub fn selected(mut self, style: Style) -> Self {
        self.selected = Some(style);
        self
    }

    pub fn checked(mut self, style: Style) -> Self {
        self.checked = Some(style);
        self
    }

    pub fn expanded(mut self, style: Style) -> Self {
        self.expanded = Some(style);
        self
    }

    pub fn loading(mut self, style: Style) -> Self {
        self.loading = Some(style);
        self
    }

    pub fn get_style(&self, state: &WidgetState) -> Style {
        let mut result = Style::new();

        if state.hovered {
            if let Some(ref s) = self.hover {
                result.extend(s.clone());
            }
        }

        if state.pressed {
            if let Some(ref s) = self.pressed {
                result.extend(s.clone());
            }
        }

        if state.focused {
            if let Some(ref s) = self.focused {
                result.extend(s.clone());
            }
        }

        if state.focused_ring {
            if let Some(ref s) = self.focused_ring {
                result.extend(s.clone());
            }
        }

        if state.disabled {
            if let Some(ref s) = self.disabled {
                result.extend(s.clone());
            }
        }

        if state.selected {
            if let Some(ref s) = self.selected {
                result.extend(s.clone());
            }
        }

        if state.checked {
            if let Some(ref s) = self.checked {
                result.extend(s.clone());
            }
        }

        if state.expanded {
            if let Some(ref s) = self.expanded {
                result.extend(s.clone());
            }
        }

        if state.loading {
            if let Some(ref s) = self.loading {
                result.extend(s.clone());
            }
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.hover.is_none()
            && self.pressed.is_none()
            && self.focused.is_none()
            && self.focused_ring.is_none()
            && self.disabled.is_none()
            && self.selected.is_none()
            && self.checked.is_none()
            && self.expanded.is_none()
            && self.loading.is_none()
    }
}

pub fn hover() -> WidgetState {
    WidgetState::new().hovered()
}

pub fn pressed() -> WidgetState {
    WidgetState::new().pressed()
}

pub fn focused() -> WidgetState {
    WidgetState::new().focused()
}

pub fn focused_ring() -> WidgetState {
    WidgetState::new().focused_ring()
}

pub fn disabled() -> WidgetState {
    WidgetState::new().disabled()
}

pub fn selected() -> WidgetState {
    WidgetState::new().selected()
}

pub fn checked() -> WidgetState {
    WidgetState::new().checked()
}

pub fn expanded() -> WidgetState {
    WidgetState::new().expanded()
}

pub fn loading() -> WidgetState {
    WidgetState::new().loading()
}

#[cfg(test)]
mod tests {
    use super::{focused, hover, loading, Pseudoclass, StatePriority, StateTracker, WidgetState};

    #[test]
    fn dominant_state_prefers_disabled_over_other_interaction_states() {
        let state = WidgetState::new()
            .hovered()
            .pressed()
            .focused()
            .loading()
            .disabled();
        assert_eq!(state.dominant_state(), StatePriority::Disabled);
    }

    #[test]
    fn pseudoclasses_include_idle_and_focus_visible() {
        let state = hover().focused_ring().focused();
        assert_eq!(
            state.pseudoclasses(),
            vec![
                Pseudoclass::Hovered,
                Pseudoclass::Focused,
                Pseudoclass::FocusVisible
            ]
        );
    }

    #[test]
    fn tracker_reports_entered_and_exited_pseudoclasses() {
        let mut tracker = StateTracker::new();
        let first = tracker.update(focused());
        assert!(first.entered.contains(&Pseudoclass::Focused));

        let second = tracker.update(loading());
        assert!(second.entered.contains(&Pseudoclass::Loading));
        assert!(second.exited.contains(&Pseudoclass::Focused));
    }
}
