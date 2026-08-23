//! Shared harness for headless functional tests: builds a standalone
//! `UserInterface`, links built components to the root, and pumps the message
//! queue through the [`ComponentRegistry`] exactly like the app loop does.

use fyrox::core::algebra::Vector2;
use fyrox::gui::UiUpdateSwitches;
use fyrox::gui::UserInterface;
use raikou_style::Theme;
use raikou_widgets::{BuildCx, Component, ComponentRegistry};
use std::rc::Rc;

pub struct Harness {
    pub ui: UserInterface,
    pub theme: Theme,
    pub registry: ComponentRegistry,
}

#[allow(dead_code)]
impl Harness {
    pub fn new() -> Self {
        Self::new_with_theme(Theme::fluent_light(), false)
    }

    /// Same harness with an explicit theme + dark flag for the global style.
    pub fn new_with_theme(theme: Theme, dark: bool) -> Self {
        let mut ui = UserInterface::new(Vector2::new(800.0, 600.0));
        // Match the app: map the theme onto fyrox's global style so
        // natively-styled widgets pick up Fluent colors.
        use raikou_style::theme::fyrox_bridge::fluent_fyrox_style_resource;
        ui.set_style(fluent_fyrox_style_resource(&theme, dark));
        Self {
            ui,
            theme,
            registry: ComponentRegistry::default(),
        }
    }

    /// Builds a component and links it to the UI root.
    pub fn build<F>(&mut self, f: F) -> Component
    where
        F: FnOnce(&mut BuildCx) -> Component,
    {
        let component = {
            let mut cx = BuildCx::new(&mut self.ui, &self.theme, &mut self.registry);
            f(&mut cx)
        };
        let root = self.ui.root();
        let mut ctx = self.ui.build_ctx();
        ctx.link(component.handle, root);
        self.ui.update_layout(Vector2::new(800.0, 600.0));
        component
    }

    /// Drains the message queue, routing each message through the registry.
    pub fn pump(&mut self) {
        loop {
            let poll_result = self.ui.poll_message_queue();
            match poll_result.message {
                Some(message) => self.registry.dispatch(&mut self.ui, &message),
                None => break,
            }
        }
    }

    /// Runs one UI update tick (so native widgets can react to queued input
    /// messages) and then drains the queue through the registry.
    pub fn update_and_pump(&mut self) {
        self.ui.update(
            Vector2::new(800.0, 600.0),
            1.0 / 60.0,
            &UiUpdateSwitches::default(),
        );
        self.pump();
    }
}

/// Counts callback invocations. Some test binaries never construct it, but
/// the shared module is compiled per-binary, hence the allow.
#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct Counter(Rc<std::cell::Cell<usize>>);

#[allow(dead_code)]
impl Counter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self) -> usize {
        self.0.get()
    }

    pub fn bump(&self) {
        self.0.set(self.0.get() + 1);
    }
}
