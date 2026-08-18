//! The StepInput component (numeric stepper).
//!
//! Maps onto fyrox's `NumericUpDownBuilder<f64>` and reports value changes
//! through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::UiMessage;
use fyrox::gui::numeric::{NumericUpDownBuilder, NumericUpDownMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, f64);

/// Event handlers of a StepInput component.
#[derive(Clone)]
pub struct StepInputHandlers {
    /// Invoked with the new value whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl StepInputHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(NumericUpDownMessage::Value(value)) = message.data::<NumericUpDownMessage<f64>>()
        {
            if let Some(callback) = &self.on_change {
                callback(ui, *value);
            }
        }
    }
}

/// Builder for a [`StepInput`] component.
#[derive(Clone)]
pub struct StepInput {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    precision: usize,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for StepInput {
    fn default() -> Self {
        Self::new()
    }
}

impl StepInput {
    /// Creates a new step input builder.
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min: f64::MIN,
            max: f64::MAX,
            step: 1.0,
            precision: 0,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the initial value.
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Sets the minimum value.
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value.
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Sets the step by which the value changes.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Sets the number of decimal places displayed.
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the value changes.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, f64) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the step input, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_step_input")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            NumericUpDownBuilder::<f64>::new(widget_builder)
                .with_value(self.value)
                .with_min_value(self.min)
                .with_max_value(self.max)
                .with_step(self.step)
                .with_precision(self.precision)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::StepInput(StepInputHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built step input, returned for convenience.
pub type StepInputHandle = Handle<UiNode>;
