//! The Slider component.
//!
//! Maps onto fyrox's `ScrollBarBuilder` (the documented slider in this fyrox
//! version) and reports value changes through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
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
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
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
        use fyrox::graph::SceneGraph;

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

        // Fluent restyle: thin track line + round thumb, no arrow buttons.
        {
            use fyrox::gui::border::Border;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::DecoratorMessage;
            use fyrox::gui::scroll_bar::ScrollBar;
            use fyrox::gui::widget::WidgetMessage;
            use fyrox::gui::{HorizontalAlignment, VerticalAlignment};
            use fyrox::gui::Thickness as FyroxThickness;
            use crate::convert::to_fyrox_color;
            use raikou_core::Color as RaikouColor;

            let theme = cx.theme().clone();
            let token = |name: &str, fallback: RaikouColor| {
                theme.color(name).unwrap_or(fallback)
            };
            let track_fill = token(
                "border.default",
                RaikouColor::new(0.0, 0.0, 0.0, 0.4),
            );
            let thumb_fill = token("text.primary", RaikouColor::new(0.0, 0.0, 0.0, 1.0));
            let pressed_fill = token("accent.solid", RaikouColor::new(0.0, 0.47, 0.84, 1.0));

            let (increase, decrease, indicator) = {
                let ui = cx.ui();
                match ui.try_get_of_type::<ScrollBar>(handle) {
                    Ok(sb) => (*sb.increase, *sb.decrease, *sb.indicator),
                    Err(_) => Default::default(),
                }
            };
            let ui = cx.ui();
            ui.send(increase, WidgetMessage::Visibility(false));
            ui.send(decrease, WidgetMessage::Visibility(false));

            // Thumb: 16x16 circle riding the track.
            ui.send(indicator, WidgetMessage::Width(16.0));
            ui.send(indicator, WidgetMessage::Height(16.0));
            if let Ok(border) = ui.try_get_mut_of_type::<Border>(indicator) {
                *border.corner_radius = 8.0f32.into();
                *border.stroke_thickness = FyroxThickness::uniform(0.0).into();
            }
            ui.send(
                indicator,
                DecoratorMessage::NormalBrush(Brush::Solid(to_fyrox_color(thumb_fill)).into()),
            );
            ui.send(
                indicator,
                DecoratorMessage::HoverBrush(Brush::Solid(to_fyrox_color(thumb_fill)).into()),
            );
            ui.send(
                indicator,
                DecoratorMessage::PressedBrush(Brush::Solid(to_fyrox_color(pressed_fill)).into()),
            );

            // Track: thin centered line.
            if let Some(body) = ui.node(handle).children().first().copied() {
                if let Ok(border) = ui.try_get_mut_of_type::<Border>(body) {
                    *border.stroke_thickness = FyroxThickness::uniform(0.0).into();
                    *border.background =
                        Brush::Solid(to_fyrox_color(track_fill)).into();
                }
                match self.orientation {
                    Orientation::Horizontal => {
                        ui.send(body, WidgetMessage::Height(4.0));
                        ui.send(body, WidgetMessage::VerticalAlignment(VerticalAlignment::Center));
                    }
                    Orientation::Vertical => {
                        ui.send(body, WidgetMessage::Width(4.0));
                        ui.send(body, WidgetMessage::HorizontalAlignment(HorizontalAlignment::Center));
                    }
                }
            }
        }

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
