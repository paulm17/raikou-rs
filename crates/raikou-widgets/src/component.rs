//! Component identity and dispatch: the bridge between a raikou builder and
//! the fyrox message system.

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::message::{KeyboardModifiers, UiMessage};
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};

use crate::button::ButtonHandlers;

/// Payload of a click event delivered to an `on_click` callback.
#[derive(Debug, Clone)]
pub struct ClickEvent {
    /// Handle of the widget that was clicked.
    pub widget_id: Handle<UiNode>,
    /// Cursor position at the moment of the click, if known.
    pub position: Option<Vector2<f32>>,
    /// Keyboard modifiers held during the click, if known.
    pub modifiers: Option<KeyboardModifiers>,
}

/// Per-component event handlers. Produced by a builder's `build`, stored by
/// the [`crate::ComponentRegistry`] and invoked from the message poll loop.
#[derive(Clone)]
pub enum ComponentKind {
    /// Handlers for a [`crate::Button`] component.
    Button(ButtonHandlers),
}

impl ComponentKind {
    /// Routes a message to the handlers of this component.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        match self {
            ComponentKind::Button(handlers) => handlers.dispatch(ui, message),
        }
    }
}

/// A built component: the handle of the native fyrox widget it spawned, plus
/// the handlers registered for it.
///
/// Convert to `Handle<UiNode>` (via `From`) to compose it as a child of other
/// fyrox widgets or containers.
#[derive(Clone)]
pub struct Component {
    /// Handle of the native fyrox widget that backs this component.
    pub handle: Handle<UiNode>,
    /// Event handlers registered for this component.
    pub kind: ComponentKind,
}

impl Component {
    /// Enables or disables the widget (disabling greys it out and blocks
    /// pointer/keyboard input).
    pub fn set_enabled(&self, ui: &mut UserInterface, enabled: bool) {
        ui.send(self.handle, WidgetMessage::Enabled(enabled));
    }

    /// Shows or hides the widget.
    pub fn set_visible(&self, ui: &mut UserInterface, visible: bool) {
        ui.send(self.handle, WidgetMessage::Visibility(visible));
    }

    /// Removes the widget (and its children) from the UI.
    pub fn remove(&self, ui: &mut UserInterface) {
        ui.send(self.handle, WidgetMessage::Remove);
    }

    /// Sends a raw message to the widget.
    pub fn send<M: fyrox::gui::message::MessageData>(&self, ui: &mut UserInterface, message: M) {
        ui.send(self.handle, message);
    }
}

impl From<Component> for Handle<UiNode> {
    fn from(component: Component) -> Self {
        component.handle
    }
}

impl From<&Component> for Handle<UiNode> {
    fn from(component: &Component) -> Self {
        component.handle
    }
}
