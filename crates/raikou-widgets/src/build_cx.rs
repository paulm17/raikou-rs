//! Build session context passed to every component's terminal `build` call.

use fyrox::gui::{BuildContext, UserInterface};

use raikou_style::Theme;

use crate::component::Component;
use crate::registry::ComponentRegistry;

/// The session used to construct components.
///
/// Carries the UI being built into, the active theme, and the registry that
/// the built component's handlers are registered into.
pub struct BuildCx<'a> {
    /// The user interface being built into.
    pub ui: &'a mut UserInterface,
    /// The active theme used to resolve component styles.
    pub theme: &'a Theme,
    /// Registry that receives the handlers of every built component.
    pub registry: &'a mut ComponentRegistry,
}

impl<'a> BuildCx<'a> {
    /// Creates a new build session.
    pub fn new(
        ui: &'a mut UserInterface,
        theme: &'a Theme,
        registry: &'a mut ComponentRegistry,
    ) -> Self {
        Self {
            ui,
            theme,
            registry,
        }
    }

    /// Returns the user interface being built into.
    pub fn ui(&mut self) -> &mut UserInterface {
        self.ui
    }

    /// Returns the active theme.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Returns a fyrox build context over the user interface.
    pub fn ctx(&mut self) -> BuildContext<'_> {
        self.ui.build_ctx()
    }

    /// Registers a built component's handlers in the registry.
    pub fn register(&mut self, component: &Component) {
        self.registry.register(component);
    }

    /// Registers a built component's handlers as a global listener that
    /// observes every message in the UI (see
    /// [`ComponentRegistry::register_global`]).
    pub fn register_global(&mut self, component: &Component) {
        self.registry.register_global(component);
    }
}
