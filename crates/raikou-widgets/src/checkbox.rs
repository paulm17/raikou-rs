//! The Checkbox component.
//!
//! Maps onto fyrox's `CheckBoxBuilder` and reports state changes through a
//! per-component `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::check_box::{CheckBoxBuilder, CheckBoxMessage};
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, bool);

/// Event handlers of a Checkbox component.
#[derive(Clone)]
pub struct CheckboxHandlers {
    /// Invoked with the new checked state whenever the box is toggled.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl CheckboxHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(CheckBoxMessage::Check(state)) = message.data::<CheckBoxMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, state.unwrap_or(false));
            }
        }
    }
}

/// Builder for a [`Checkbox`] component.
///
/// ```rust,ignore
/// let checkbox = Checkbox::new()
///     .checked(true)
///     .on_change(|ui, checked| println!("checked: {checked}"))
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct Checkbox {
    label: String,
    checked: bool,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Checkbox {
    /// Creates a new checkbox builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            checked: false,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the checkbox label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Sets the initial checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the checkbox is toggled.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, bool) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the checkbox, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let label_handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            fyrox::gui::text::TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(to_fyrox_thickness(Thickness::new(0.0, 0.0, 0.0, 0.0))),
            )
            .with_text(&self.label)
            .with_font(font)
            .build(&mut ctx)
            .to_base()
        };

        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_checkbox")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            CheckBoxBuilder::new(widget_builder)
                .checked(Some(self.checked))
                .with_content(label_handle)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Checkbox(CheckboxHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built checkbox, returned for convenience.
pub type CheckboxHandle = Handle<UiNode>;
