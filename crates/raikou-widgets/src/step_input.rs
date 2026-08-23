//! The StepInput component (numeric stepper).
//!
//! Maps onto fyrox's `NumericUpDownBuilder<f64>` and reports value changes
//! through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::numeric::{NumericUpDown, NumericUpDownBuilder, NumericUpDownMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_color;

type ChangeCallback = dyn Fn(&mut UserInterface, f64);

/// Event handlers of a StepInput component.
#[derive(Clone)]
pub struct StepInputHandlers {
    /// Invoked with the new value whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
    /// The inner numeric widget that receives programmatic commands.
    pub command_target: Handle<UiNode>,
}

impl StepInputHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(value) = message.data::<NumericUpDownMessage<f64>>() {
            // Forward ToWidget commands aimed at the outer chrome to the
            // inner numeric widget (skips the forwarded copy itself).
            if message.direction() == MessageDirection::ToWidget
                && message.destination() != self.command_target
            {
                ui.send(self.command_target, value.clone());
                return;
            }
            if message.direction() != MessageDirection::FromWidget {
                return;
            }
            if let Some(NumericUpDownMessage::Value(value)) =
                message.data::<NumericUpDownMessage<f64>>()
            {
                if let Some(callback) = &self.on_change {
                    callback(ui, *value);
                }
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
        let theme = cx.theme().clone();

        let inner = {
            let widget_builder = WidgetBuilder::new().with_name("raikou_step_input_inner");

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

        let handle = {
            let mut ctx = cx.ctx();
            crate::field::field_chrome(
                &mut ctx,
                &theme,
                inner,
                crate::field::FIELD_MIN_HEIGHT,
                self.margin,
            )
        };

        // Fluent restyle: spinner plates are transparent until hovered and
        // the glyphs become thin stroked chevrons in the theme text color
        // (Avalonia NumericUpDown spinners).
        fluent_spinner_buttons(&mut cx.ctx(), &theme, inner);

        let component = Component {
            handle,
            kind: ComponentKind::StepInput(StepInputHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
            }),
        };
        cx.register(&component);
        // The inner numeric widget emits the FromWidget messages; register it
        // too so exact-destination dispatch finds the handlers.
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::StepInput(StepInputHandlers {
                on_change: self.on_change,
                command_target: inner,
            }),
        });
        component
    }
}

/// Fluent restyle of the stock spinner buttons: plates are transparent until
/// hovered (subtle list tint on hover/press, no stroke border) and the filled
/// accent triangles become thin stroked chevrons in the theme's secondary
/// text color.
fn fluent_spinner_buttons(
    ctx: &mut fyrox::gui::BuildContext,
    theme: &raikou_style::Theme,
    inner: Handle<UiNode>,
) {
    use fyrox::core::algebra::Vector2;
    use fyrox::gui::button::{Button, ButtonMessage};
    use fyrox::gui::decorator::Decorator;
    use fyrox::gui::vector_image::{Primitive, VectorImage};

    let buttons = {
        let node = &ctx[inner];
        let Some(numeric) = node.cast::<NumericUpDown<f64>>() else {
            return;
        };
        let inc: Handle<UiNode> = (*numeric.increase).to_base();
        let dec: Handle<UiNode> = (*numeric.decrease).to_base();
        [(inc, true), (dec, false)]
    };

    let fallback_glyph = raikou_core::Color::new(0.35, 0.35, 0.39, 1.0);
    let glyph = Brush::Solid(to_fyrox_color(
        theme.color("text.secondary").unwrap_or(fallback_glyph),
    ));
    let fallback_low = raikou_core::Color::new(0.0, 0.0, 0.0, 0.05);
    let hover_plate = Brush::Solid(to_fyrox_color(
        theme.color("fluent.list.low").unwrap_or(fallback_low),
    ));
    let fallback_medium = raikou_core::Color::new(0.0, 0.0, 0.0, 0.10);
    let pressed_plate = Brush::Solid(to_fyrox_color(
        theme.color("fluent.list.medium").unwrap_or(fallback_medium),
    ));
    let transparent = Brush::Solid(fyrox::core::color::Color::TRANSPARENT);

    // The stock numeric paints its own dark back border, which shows through
    // around the spinners; the raikou field chrome already provides the
    // Fluent chrome, so retire the stock fill and stroke.
    let back_border = ctx[inner].children().first().copied();
    if let Some(back) = back_border {
        if let Some(b) = ctx[back].cast_mut::<fyrox::gui::border::Border>() {
            b.widget
                .background
                .set_value_and_mark_modified(transparent.clone().into());
            b.widget
                .foreground
                .set_value_and_mark_modified(transparent.clone().into());
        }
    }

    let thickness = 1.5;
    let up_prims = vec![
        Primitive::Line {
            begin: Vector2::new(1.5, 3.25),
            end: Vector2::new(4.0, 0.75),
            thickness,
        },
        Primitive::Line {
            begin: Vector2::new(4.0, 0.75),
            end: Vector2::new(6.5, 3.25),
            thickness,
        },
    ];
    let down_prims = vec![
        Primitive::Line {
            begin: Vector2::new(1.5, 1.75),
            end: Vector2::new(4.0, 4.25),
            thickness,
        },
        Primitive::Line {
            begin: Vector2::new(4.0, 4.25),
            end: Vector2::new(6.5, 1.75),
            thickness,
        },
    ];

    for (button, is_increase) in buttons {
        if button.is_none() {
            continue;
        }

        // Transparent-until-hover plate with no stroke border.
        let (decorator, content) = {
            let node = &ctx[button];
            match node.cast::<Button>() {
                Some(b) => (*b.decorator, *b.content),
                None => continue,
            }
        };
        if !decorator.is_none() {
            if let Some(d) = ctx[decorator].cast_mut::<Decorator>() {
                // The visible fill is the inner Border's background; clear it
                // now (the brush fields below only apply on state changes).
                d.border
                    .widget
                    .background
                    .set_value_and_mark_modified(transparent.clone().into());
                d.normal_brush
                    .set_value_and_mark_modified(transparent.clone().into());
                d.selected_brush
                    .set_value_and_mark_modified(transparent.clone().into());
                d.hover_brush
                    .set_value_and_mark_modified(hover_plate.clone().into());
                d.pressed_brush
                    .set_value_and_mark_modified(pressed_plate.clone().into());
                d.border
                    .widget
                    .foreground
                    .set_value_and_mark_modified(transparent.clone().into());
            }
        }

        // Swap the filled accent triangle for a thin stroked chevron.
        if !content.is_none() {
            if let Some(arrow) = ctx[content].cast_mut::<VectorImage>() {
                arrow.primitives.set_value_and_mark_modified(
                    if is_increase {
                        up_prims.clone()
                    } else {
                        down_prims.clone()
                    },
                );
                arrow.widget.set_width(8.0);
                arrow.widget.set_height(5.0);
                arrow
                    .widget
                    .foreground
                    .set_value_and_mark_modified(glyph.clone().into());
            }
        }

        // Avalonia's RepeatButton steps while held; fyrox ships the machinery
        // (0.1 s interval) but the numeric builder never enables it.
        ctx.send_message(
            fyrox::gui::message::UiMessage::for_widget(
                button,
                ButtonMessage::RepeatClicksOnHold(true),
            )
            .with_direction(fyrox::gui::message::MessageDirection::ToWidget),
        );
    }
}

/// A handle to a built step input, returned for convenience.
pub type StepInputHandle = Handle<UiNode>;
