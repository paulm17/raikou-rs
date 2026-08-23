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
    /// Range + layout handles for the Fluent fill-before-thumb element.
    min: f32,
    max: f32,
    body: Handle<UiNode>,
    fill: Handle<UiNode>,
    /// Last reported value; dedupes the synthetic initial echo.
    last_value: std::cell::Cell<f32>,
}

impl SliderHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(ScrollBarMessage::Value(value)) = message.data::<ScrollBarMessage>() {
            self.sync_fill(ui, *value);
            if self.last_value.get() != *value {
                self.last_value.set(*value);
                if let Some(callback) = &self.on_change {
                    callback(ui, *value);
                }
            }
        }
    }

    /// Sizes the accent fill before the thumb (Fluent slider look).
    fn sync_fill(&self, ui: &mut UserInterface, value: f32) {
        use fyrox::graph::SceneGraph;

        if self.fill.is_none() || self.body.is_none() {
            return;
        }
        let span = (self.max - self.min).max(f32::EPSILON);
        let pct = ((value - self.min) / span).clamp(0.0, 1.0);
        let track_w = ui.node(self.body).actual_local_size().x;
        ui.send(
            self.fill,
            fyrox::gui::widget::WidgetMessage::Width(pct * track_w),
        );
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
        let mut fill_handle = Handle::NONE;
        let mut body_handle = Handle::NONE;
        {
            use crate::convert::to_fyrox_color;
            use fyrox::gui::border::Border;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::DecoratorMessage;
            use fyrox::gui::scroll_bar::ScrollBar;
            use fyrox::gui::widget::WidgetMessage;
            use fyrox::gui::Thickness as FyroxThickness;
            use fyrox::gui::{HorizontalAlignment, VerticalAlignment};
            use raikou_core::Color as RaikouColor;

            let theme = cx.theme().clone();
            let token = |name: &str, fallback: RaikouColor| theme.color(name).unwrap_or(fallback);
            let track_fill = token("border.default", RaikouColor::new(0.0, 0.0, 0.0, 0.4));
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
            // End the shared `ui` reborrow here: the fill element below needs
            // fresh `cx.ctx()`/`cx.ui()` calls and NLL keeps the old borrow
            // alive otherwise.
            drop(ui);

            // Track: thin centered line.
            let body = cx.ui().node(handle).children().first().copied();
            if let Some(body) = body {
                let ui = cx.ui();
                if let Ok(border) = ui.try_get_mut_of_type::<Border>(body) {
                    *border.stroke_thickness = FyroxThickness::uniform(0.0).into();
                    *border.background = Brush::Solid(to_fyrox_color(track_fill)).into();
                }
                let (alignment_msg, size_msg) = match self.orientation {
                    Orientation::Horizontal => (
                        WidgetMessage::VerticalAlignment(VerticalAlignment::Center),
                        WidgetMessage::Height(4.0),
                    ),
                    Orientation::Vertical => (
                        WidgetMessage::HorizontalAlignment(HorizontalAlignment::Center),
                        WidgetMessage::Width(4.0),
                    ),
                };
                ui.send(body, size_msg);
                ui.send(body, alignment_msg.clone());
                ui.send(indicator, alignment_msg);
            }

            // Fluent accent fill before the thumb: a child of the track
            // body, sized proportionally on every value change.
            if let Some(body) = body {
                use fyrox::gui::border::BorderBuilder;
                let fill_accent = token("accent.solid", RaikouColor::new(0.0, 0.47, 0.84, 1.0));
                let fill = {
                    let mut ctx = cx.ctx();
                    BorderBuilder::new(
                        WidgetBuilder::new()
                            .with_name("raikou_slider_fill")
                            .with_vertical_alignment(VerticalAlignment::Center)
                            .with_horizontal_alignment(HorizontalAlignment::Left)
                            .with_height(4.0)
                            .with_width(0.0)
                            .with_background(Brush::Solid(to_fyrox_color(fill_accent)).into()),
                    )
                    .with_corner_radius(2.0.into())
                    .with_stroke_thickness(FyroxThickness::uniform(0.0).into())
                    .build(&mut ctx)
                    .to_base()
                };
                {
                    let mut ctx = cx.ctx();
                    ctx.link(fill, body);
                }
                fill_handle = fill;
                body_handle = body;
            }
        }

        let component = Component {
            handle,
            kind: ComponentKind::Slider(SliderHandlers {
                on_change: self.on_change,
                min: self.min,
                max: self.max,
                body: body_handle,
                fill: fill_handle,
                last_value: std::cell::Cell::new(self.value),
            }),
        };
        cx.register(&component);

        // Size the accent fill for the initial value: a synthetic FromWidget
        // echo routes through our own dispatcher on the next pump (the
        // last_value dedupe keeps on_change silent for it).
        cx.ui().send_message(
            UiMessage::with_data(ScrollBarMessage::Value(self.value))
                .with_destination(handle)
                .with_direction(MessageDirection::FromWidget),
        );

        component
    }
}

/// A handle to a built slider, returned for convenience.
pub type SliderHandle = Handle<UiNode>;
