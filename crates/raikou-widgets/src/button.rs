//! The Button component.
//!
//! Mirrors the raikou `Button` builder API and maps it onto fyrox's
//! `ButtonBuilder` + `DecoratorBuilder` (for stateful backgrounds).

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::graph::SceneGraph;
use fyrox::gui::border::BorderBuilder;
use fyrox::gui::brush::Brush;
use fyrox::gui::button::{ButtonBuilder, ButtonMessage};
use fyrox::gui::decorator::DecoratorBuilder;
use fyrox::gui::dropdown_list::DropdownList;
use fyrox::gui::list_view::ListView;
use fyrox::gui::message::{KeyCode, MessageDirection, MouseButton, UiMessage};
use fyrox::gui::text_box::TextBox;
use fyrox::gui::tree::Tree;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::{Color, ControlSize, Length, Thickness};
use raikou_style::ButtonVariant;

use crate::build_cx::BuildCx;
use crate::component::{ClickEvent, Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};
use crate::loading_indicator::{LoadingIndicator, LoadingIndicatorMode};

type ClickCallback = dyn Fn(&mut UserInterface, &ClickEvent);
type SimpleCallback = dyn Fn(&mut UserInterface);

/// Controls when a button's `on_click` fires.
///
/// Mirrors the raikou reference `ClickMode` semantics:
/// - [`ClickMode::Release`] (default) fires when the pointer is released over
///   the button (and on keyboard activation).
/// - [`ClickMode::Press`] fires immediately on pointer press.
/// - [`ClickMode::Hover`] fires when the pointer first enters the button.
///
/// Keyboard activation (Enter/Space) fires in every mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ClickMode {
    /// Fires on pointer release over the button.
    #[default]
    Release,
    /// Fires on pointer press.
    Press,
    /// Fires on pointer enter.
    Hover,
}

/// Event handlers of a Button component, kept in the [`crate::ComponentRegistry`].
#[derive(Clone)]
pub struct ButtonHandlers {
    /// Invoked when the button is clicked.
    pub on_click: Option<Rc<ClickCallback>>,
    /// Invoked when the pointer enters the button.
    pub on_mouse_over: Option<Rc<SimpleCallback>>,
    /// Invoked when the pointer leaves the button.
    pub on_mouse_out: Option<Rc<SimpleCallback>>,
    /// Controls when `on_click` fires.
    pub click_mode: ClickMode,
    /// When `true` all interaction is suppressed (the button shows a spinner).
    pub loading: bool,
    /// Handle of the button widget (used to skip messages aimed at itself).
    pub(crate) handle: Handle<UiNode>,
    /// Whether this is the window's default (Enter) button. When set, Enter
    /// pressed anywhere outside an interactive control activates it — fyrox
    /// only activates the *focused* button natively.
    pub(crate) is_default: bool,
    /// `true` for the registry-global clone that exists purely to provide
    /// default-button wiring; it must never run click callbacks, or every
    /// sibling button's Click would leak into it.
    pub(crate) wiring_only: bool,
}

impl ButtonHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if self.loading {
            return;
        }
        if self.wiring_only {
            self.run_default_wiring(ui, message);
            return;
        }

        // Default-button wiring: Enter activates this button even when focus
        // sits elsewhere (Avalonia `IsDefault`). Messages aimed at this very
        // button fall through so the focused case fires exactly once via
        // fyrox's native handling. Interactive destinations (other buttons,
        // text boxes, lists...) keep their keys; Space never triggers the
        // default button.
        self.run_default_wiring(ui, message);

        if let Some(WidgetMessage::MouseEnter) = message.data::<WidgetMessage>() {
            if let Some(callback) = &self.on_mouse_over {
                callback(ui);
            }
            if self.click_mode == ClickMode::Hover {
                self.fire_click(ui, message);
            }
            return;
        }
        if let Some(WidgetMessage::MouseLeave) = message.data::<WidgetMessage>() {
            if let Some(callback) = &self.on_mouse_out {
                callback(ui);
            }
            return;
        }

        match self.click_mode {
            ClickMode::Release => {
                if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>() {
                    self.fire_click(ui, message);
                }
            }
            ClickMode::Press => {
                if let Some(WidgetMessage::MouseDown { button, .. }) =
                    message.data::<WidgetMessage>()
                {
                    if *button == MouseButton::Left {
                        self.fire_click(ui, message);
                    }
                } else if let Some(WidgetMessage::KeyUp(key)) = message.data::<WidgetMessage>() {
                    if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) {
                        self.fire_click(ui, message);
                    }
                }
            }
            ClickMode::Hover => {
                if let Some(WidgetMessage::KeyUp(key)) = message.data::<WidgetMessage>() {
                    if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) {
                        self.fire_click(ui, message);
                    }
                }
            }
        }
    }

    /// Default-button wiring shared by the exact-path and global handler
    /// clones (see `wiring_only`).
    fn run_default_wiring(&self, ui: &mut UserInterface, message: &UiMessage) {
        if !self.is_default
            || message.direction() != MessageDirection::ToWidget
            || message.destination() == self.handle
        {
            return;
        }
        if let Some(WidgetMessage::KeyDown(key)) = message.data::<WidgetMessage>() {
            if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter)
                && !destination_is_interactive(ui, message.destination())
            {
                self.fire_click(ui, message);
            }
        }
    }

    fn fire_click(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(callback) = &self.on_click {
            let event = ClickEvent {
                widget_id: message.destination(),
                position: None,
                modifiers: Some(ui.keyboard_modifiers()),
            };
            callback(ui, &event);
        }
    }
}

/// Walks the ancestors of `destination` and reports whether focus effectively
/// sits in a control that consumes Enter itself (buttons — including menu
/// items — multi-line text boxes, dropdowns, trees and lists). Single-line
/// text boxes do NOT consume Enter, matching Avalonia's default-button rule.
fn destination_is_interactive(ui: &UserInterface, destination: Handle<UiNode>) -> bool {
    let mut current = destination;
    while current.is_some() {
        let Ok(node) = ui.try_get_node(current) else {
            break;
        };
        if node.cast::<fyrox::gui::button::Button>().is_some()
            || node
                .cast::<TextBox>()
                .is_some_and(|text_box| *text_box.multiline)
            || node.cast::<DropdownList>().is_some()
            || node.cast::<Tree>().is_some()
            || node.cast::<ListView>().is_some()
        {
            return true;
        }
        current = node.parent();
    }
    false
}

/// Builder for a [`crate::Button`] component.
///
/// ```rust,ignore
/// let button = Button::new()
///     .text("Save")
///     .variant(ButtonVariant::Filled)
///     .size(ControlSize::Medium)
///     .on_click(|ui, _event| println!("clicked!"))
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct Button {
    label: String,
    variant: ButtonVariant,
    size: ControlSize,
    width: Length,
    height: Length,
    padding: Thickness,
    margin: Thickness,
    corner_radius: f32,
    is_default: bool,
    is_cancel: bool,
    click_mode: ClickMode,
    loading: bool,
    content: Option<Handle<UiNode>>,
    on_click: Option<Rc<ClickCallback>>,
    on_mouse_over: Option<Rc<SimpleCallback>>,
    on_mouse_out: Option<Rc<SimpleCallback>>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    /// Creates a new button builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            variant: ButtonVariant::Filled,
            size: ControlSize::Medium,
            width: Length::Auto,
            height: Length::Shrink,
            padding: ControlSize::Medium.padding(),
            margin: Thickness::ZERO,
            corner_radius: 4.0,
            is_default: false,
            is_cancel: false,
            click_mode: ClickMode::Release,
            loading: false,
            content: None,
            on_click: None,
            on_mouse_over: None,
            on_mouse_out: None,
        }
    }

    /// Sets the button label text. Clears any custom child content.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self.content = None;
        self
    }

    /// Sets the visual appearance.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the control size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self.padding = size.padding();
        self
    }

    /// Sets an explicit width.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets an explicit height.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the content padding.
    pub fn padding(mut self, padding: Thickness) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the corner radius in logical pixels.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Marks the button as the default (Enter) button of the window.
    pub fn is_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    /// Marks the button as the cancel (Escape) button of the window.
    pub fn is_cancel(mut self, is_cancel: bool) -> Self {
        self.is_cancel = is_cancel;
        self
    }

    /// Sets when `on_click` fires (release, press, or hover).
    pub fn click_mode(mut self, click_mode: ClickMode) -> Self {
        self.click_mode = click_mode;
        self
    }

    /// Puts the button into a loading state. Interaction is suppressed and the
    /// label is replaced by an animated spinner.
    pub fn is_loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Sets a custom child widget as the button content, replacing the label.
    pub fn content(mut self, content: Handle<UiNode>) -> Self {
        self.content = Some(content);
        self.label.clear();
        self
    }

    /// Sets the callback invoked when the button is clicked.
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, &ClickEvent) + 'static,
    {
        self.on_click = Some(Rc::new(callback));
        self
    }

    /// Sets the callback invoked when the pointer enters the button.
    pub fn on_mouse_over<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface) + 'static,
    {
        self.on_mouse_over = Some(Rc::new(callback));
        self
    }

    /// Sets the callback invoked when the pointer leaves the button.
    pub fn on_mouse_out<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface) + 'static,
    {
        self.on_mouse_out = Some(Rc::new(callback));
        self
    }

    /// Builds the button, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let style = cx.theme().resolve_button_style(self.variant, self.size);
        let font_size = style.font_size;

        let decorator: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            DecoratorBuilder::new(
                BorderBuilder::new(WidgetBuilder::new())
                    .with_pad_by_corner_radius(false)
                    .with_corner_radius(style.corner_radius.into())
                    .with_stroke_thickness(to_fyrox_thickness(style.border_thickness).into()),
            )
            .with_normal_brush(Brush::Solid(to_fyrox_color(style.background)).into())
            .with_hover_brush(Brush::Solid(to_fyrox_color(style.hover)).into())
            .with_pressed_brush(Brush::Solid(to_fyrox_color(style.pressed)).into())
            .build(&mut ctx)
            .to_base()
        };

        let mut widget_builder = WidgetBuilder::new()
            .with_name("raikou_button")
            .with_margin(to_fyrox_thickness(self.margin))
            .with_foreground(Brush::Solid(to_fyrox_color(style.text)).into())
            .with_background(Brush::Solid(to_fyrox_color(Color::TRANSPARENT)).into());
        if let Some(width) = self.width.resolve() {
            widget_builder = widget_builder.with_width(width);
        }
        if let Some(height) = self.height.resolve() {
            widget_builder = widget_builder.with_height(height);
        } else {
            // Fluent controls have fixed heights per size; this also stops
            // stretchy parents inflating the measured height.
            widget_builder = widget_builder.with_height(self.size.min_height());
        }
        if self.padding != Thickness::ZERO {
            widget_builder = widget_builder.with_min_size(fyrox::core::algebra::Vector2::new(
                self.padding.left + self.padding.right,
                self.padding.top + self.padding.bottom,
            ));
        }

        let spinner_node = if self.loading {
            Some(LoadingIndicator::build_node(
                cx,
                LoadingIndicator::new()
                    .mode(LoadingIndicatorMode::Pulse)
                    .color(style.text)
                    .size(self.size.icon_size())
                    .stroke_width(2.0),
            ))
        } else {
            None
        };

        let handle = {
            let mut ctx = cx.ctx();
            let mut builder = ButtonBuilder::new(widget_builder);
            if let Some(spinner) = spinner_node {
                builder = builder.with_content(spinner);
            } else if let Some(content) = self.content {
                builder = builder.with_content(content);
            } else {
                let font = ctx.default_font();
                builder = builder.with_text_and_font_size(&self.label, font, font_size.into());
            }
            builder.with_back(decorator).build(&mut ctx).to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Button(ButtonHandlers {
                on_click: self.on_click,
                on_mouse_over: self.on_mouse_over,
                on_mouse_out: self.on_mouse_out,
                click_mode: self.click_mode,
                loading: self.loading,
                handle,
                is_default: self.is_default,
                wiring_only: false,
            }),
        };
        cx.register(&component);
        if self.is_default {
            // The default button must see Enter presses aimed anywhere in the
            // window. The global clone is wiring-only (no click callbacks) so
            // sibling clicks never leak into it; the pass skips messages
            // aimed at this handle so the focused case fires exactly once.
            let global = Component {
                handle,
                kind: ComponentKind::Button(ButtonHandlers {
                    wiring_only: true,
                    ..component_kind_handlers(&component)
                }),
            };
            cx.register_global(&global);
        }
        component
    }
}

/// Clones a button component's handler payload (used for the wiring-only
/// global registration).
fn component_kind_handlers(component: &Component) -> ButtonHandlers {
    match &component.kind {
        ComponentKind::Button(handlers) => handlers.clone(),
        _ => unreachable!("button component always carries Button handlers"),
    }
}

/// A handle to a built button, returned for convenience.
pub type ButtonHandle = Handle<UiNode>;
