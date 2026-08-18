//! The Slider component.
//!
//! Maps onto fyrox's `ScrollBarBuilder` (the documented slider in this fyrox
//! version) and reports value changes through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::scroll_bar::{ScrollBarBuilder, ScrollBarMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{Orientation, UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, f32);

/// Event handlers of a Slider component.
#[derive(Clone)]
pub struct SliderHandlers {
    /// Invoked with the new value whenever the slider moves.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl SliderHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(ScrollBarMessage::Value(value)) = message.data::<ScrollBarMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, *value);
            }
        }
    }
}

/// Builder for a [`Slider`] component.
#[derive(Clone)]
pub struct Slider {
    min: f32,
    max: f32,
    value: f32,
    step: f32,
    orientation: Orientation,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Slider {
    /// Creates a new slider builder.
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 0.0,
            step: 1.0,
            orientation: Orientation::Horizontal,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the minimum value.
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value.
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Sets the initial value.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Sets the step by which the value snaps.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Sets the orientation.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the slider value changes.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, f32) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the slider, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_slider")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            ScrollBarBuilder::new(widget_builder)
                .with_min(self.min)
                .with_max(self.max)
                .with_value(self.value)
                .with_step(self.step)
                .with_orientation(self.orientation)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Slider(SliderHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built slider, returned for convenience.
pub type SliderHandle = Handle<UiNode>;
