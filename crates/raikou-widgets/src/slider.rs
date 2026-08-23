//! The Slider component.
//!
//! Maps onto fyrox's `ScrollBarBuilder` (the documented slider in this fyrox
//! version) and reports value changes through an `on_change` handler.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{KeyCode, MessageDirection, MouseButton, UiMessage};
use fyrox::gui::scroll_bar::{ScrollBarBuilder, ScrollBarMessage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
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
    pub(crate) handle: Handle<UiNode>,
    min: f32,
    max: f32,
    /// Snap increment; `0.0` disables snapping.
    step: f32,
    body: Handle<UiNode>,
    fill: Handle<UiNode>,
    /// Last reported value; dedupes the synthetic initial echo.
    last_value: std::cell::Cell<f32>,
    /// Set while a snap-correction echo is in flight so the corrected value
    /// is accepted as-is (guards ranges whose bounds are not step multiples).
    correcting: std::cell::Cell<bool>,
}

impl SliderHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(ScrollBarMessage::Value(value)) = message.data::<ScrollBarMessage>() {
            // Snap off-grid values back onto the step lattice (Avalonia
            // sliders quantize every committed value). The correction echo
            // bypasses re-snapping once.
            if self.correcting.get() {
                self.correcting.set(false);
            } else if let Some(snapped) = self.snap(*value) {
                self.correcting.set(true);
                ui.send(self.handle, ScrollBarMessage::Value(snapped));
                return;
            }
            self.sync_fill(ui, *value);
            if self.last_value.get() != *value {
                self.last_value.set(*value);
                if let Some(callback) = &self.on_change {
                    callback(ui, *value);
                }
            }
        }
    }

    /// Rounds `value` onto the step lattice; `None` when already aligned or
    /// when stepping is disabled.
    fn snap(&self, value: f32) -> Option<f32> {
        if self.step <= 0.0 {
            return None;
        }
        let snapped = self.min + ((value - self.min) / self.step).round() * self.step;
        let snapped = snapped.clamp(self.min, self.max);
        (snapped != value).then_some(snapped)
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

/// Global watcher that jumps a slider to the clicked track position
/// (Fluent track-click behavior). Thumb presses keep their native drag.
#[derive(Clone)]
pub struct SliderJump {
    /// The slider widget.
    pub(crate) target: Handle<UiNode>,
    /// The thin track line whose bounds map position → value.
    pub(crate) body: Handle<UiNode>,
    /// The thumb; clicks on it are ignored so native dragging works.
    pub(crate) indicator: Handle<UiNode>,
    pub(crate) orientation: Orientation,
    pub(crate) min: f32,
    pub(crate) max: f32,
}

impl SliderJump {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        use fyrox::graph::SceneGraph;

        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(WidgetMessage::MouseDown { button: MouseButton::Left, pos, .. }) =
            message.data::<WidgetMessage>()
        else {
            return;
        };
        if !crate::component::is_in_subtree(ui, message.destination(), self.target)
            || crate::component::is_in_subtree(ui, message.destination(), self.indicator)
            || self.body.is_none()
        {
            return;
        }
        let bounds = ui.node(self.body).screen_bounds();
        let pct = match self.orientation {
            Orientation::Horizontal => (pos.x - bounds.position.x) / bounds.size.x,
            Orientation::Vertical => (pos.y - bounds.position.y) / bounds.size.y,
        }
        .clamp(0.0, 1.0);
        let value = self.min + pct * (self.max - self.min);
        ui.send(self.target, ScrollBarMessage::Value(value));
    }
}

/// Global key watcher giving sliders Avalonia's arrow/Home/End navigation
/// (fyrox scroll bars have no keyboard handling of their own).
#[derive(Clone)]
pub struct SliderNav {
    /// The slider widget.
    pub(crate) target: Handle<UiNode>,
    pub(crate) step: f32,
    pub(crate) min: f32,
    pub(crate) max: f32,
}

impl SliderNav {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        use fyrox::graph::SceneGraph;
        use fyrox::gui::scroll_bar::ScrollBar;

        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(WidgetMessage::KeyDown(key)) = message.data::<WidgetMessage>() else {
            return;
        };
        if !crate::component::is_in_subtree(ui, message.destination(), self.target)
            || crate::tabs::in_text_entry(ui, message.destination())
        {
            return;
        }
        let delta = match key {
            KeyCode::ArrowRight | KeyCode::ArrowUp => Some(self.step),
            KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-self.step),
            _ => None,
        };
        let Some(current) = ui
            .try_get_of_type::<ScrollBar>(self.target)
            .ok()
            .map(|bar| *bar.value)
        else {
            return;
        };
        let next = if let Some(delta) = delta {
            current + delta
        } else {
            match key {
                KeyCode::Home => self.min,
                KeyCode::End => self.max,
                _ => return,
            }
        };
        ui.send(
            self.target,
            ScrollBarMessage::Value(next.clamp(self.min, self.max)),
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
        let (increase, decrease, indicator) = {
            use fyrox::gui::scroll_bar::ScrollBar;

            let ui = cx.ui();
            match ui.try_get_of_type::<ScrollBar>(handle) {
                Ok(sb) => (*sb.increase, *sb.decrease, *sb.indicator),
                Err(_) => Default::default(),
            }
        };
        let mut fill_handle = Handle::NONE;
        let mut body_handle = Handle::NONE;
        {
            use crate::convert::to_fyrox_color;
            use fyrox::gui::border::Border;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::DecoratorMessage;
            use fyrox::gui::widget::WidgetMessage;
            use fyrox::gui::Thickness as FyroxThickness;
            use fyrox::gui::{HorizontalAlignment, VerticalAlignment};
            use raikou_core::Color as RaikouColor;

            let theme = cx.theme().clone();
            let token = |name: &str, fallback: RaikouColor| theme.color(name).unwrap_or(fallback);
            let track_fill = token("border.default", RaikouColor::new(0.0, 0.0, 0.0, 0.4));
            let thumb_fill = token("text.primary", RaikouColor::new(0.0, 0.0, 0.0, 1.0));
            let pressed_fill = token("accent.solid", RaikouColor::new(0.0, 0.47, 0.84, 1.0));

            // Scope the shared `ui` reborrow here: the fill element below
            // needs fresh `cx.ctx()`/`cx.ui()` calls and NLL keeps the old
            // borrow alive otherwise.
            {
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
            }

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
                on_change: self.on_change.clone(),
                handle,
                min: self.min,
                max: self.max,
                step: self.step,
                body: body_handle,
                fill: fill_handle,
                last_value: std::cell::Cell::new(self.value),
                correcting: std::cell::Cell::new(false),
            }),
        };
        cx.register(&component);

        // Track-click jumps + keyboard navigation ride global watchers:
        // presses and focus land on deep children of the slider, which
        // exact-path dispatch never sees.
        cx.register_global(&Component {
            handle,
            kind: ComponentKind::SliderJump(SliderJump {
                target: handle,
                body: body_handle,
                indicator,
                orientation: self.orientation,
                min: self.min,
                max: self.max,
            }),
        });
        cx.register_global(&Component {
            handle,
            kind: ComponentKind::SliderNav(SliderNav {
                target: handle,
                step: self.step,
                min: self.min,
                max: self.max,
            }),
        });

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
