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
    /// Handlers that observe messages aimed anywhere in the tree. Used for
    /// window-wide behaviors (Enter-to-default button, focus rings,
    /// double-click detection) whose trigger messages are aimed at whatever
    /// node happens to sit under the cursor.
    globals: Vec<(Handle<UiNode>, ComponentKind)>,
}

impl ComponentRegistry {
    /// Registers (or replaces) the handlers of a built component.
    pub fn register(&mut self, component: &Component) {
        self.map.insert(component.handle, component.kind.clone());
    }

    /// Registers handlers that receive every message in the UI, regardless of
    /// its destination. Messages aimed at the registered handle itself are
    /// still delivered only through [`Self::register`] (the global pass skips
    /// them), so a widget can be both an exact and a global listener without
    /// seeing its own messages twice.
    pub fn register_global(&mut self, component: &Component) {
        let kind_id = std::mem::discriminant(&component.kind);
        let duplicate = self.globals.iter().any(|(h, k)| {
            *h == component.handle && std::mem::discriminant(k) == kind_id
        });
        if !duplicate {
            self.globals
                .push((component.handle, component.kind.clone()));
        }
    }

    /// Removes the handlers associated with a widget handle.
    pub fn unregister(&mut self, handle: Handle<UiNode>) {
        self.map.remove(&handle);
        self.globals.retain(|(h, _)| *h != handle);
    }

    /// Removes all registered handlers.
    pub fn clear(&mut self) {
        self.map.clear();
        self.globals.clear();
    }

    /// Number of registered components.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether no components are registered.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Dispatches a UI message to the handlers of its destination widget and
    /// to every global listener. Called from the app's message poll loop for
    /// every message.
    pub fn dispatch(&mut self, ui: &mut UserInterface, message: &UiMessage) {
        let destination = message.destination();
        let mut exact_kind = None;
        if let Some(kind) = self.map.get_mut(&destination) {
            exact_kind = Some(std::mem::discriminant(kind));
            kind.dispatch(ui, message);
        }
        for (handle, kind) in &self.globals {
            // Skip a global listener only when the very same handler kind
            // already saw this message through the exact pass; other watchers
            // sharing the handle (focus ring vs. word select) must still run.
            if *handle == destination && exact_kind == Some(std::mem::discriminant(kind)) {
                continue;
            }
            kind.dispatch(ui, message);
        }
    }
}
