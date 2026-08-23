//! The LoadingIndicator component — a self-animating spinner with ten modes.
//!
//! Implemented as a custom fyrox [`Control`] that advances its animation in
//! `update()` and emits immediate-mode geometry in `draw()`. Under
//! `RenderMode::OnChanges` the control forces a redraw every frame by setting
//! `ui.need_render` and invalidating its visual.

use std::f32::consts::{PI, TAU};
use std::ops::Range;

use fyrox::core::algebra::Vector2;
use fyrox::core::color::Color as FyroxColor;
use fyrox::core::math::Rect;
use fyrox::core::pool::Handle;
use fyrox::core::reflect::prelude::*;
use fyrox::core::visitor::prelude::*;
use fyrox::gui::brush::Brush;
use fyrox::gui::draw::{CommandTexture, Draw, DrawingContext};
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::{Widget, WidgetBuilder};
use fyrox::gui::{Control, UiNode, UserInterface};

use raikou_core::{Color, Length, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

/// The animation mode of a [`LoadingIndicator`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect, Visit, Default)]
#[reflect(type_uuid = "41a5034b-4e50-4fdf-b94e-d4a3682d09a0")]
pub enum LoadingIndicatorMode {
    /// A single rotating arc (270° sweep).
    Arc,
    /// Two rotating arcs of different sweeps.
    Arcs,
    /// Three concentric arcs with animated phase.
    ArcsRing,
    /// Two bouncing circles out of phase.
    DoubleBounce,
    /// A rotating, pulsing square.
    FlipPlane,
    /// A pulsing circle (the default; also used inside loading buttons).
    #[default]
    Pulse,
    /// Eight dots pulsing around a circle.
    Ring,
    /// Three bouncing dots.
    ThreeDots,
    /// Five waving bars.
    Wave,
    /// Fluent-style indeterminate bar: an accent block sweeps across a
    /// muted track. Give the indicator a width via `.width(...)`; the
    /// bar thickness follows `stroke_width`.
    Bar,
}

/// fyrox control that renders the spinner animation.
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "fb8407dc-ac74-4737-a249-621c6423235a")]
#[reflect(derived_type = "UiNode")]
pub struct LoadingIndicatorControl {
    widget: Widget,
    mode: LoadingIndicatorMode,
    color: FyroxColor,
    size: f32,
    stroke_width: f32,
    speed_ratio: f32,
    is_active: bool,
    is_visible: bool,
    animation_time: f32,
}

fyrox::gui::define_widget_deref!(LoadingIndicatorControl);

impl LoadingIndicatorControl {
    fn commit_solid(&self, drawing_context: &mut DrawingContext) {
        self.commit_color(self.color, drawing_context);
    }

    fn commit_color(&self, color: FyroxColor, drawing_context: &mut DrawingContext) {
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(color),
            CommandTexture::None,
            &self.material,
            None,
        );
    }

    /// Returns the spinner color with an animated alpha (rgb unchanged).
    fn with_alpha(&self, alpha: f32) -> FyroxColor {
        FyroxColor::from_rgba(
            self.color.r,
            self.color.g,
            self.color.b,
            (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    fn draw_arc_common(
        &self,
        drawing_context: &mut DrawingContext,
        center: Vector2<f32>,
        start_fraction: f32,
        sweep_fraction: f32,
    ) {
        let radius = (self.size / 2.0 - self.stroke_width).max(0.0);
        let rotation = (self.animation_time * 360.0) % 360.0;
        let angles: Range<f32> = ((start_fraction * 360.0 + rotation).to_radians())
            ..((start_fraction + sweep_fraction) * 360.0 + rotation).to_radians();
        drawing_context.push_arc(center, radius, angles, 32, self.stroke_width);
        self.commit_solid(drawing_context);
    }

    fn draw_arcs_ring(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let radius = (self.size / 2.0 - self.stroke_width).max(0.0);
        for index in 0..3 {
            let start = ((self.animation_time * 0.25) + index as f32 * 0.25) % 1.0;
            let angles: Range<f32> =
                (start * 360.0).to_radians()..((start + 0.18) * 360.0).to_radians();
            drawing_context.push_arc(
                center,
                radius - index as f32 * self.stroke_width,
                angles,
                32,
                self.stroke_width,
            );
            self.commit_solid(drawing_context);
        }
    }

    fn draw_double_bounce(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let t = self.animation_time.fract();
        for phase in [0.0_f32, 0.5] {
            let local_t = ((t + phase) % 1.0 * PI).sin().abs();
            drawing_context.push_circle_filled(
                center,
                (self.size * 0.5 * local_t).max(1.0),
                24,
                self.with_alpha(0.25 + 0.5 * local_t),
            );
            self.commit_color(self.color, drawing_context);
        }
    }

    fn draw_flip_plane(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let t = self.animation_time.fract();
        let scale = 0.5 + 0.5 * (t * TAU).cos().abs();
        let size = self.size * scale;
        let rotation = (t * 360.0).to_radians();
        let half = size / 2.0;
        let (cos, sin) = (rotation.cos(), rotation.sin());
        let corner = |offset: Vector2<f32>| {
            Vector2::new(
                center.x + offset.x * cos - offset.y * sin,
                center.y + offset.x * sin + offset.y * cos,
            )
        };
        let corners = [
            corner(Vector2::new(-half, -half)),
            corner(Vector2::new(half, -half)),
            corner(Vector2::new(half, half)),
            corner(Vector2::new(-half, half)),
        ];
        drawing_context.push_triangle_filled([corners[0], corners[1], corners[2]]);
        drawing_context.push_triangle_filled([corners[0], corners[2], corners[3]]);
        self.commit_solid(drawing_context);
    }

    fn draw_pulse(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let radius = self.size / 2.0;
        let t = self.animation_time.fract();
        let scale = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        drawing_context.push_circle_filled(
            center,
            radius * scale,
            24,
            self.with_alpha(0.4 + 0.6 * scale),
        );
        self.commit_color(self.color, drawing_context);
    }

    fn draw_ring(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let radius = self.size / 2.0 - 4.0;
        let dot_count = 8;
        for i in 0..dot_count {
            let angle = (i as f32 / dot_count as f32) * TAU;
            let pos = Vector2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let offset = i as f32 / dot_count as f32;
            let t = (self.animation_time + offset).fract();
            let scale = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
            drawing_context.push_circle_filled(
                pos,
                (self.size / 10.0).max(1.0) * scale,
                12,
                self.with_alpha(0.3 + 0.7 * scale),
            );
            self.commit_color(self.color, drawing_context);
        }
    }

    fn draw_three_dots(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let spacing = self.size / 3.0;
        for i in 0..3 {
            let x = center.x + (i as f32 - 1.0) * spacing * 0.5;
            let offset = i as f32 * 0.15;
            let bounce = (((self.animation_time + offset) % 0.6) / 0.6 * PI)
                .sin()
                .abs();
            let y = center.y - bounce * (self.size / 5.0);
            drawing_context.push_circle_filled(Vector2::new(x, y), self.size / 9.0, 12, self.color);
            self.commit_color(self.color, drawing_context);
        }
    }

    fn draw_wave(&self, drawing_context: &mut DrawingContext, center: Vector2<f32>) {
        let bar_width = self.size / 8.0;
        let spacing = bar_width * 1.5;
        for i in 0..5 {
            let offset = i as f32 * 0.12;
            let wave = (((self.animation_time + offset) % 0.8) / 0.8 * PI)
                .sin()
                .abs();
            let height = self.size * (0.25 + 0.55 * wave);
            let x = center.x + (i as f32 - 2.0) * spacing;
            let y = center.y - height / 2.0;
            drawing_context.push_rect_filled(&Rect::new(x, y, bar_width, height), None);
            self.commit_solid(drawing_context);
        }
    }
    /// Fluent-style indeterminate bar: a muted full-width track with an
    /// accent block sweeping left-to-right on a two-second loop.
    fn draw_bar(&self, drawing_context: &mut DrawingContext) {
        let bounds = self.bounding_rect();
        let w = bounds.w().max(1.0);
        let bar_h = self.stroke_width.max(1.0);
        let y = bounds.h() / 2.0 - bar_h / 2.0;

        // Muted track (Fluent BaseLow is foreground at ~20% alpha).
        drawing_context.push_rect_filled(&Rect::new(0.0, y, w, bar_h), None);
        self.commit_color(self.with_alpha(0.2), drawing_context);

        // Accent block; smoothstep easing gives it the Fluent feel of
        // accelerating away from the left edge and decelerating out.
        let block_w = (w * 0.4).clamp(16.0, 60.0);
        let t = (self.animation_time % 2.0) / 2.0;
        let eased = t * t * (3.0 - 2.0 * t);
        let x = -block_w + eased * (w + block_w);
        drawing_context.push_rounded_rect_filled(
            &Rect::new(x, y, block_w, bar_h),
            bar_h / 2.0,
            8,
        );
        self.commit_solid(drawing_context);
    }
}

/// Accessors used by tests and embedding components.
impl LoadingIndicatorControl {
    /// The current animation mode.
    pub fn mode(&self) -> LoadingIndicatorMode {
        self.mode
    }

    /// The resolved draw color (theme accent when the builder set none).
    pub fn color(&self) -> FyroxColor {
        self.color
    }

    /// Seconds of accumulated animation time.
    pub fn animation_time(&self) -> f32 {
        self.animation_time
    }

    /// Whether the animation clock is running.
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Control for LoadingIndicatorControl {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);
    }

    fn update(&mut self, dt: f32, ui: &mut UserInterface) {
        if self.is_active {
            self.animation_time += dt * self.speed_ratio;
        }
        self.invalidate_visual();
        ui.need_render = true;
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        if !self.is_visible {
            return;
        }

        let bounds = self.bounding_rect();
        let center = Vector2::new(bounds.w() / 2.0, bounds.h() / 2.0);

        match self.mode {
            LoadingIndicatorMode::Arc => {
                self.draw_arc_common(drawing_context, center, 0.0, 0.75);
            }
            LoadingIndicatorMode::Arcs => {
                self.draw_arc_common(drawing_context, center, 0.0, 0.35);
                self.draw_arc_common(drawing_context, center, 0.5, 0.25);
            }
            LoadingIndicatorMode::ArcsRing => self.draw_arcs_ring(drawing_context, center),
            LoadingIndicatorMode::DoubleBounce => self.draw_double_bounce(drawing_context, center),
            LoadingIndicatorMode::FlipPlane => self.draw_flip_plane(drawing_context, center),
            LoadingIndicatorMode::Pulse => self.draw_pulse(drawing_context, center),
            LoadingIndicatorMode::Ring => self.draw_ring(drawing_context, center),
            LoadingIndicatorMode::ThreeDots => self.draw_three_dots(drawing_context, center),
            LoadingIndicatorMode::Wave => self.draw_wave(drawing_context, center),
            LoadingIndicatorMode::Bar => self.draw_bar(drawing_context),
        }
    }
}

/// Builder for a [`crate::LoadingIndicator`] component.
///
/// ```rust,ignore
/// let spinner = LoadingIndicator::new()
///     .mode(LoadingIndicatorMode::Ring)
///     .color(theme.color("text.primary").unwrap())
///     .size(32.0)
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct LoadingIndicator {
    id: String,
    mode: LoadingIndicatorMode,
    color: Option<Color>,
    size: f32,
    stroke_width: f32,
    speed_ratio: f32,
    is_active: bool,
    is_visible: bool,
    width: Length,
    height: Length,
    padding: Thickness,
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadingIndicator {
    /// Creates a new loading indicator builder.
    ///
    /// The color defaults to the active theme's `accent.solid` token
    /// (Fluent indicators are monochrome accent); `.color()` overrides it.
    pub fn new() -> Self {
        Self {
            id: String::new(),
            mode: LoadingIndicatorMode::Pulse,
            color: None,
            size: 24.0,
            stroke_width: 2.0,
            speed_ratio: 1.0,
            is_active: true,
            is_visible: true,
            width: Length::Fixed(24.0),
            height: Length::Fixed(24.0),
            padding: Thickness::ZERO,
        }
    }

    /// Sets the identifier of the indicator (used as the widget name).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the animation mode.
    pub fn mode(mut self, mode: LoadingIndicatorMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the spinner color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the size of the spinner in logical pixels (clamped to >= 1.0).
    ///
    /// Also resizes the width/height when they are still fixed lengths.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        if matches!(self.width, Length::Fixed(_)) {
            self.width = Length::Fixed(self.size);
        }
        if matches!(self.height, Length::Fixed(_)) {
            self.height = Length::Fixed(self.size);
        }
        self
    }

    /// Sets the stroke width of arc-based modes (clamped to >= 1.0).
    pub fn stroke_width(mut self, stroke_width: f32) -> Self {
        self.stroke_width = stroke_width.max(1.0);
        self
    }

    /// Sets the animation speed multiplier (clamped to 0.1..=5.0).
    pub fn speed_ratio(mut self, ratio: f32) -> Self {
        self.speed_ratio = ratio.clamp(0.1, 5.0);
        self
    }

    /// Pauses/resumes the animation. When inactive the last frame is kept.
    pub fn is_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Controls whether the indicator is rendered at all.
    pub fn is_visible(mut self, visible: bool) -> Self {
        self.is_visible = visible;
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

    /// Sets the space around the spinner.
    pub fn padding(mut self, padding: Thickness) -> Self {
        self.padding = padding;
        self
    }

    /// Builds the loading indicator, adds it to the UI and registers it.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let handle = Self::build_node(cx, self);
        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }

    /// Builds only the fyrox node, without registering a component. Used
    /// internally by components that embed a spinner (e.g. a loading button).
    pub(crate) fn build_node(cx: &mut BuildCx, builder: LoadingIndicator) -> Handle<UiNode> {
        let width = builder.width.resolve().unwrap_or(builder.size);
        let height = builder.height.resolve().unwrap_or(builder.size);
        let name = if builder.id.is_empty() {
            "raikou_loading_indicator"
        } else {
            &builder.id
        };
        // Fluent indicators are monochrome accent: fall back to the theme's
        // accent token when the builder did not pin an explicit color.
        let color = builder.color.unwrap_or_else(|| {
            cx.theme()
                .color("accent.solid")
                .unwrap_or(Color::new(0.13, 0.39, 0.94, 1.0))
        });
        let control = LoadingIndicatorControl {
            widget: WidgetBuilder::new()
                .with_name(name)
                .with_need_update(true)
                .with_width(width)
                .with_height(height)
                .with_margin(to_fyrox_thickness(builder.padding))
                .build(&cx.ctx()),
            mode: builder.mode,
            color: to_fyrox_color(color),
            size: builder.size,
            stroke_width: builder.stroke_width,
            speed_ratio: builder.speed_ratio,
            is_active: builder.is_active,
            is_visible: builder.is_visible,
            animation_time: 0.0,
        };
        cx.ctx().add(control).transmute()
    }
}

/// A handle to a built loading indicator.
pub type LoadingIndicatorHandle = Handle<UiNode>;
