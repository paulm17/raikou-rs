//! Component identity and dispatch: the bridge between a raikou builder and
//! the fyrox message system.

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::message::{KeyboardModifiers, UiMessage};
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};

use crate::accordion::AccordionItemHandlers;
use crate::button::ButtonHandlers;
use crate::checkbox::CheckboxHandlers;
use crate::combobox::ComboboxHandlers;
use crate::context_menu::ContextMenuHandlers;
use crate::menu::MenuBarHandlers;
use crate::radio::{RadioGroupHandlers, RadioGroupItemHandlers, RadioHandlers};
use crate::scroll_area::ScrollAreaHandlers;
use crate::select::SelectHandlers;
use crate::slider::SliderHandlers;
use crate::step_input::StepInputHandlers;
use crate::switch::SwitchHandlers;
use crate::tabs::TabsHandlers;
use crate::text_area::TextAreaHandlers;
use crate::text_input::TextInputHandlers;
use crate::tree::TreeHandlers;

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
    /// Handlers for a [`crate::Checkbox`] component.
    Checkbox(CheckboxHandlers),
    /// Handlers for a [`crate::Switch`] component.
    Switch(SwitchHandlers),
    /// Handlers for a single [`crate::Radio`] option.
    Radio(RadioHandlers),
    /// Handlers for a [`crate::RadioGroup`] container.
    RadioGroup(RadioGroupHandlers),
    /// Handlers for one option within a [`crate::RadioGroup`].
    RadioGroupItem(RadioGroupItemHandlers),
    /// Handlers for a [`crate::Slider`] component.
    Slider(SliderHandlers),
    /// Handlers for a [`crate::TextInput`] component.
    TextInput(TextInputHandlers),
    /// Handlers for a [`crate::TextArea`] component.
    TextArea(TextAreaHandlers),
    /// Handlers for a [`crate::StepInput`] component.
    StepInput(StepInputHandlers),
    /// Handlers for one item within an [`crate::Accordion`].
    AccordionItem(AccordionItemHandlers),
    /// Handlers for a [`crate::Tabs`] component.
    Tabs(TabsHandlers),
    /// Handlers for a [`crate::ScrollArea`] component.
    ScrollArea(ScrollAreaHandlers),
    /// Handlers for a [`crate::MenuBar`] component.
    MenuBar(MenuBarHandlers),
    /// Handlers for a [`crate::ContextMenu`] component.
    ContextMenu(ContextMenuHandlers),
    /// Handlers for a [`crate::Select`] component.
    Select(SelectHandlers),
    /// Handlers for a [`crate::Combobox`] component.
    Combobox(ComboboxHandlers),
    /// Handlers for a [`crate::Tree`] component.
    Tree(TreeHandlers),
    /// A component with no dispatchable handlers (e.g. a static label or box).
    Static,
}

impl ComponentKind {
    /// Routes a message to the handlers of this component.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        match self {
            ComponentKind::Button(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Checkbox(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Switch(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Radio(handlers) => handlers.dispatch(ui, message),
            ComponentKind::RadioGroup(handlers) => handlers.dispatch(ui, message),
            ComponentKind::RadioGroupItem(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Slider(handlers) => handlers.dispatch(ui, message),
            ComponentKind::TextInput(handlers) => handlers.dispatch(ui, message),
            ComponentKind::TextArea(handlers) => handlers.dispatch(ui, message),
            ComponentKind::StepInput(handlers) => handlers.dispatch(ui, message),
            ComponentKind::AccordionItem(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Tabs(handlers) => handlers.dispatch(ui, message),
            ComponentKind::ScrollArea(handlers) => handlers.dispatch(ui, message),
            ComponentKind::MenuBar(handlers) => handlers.dispatch(ui, message),
            ComponentKind::ContextMenu(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Select(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Combobox(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Tree(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Static => {}
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
