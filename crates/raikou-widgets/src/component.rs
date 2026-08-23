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
use crate::image::ImageHandlers;
use crate::menu::MenuBarHandlers;
use crate::radio::{RadioGroupHandlers, RadioGroupItemHandlers, RadioHandlers};
use crate::scroll_area::ScrollAreaHandlers;
use crate::select::SelectHandlers;
use crate::slider::{SliderHandlers, SliderJump, SliderNav};
use crate::step_input::StepInputHandlers;
use crate::switch::SwitchHandlers;
use crate::select::SelectNavHandlers;
use crate::tabs::{TabsHandlers, TabsNavHandlers};
use crate::text_area::TextAreaHandlers;
use crate::text_input::{FocusRingHandlers, TextInputHandlers, WordSelectHandlers};
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
    /// Global watcher jumping a slider to a clicked track position.
    SliderJump(SliderJump),
    /// Global key watcher giving sliders arrow/Home/End navigation.
    SliderNav(SliderNav),
    /// Handlers for a [`crate::TextInput`] component.
    TextInput(TextInputHandlers),
    /// Global focus watcher that accents a text field's chrome while any node
    /// of its subtree holds keyboard focus (see [`crate::TextInput`]).
    FocusRing(FocusRingHandlers),
    /// Global double-click watcher that selects the word under the caret of a
    /// text box (presses are aimed at whatever child sits under the cursor).
    WordSelect(WordSelectHandlers),
    /// Global key watcher that cycles an open dropdown's list with the arrow
    /// keys (focus sits inside the flyout, so exact-path dispatch misses it).
    SelectNav(SelectNavHandlers),
    /// Global key watcher that switches tabs with Left/Right (fyrox's
    /// TabControl has no keyboard handling of its own).
    TabsNav(TabsNavHandlers),
    /// Handlers for a [`crate::TextArea`] component.
    TextArea(TextAreaHandlers),
    /// Handlers for a [`crate::StepInput`] component.
    StepInput(StepInputHandlers),
    /// Handlers for one item within an [`crate::Accordion`].
    AccordionItem(AccordionItemHandlers),
    /// Global watcher making a whole [`crate::Accordion`] header a click
    /// target (mouse-ups land on whatever deep child sits under the cursor).
    AccordionHeaderHit(crate::accordion::AccordionHeaderHit),
    /// Handlers for a [`crate::Tabs`] component.
    Tabs(TabsHandlers),
    /// Handlers for a [`crate::ScrollArea`] component.
    ScrollArea(ScrollAreaHandlers),
    /// Overlay-thumb auto-hide helpers registered on every node of a
    /// [`crate::ScrollArea`] subtree (enter/leave are aimed at the exact
    /// widget under the cursor, so every descendant needs a registration).
    ScrollAreaAutoHide(crate::scroll_area::ScrollAutoHideHandlers),
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
    /// Handlers for an [`crate::Image`] component.
    Image(ImageHandlers),
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
            ComponentKind::SliderJump(handlers) => handlers.dispatch(ui, message),
            ComponentKind::SliderNav(handlers) => handlers.dispatch(ui, message),
            ComponentKind::TextInput(handlers) => handlers.dispatch(ui, message),
            ComponentKind::FocusRing(handlers) => handlers.dispatch(ui, message),
            ComponentKind::WordSelect(handlers) => handlers.dispatch(ui, message),
            ComponentKind::SelectNav(handlers) => handlers.dispatch(ui, message),
            ComponentKind::TabsNav(handlers) => handlers.dispatch(ui, message),
            ComponentKind::TextArea(handlers) => handlers.dispatch(ui, message),
            ComponentKind::StepInput(handlers) => handlers.dispatch(ui, message),
            ComponentKind::AccordionItem(handlers) => handlers.dispatch(ui, message),
            ComponentKind::AccordionHeaderHit(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Tabs(handlers) => handlers.dispatch(ui, message),
            ComponentKind::ScrollArea(handlers) => handlers.dispatch(ui, message),
            ComponentKind::ScrollAreaAutoHide(handlers) => handlers.dispatch(ui, message),
            ComponentKind::MenuBar(handlers) => handlers.dispatch(ui, message),
            ComponentKind::ContextMenu(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Select(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Combobox(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Tree(handlers) => handlers.dispatch(ui, message),
            ComponentKind::Image(_) => {}
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

/// Walks the ancestors of `start` (inclusive) and reports whether `target`
/// is among them. Global watchers use this to attribute messages aimed at
/// whatever deep child the pointer or focus landed on.
pub(crate) fn is_in_subtree(
    ui: &UserInterface,
    start: Handle<UiNode>,
    target: Handle<UiNode>,
) -> bool {
    use fyrox::graph::SceneGraph;

    let mut current = start;
    while current.is_some() {
        if current == target {
            return true;
        }
        let Ok(node) = ui.try_get_node(current) else {
            break;
        };
        current = node.parent();
    }
    false
}
