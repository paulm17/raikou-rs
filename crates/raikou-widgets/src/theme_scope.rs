//! The ThemeScope component — a pass-through container that scopes a theme
//! override to its subtree.
//!
//! In this fyrox-backed port, component styles are resolved at `build` time
//! from the active [`crate::BuildCx`] theme, so a scoped theme cannot (yet)
//! re-resolve already-built descendants. `ThemeScope` therefore acts as a
//! structural pass-through: it returns its child's handle and retains the
//! scoped theme for reference.

use fyrox::core::pool::Handle;
use fyrox::gui::UiNode;

use raikou_style::Theme;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};

/// Builder for a [`crate::ThemeScope`] component.
pub struct ThemeScope {
    child: Handle<UiNode>,
    theme: Option<Theme>,
}

impl ThemeScope {
    /// Creates a new theme scope wrapping a built child.
    pub fn new(child: impl Into<Handle<UiNode>>) -> Self {
        Self {
            child: child.into(),
            theme: None,
        }
    }

    /// Records a theme override for the subtree.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the scoped theme override, if any.
    pub fn scoped_theme(&self) -> Option<&Theme> {
        self.theme.as_ref()
    }

    /// Builds the theme scope (a pass-through of its child).
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let component = Component {
            handle: self.child,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built theme scope.
pub type ThemeScopeHandle = Handle<UiNode>;
