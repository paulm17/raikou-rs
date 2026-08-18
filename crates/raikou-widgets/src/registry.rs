//! The component registry: maps built widget handles to their handlers so the
//! app's message poll loop can dispatch events back to raikou callbacks.

use std::collections::HashMap;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::{UiNode, UserInterface};

use crate::component::{Component, ComponentKind};

/// Maps native fyrox widget handles to the raikou handlers that drive them.
///
/// The app is expected to own one of these and call [`Self::dispatch`] for
/// every message pulled from the UI queue.
#[derive(Default)]
pub struct ComponentRegistry {
    map: HashMap<Handle<UiNode>, ComponentKind>,
}

impl ComponentRegistry {
    /// Registers (or replaces) the handlers of a built component.
    pub fn register(&mut self, component: &Component) {
        self.map.insert(component.handle, component.kind.clone());
    }

    /// Removes the handlers associated with a widget handle.
    pub fn unregister(&mut self, handle: Handle<UiNode>) {
        self.map.remove(&handle);
    }

    /// Removes all registered handlers.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Number of registered components.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether no components are registered.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Dispatches a UI message to the handlers of its destination widget, if
    /// any. Called from the app's message poll loop for every message.
    pub fn dispatch(&mut self, ui: &mut UserInterface, message: &UiMessage) {
        let Some(kind) = self.map.get_mut(&message.destination()) else {
            return;
        };
        kind.dispatch(ui, message);
    }
}
