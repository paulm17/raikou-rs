//! The Button component.
//!
//! Mirrors the raikou `Button` builder API and maps it onto fyrox's
//! `ButtonBuilder` + `DecoratorBuilder` (for stateful backgrounds).

use std::rc::Rc;

use fyrox::core::color::Color;
use fyrox::core::pool::Handle;
use fyrox::gui::border::BorderBuilder;
use fyrox::gui::brush::Brush;
use fyrox::gui::button::{ButtonBuilder, ButtonMessage};
use fyrox::gui::decorator::DecoratorBuilder;
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface, Thickness};

use raikou_core::{ControlSize, Length};
use raikou_style::ButtonVariant;

use crate::build_cx::BuildCx;
use crate::component::{ClickEvent, Component, ComponentKind};

type ClickCallback = dyn Fn(&mut UserInterface, &ClickEvent);
type SimpleCallback = dyn Fn(&mut UserInterface);

/// Event handlers of a Button component, kept in the [`crate::ComponentRegistry`].
#[derive(Clone)]
pub struct ButtonHandlers {
    /// Invoked when the button is clicked.
    pub on_click: Option<Rc<ClickCallback>>,
    /// Invoked when the pointer enters the button.
    pub on_mouse_over: Option<Rc<SimpleCallback>>,
    /// Invoked when the pointer leaves the button.
    pub on_mouse_out: Option<Rc<SimpleCallback>>,
}

impl ButtonHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>() {
            if let Some(callback) = &self.on_click {
                let event = ClickEvent {
                    widget_id: message.destination(),
                    position: None,
                    modifiers: Some(ui.keyboard_modifiers()),
                };
                callback(ui, &event);
            }
        } else if let Some(WidgetMessage::MouseEnter) = message.data::<WidgetMessage>() {
            if let Some(callback) = &self.on_mouse_over {
                callback(ui);
            }
        } else if let Some(WidgetMessage::MouseLeave) = message.data::<WidgetMessage>() {
            if let Some(callback) = &self.on_mouse_out {
                callback(ui);
            }
        }
    }
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
            margin: Thickness::zero(),
            corner_radius: 4.0,
            is_default: false,
            is_cancel: false,
            on_click: None,
            on_mouse_over: None,
            on_mouse_out: None,
        }
    }

    /// Sets the button label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
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
                    .with_stroke_thickness(style.border_thickness.into()),
            )
            .with_normal_brush(Brush::Solid(style.background).into())
            .with_hover_brush(Brush::Solid(style.hover).into())
            .with_pressed_brush(Brush::Solid(style.pressed).into())
            .build(&mut ctx)
            .to_base()
        };

        let mut widget_builder = WidgetBuilder::new()
            .with_name("raikou_button")
            .with_margin(self.margin)
            .with_background(Brush::Solid(Color::TRANSPARENT).into());
        if let Some(width) = self.width.resolve() {
            widget_builder = widget_builder.with_width(width);
        }
        if let Some(height) = self.height.resolve() {
            widget_builder = widget_builder.with_height(height);
        }

        let handle = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            ButtonBuilder::new(widget_builder)
                .with_text_and_font_size(&self.label, font, font_size.into())
                .with_back(decorator)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Button(ButtonHandlers {
                on_click: self.on_click,
                on_mouse_over: self.on_mouse_over,
                on_mouse_out: self.on_mouse_out,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built button, returned for convenience.
pub type ButtonHandle = Handle<UiNode>;
