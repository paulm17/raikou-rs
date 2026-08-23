//! raikou-playground — interactive component playground, ported from the
//! raikou-rs reference project.
//!
//! This crate provides the building blocks for a full-screen playground app:
//!
//! - [`PlaygroundShell`] — a three-pane layout with a live preview area (left),
//!   a scrollable controls sidebar (right) and a code panel (bottom).
//! - [`PlaygroundPreview`] — a rounded "stage" card that centres a child widget
//!   with an optional content size cap.
//! - [`PlaygroundCodePanel`] — a titled card containing a scrollable code block.
//! - [`PlaygroundCodeBlock`] — a code text node whose contents are regenerated
//!   from a closure via [`update_code`].
//! - [`control_slider`] — a pre-tuned [`Slider`] builder for control palettes.
//! - [`playground_notes`] — a `Stack` of a heading plus explanatory notes.
//!
//! All three layout pieces are implemented as custom fyrox `Control` widgets so
//! they can position their children at exact rectangles (the same approach the
//! fyrox `Canvas` widget uses via `measure_node`/`arrange_node`).

use std::rc::Rc;

use fyrox::core::algebra::Vector2;
use fyrox::core::color::Color as FyroxColor;
use fyrox::core::math::Rect;
use fyrox::core::pool::Handle;
use fyrox::core::reflect::prelude::*;
use fyrox::core::visitor::prelude::*;
use fyrox::gui::brush::Brush;
use fyrox::gui::draw::{CommandTexture, Draw, DrawingContext};
use fyrox::gui::message::UiMessage;
use fyrox::gui::scroll_viewer::ScrollViewerBuilder;
use fyrox::gui::text::TextBuilder;
use fyrox::gui::widget::{Widget, WidgetBuilder};
use fyrox::gui::{Control, UiNode, UserInterface};

use raikou::prelude::*;
use raikou::{to_fyrox_color, Color};
use raikou_style::Theme;

/// Caps a size to a finite working bound. Root canvases measure their children
/// with an unbounded available size; using it directly would propagate infinite
/// sizes through layout and produce no visible geometry.
fn bounded(v: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(
        if v.x.is_finite() { v.x } else { 4096.0 },
        if v.y.is_finite() { v.y } else { 4096.0 },
    )
}

/// Resolves a theme color token, falling back to `fallback` when missing.
fn token(theme: &Theme, name: &str, fallback: Color) -> FyroxColor {
    to_fyrox_color(theme.color(name).unwrap_or(fallback))
}

// ---------------------------------------------------------------------------
// PlaygroundShell
// ---------------------------------------------------------------------------

/// A custom fyrox control that splits its bounds into a preview card (left),
/// a controls card (right) and a full-width code card (bottom).
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "f4a1b2c3-d4e5-4f6a-9b8c-1d2e3f4a5b6c")]
#[reflect(derived_type = "UiNode")]
pub struct PlaygroundShellControl {
    widget: Widget,
    preview: Handle<UiNode>,
    controls: Handle<UiNode>,
    code: Handle<UiNode>,
    sidebar_width: f32,
    code_height: f32,
    outer_padding: f32,
    gap: f32,
    page_bg: FyroxColor,
    card_fill: FyroxColor,
    card_border: FyroxColor,
}

fyrox::gui::define_widget_deref!(PlaygroundShellControl);

impl PlaygroundShellControl {
    fn usable_width(&self, total: f32) -> f32 {
        (self.finite(total) - self.outer_padding * 2.0).max(0.0)
    }

    fn top_height(&self, total: f32) -> f32 {
        (self.finite(total) - self.outer_padding * 2.0 - self.gap - self.code_height).max(240.0)
    }

    fn preview_width(&self, usable_w: f32) -> f32 {
        (usable_w - self.sidebar_width - self.gap).max(320.0)
    }

    /// Root canvases measure their children with an unbounded available size;
    /// cap it so geometry stays finite even if the shell is not explicitly
    /// sized by its parent.
    fn finite(&self, value: f32) -> f32 {
        if value.is_finite() {
            value
        } else {
            4096.0
        }
    }
}

impl Control for PlaygroundShellControl {
    fn measure_override(&self, ui: &UserInterface, available_size: Vector2<f32>) -> Vector2<f32> {
        let usable_w = self.usable_width(available_size.x);
        let top_h = self.top_height(available_size.y);
        let preview_w = self.preview_width(usable_w);

        ui.measure_node(self.preview, Vector2::new(preview_w, top_h));
        ui.measure_node(
            self.controls,
            Vector2::new(self.sidebar_width - 32.0, top_h - 32.0),
        );
        ui.measure_node(self.code, Vector2::new(usable_w, self.code_height));

        Vector2::new(self.finite(available_size.x), self.finite(available_size.y))
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vector2<f32>) -> Vector2<f32> {
        let usable_w = self.usable_width(final_size.x);
        let top_h = self.top_height(final_size.y);
        let preview_w = self.preview_width(usable_w);

        let preview_card = Rect::new(self.outer_padding, self.outer_padding, preview_w, top_h);
        let controls_card = Rect::new(
            preview_card.position.x + preview_card.size.x + self.gap,
            self.outer_padding,
            self.sidebar_width,
            top_h,
        );
        let code_rect = Rect::new(
            self.outer_padding,
            self.outer_padding + top_h + self.gap,
            usable_w,
            self.code_height,
        );

        ui.arrange_node(self.preview, &preview_card);
        ui.arrange_node(
            self.controls,
            &Rect::new(
                controls_card.position.x + 16.0,
                controls_card.position.y + 18.0,
                controls_card.size.x - 32.0,
                controls_card.size.y - 36.0,
            ),
        );
        ui.arrange_node(self.code, &code_rect);

        final_size
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        let bounds = self.widget.bounding_rect();

        // Page background.
        drawing_context.push_rect_filled(&bounds, None);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.page_bg),
            CommandTexture::None,
            &self.material,
            None,
        );

        let usable_w = self.usable_width(bounds.size.x);
        let top_h = self.top_height(bounds.size.y);
        let preview_w = self.preview_width(usable_w);

        let preview_card = Rect::new(self.outer_padding, self.outer_padding, preview_w, top_h);
        let controls_card = Rect::new(
            preview_card.position.x + preview_card.size.x + self.gap,
            self.outer_padding,
            self.sidebar_width,
            top_h,
        );

        for card in [preview_card, controls_card] {
            drawing_context.push_rounded_rect_filled(&card, 8.0, 12);
            drawing_context.commit(
                self.clip_bounds(),
                Brush::Solid(self.card_fill),
                CommandTexture::None,
                &self.material,
                None,
            );
            drawing_context.push_rounded_rect(&card, 1.0, 8.0, 12);
            drawing_context.commit(
                self.clip_bounds(),
                Brush::Solid(self.card_border),
                CommandTexture::None,
                &self.material,
                None,
            );
        }
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);
    }
}

/// Builder for a [`PlaygroundShellControl`].
#[derive(Clone)]
pub struct PlaygroundShell {
    preview: Handle<UiNode>,
    controls: Handle<UiNode>,
    code: Handle<UiNode>,
    sidebar_width: f32,
    code_height: f32,
    outer_padding: f32,
    gap: f32,
}

impl Default for PlaygroundShell {
    fn default() -> Self {
        Self::new(Handle::NONE, Handle::NONE, Handle::NONE)
    }
}

impl PlaygroundShell {
    /// Creates a new shell with the given preview, controls and code content.
    ///
    /// The controls content is wrapped in a vertical scroll area automatically.
    pub fn new(
        preview: impl Into<Handle<UiNode>>,
        controls: impl Into<Handle<UiNode>>,
        code: impl Into<Handle<UiNode>>,
    ) -> Self {
        Self {
            preview: preview.into(),
            controls: controls.into(),
            code: code.into(),
            sidebar_width: 280.0,
            code_height: 280.0,
            outer_padding: 24.0,
            gap: 14.0,
        }
    }

    /// Sets the controls sidebar width (default 280, clamped to >= 180).
    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width.max(180.0);
        self
    }

    /// Sets the code panel height (default 280, clamped to >= 120).
    pub fn code_height(mut self, height: f32) -> Self {
        self.code_height = height.max(120.0);
        self
    }

    /// Sets the outer padding around the whole layout (default 24).
    pub fn outer_padding(mut self, padding: f32) -> Self {
        self.outer_padding = padding.max(0.0);
        self
    }

    /// Sets the gap between the cards (default 14).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    /// Builds the shell, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();
        let page_bg = token(&theme, "surface.panel", Color::new(0.95, 0.95, 0.95, 1.0));
        let card_fill = token(&theme, "surface.elevated", Color::new(1.0, 1.0, 1.0, 1.0));
        let card_border = token(&theme, "border.subtle", Color::new(0.0, 0.0, 0.0, 0.14));

        let controls: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            ScrollViewerBuilder::new(WidgetBuilder::new())
                .with_content(self.controls)
                .with_vertical_scroll_allowed(true)
                .with_horizontal_scroll_allowed(false)
                .with_v_scroll_speed(48.0)
                .build(&mut ctx)
                .to_base()
        };

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let control = PlaygroundShellControl {
                widget: WidgetBuilder::new()
                    .with_name("raikou_playground_shell")
                    .with_children(vec![self.preview, controls, self.code])
                    .build(&ctx),
                preview: self.preview,
                controls,
                code: self.code,
                sidebar_width: self.sidebar_width,
                code_height: self.code_height,
                outer_padding: self.outer_padding,
                gap: self.gap,
                page_bg,
                card_fill,
                card_border,
            };
            ctx.add(control).transmute()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built playground shell.
pub type PlaygroundShellHandle = Handle<UiNode>;

// ---------------------------------------------------------------------------
// PlaygroundPreview
// ---------------------------------------------------------------------------

/// A custom fyrox control that draws a rounded "stage" card and centres its
/// child inside the card with an optional content size cap.
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d")]
#[reflect(derived_type = "UiNode")]
pub struct PlaygroundPreviewControl {
    widget: Widget,
    child: Handle<UiNode>,
    padding: f32,
    max_width: Option<f32>,
    max_height: Option<f32>,
    stage_color: FyroxColor,
    border_color: FyroxColor,
    radius: f32,
}

fyrox::gui::define_widget_deref!(PlaygroundPreviewControl);

impl Control for PlaygroundPreviewControl {
    fn measure_override(&self, ui: &UserInterface, available_size: Vector2<f32>) -> Vector2<f32> {
        let available_size = bounded(available_size);
        let inner_width = (available_size.x - self.padding * 2.0).max(0.0);
        let inner_height = (available_size.y - self.padding * 2.0).max(0.0);
        let child_width = self
            .max_width
            .unwrap_or(inner_width)
            .min(inner_width)
            .max(1.0);
        let child_height = self
            .max_height
            .unwrap_or(inner_height)
            .min(inner_height)
            .max(1.0);
        ui.measure_node(self.child, Vector2::new(child_width, child_height));
        // Hug the measured content instead of claiming all available space.
        let desired = ui.nodes().borrow(self.child).desired_size();
        Vector2::new(
            (desired.x + self.padding * 2.0).min(available_size.x),
            (desired.y + self.padding * 2.0).min(available_size.y),
        )
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vector2<f32>) -> Vector2<f32> {
        let final_size = bounded(final_size);
        let inner_width = (final_size.x - self.padding * 2.0).max(0.0);
        let inner_height = (final_size.y - self.padding * 2.0).max(0.0);
        let width_limit = self.max_width.unwrap_or(inner_width).min(inner_width);
        let height_limit = self.max_height.unwrap_or(inner_height).min(inner_height);

        let desired = ui.nodes().borrow(self.child).desired_size();
        let child_width = desired.x.min(width_limit);
        let child_height = desired.y.min(height_limit);
        let child_x = self.padding + (inner_width - child_width).max(0.0) * 0.5;
        let child_y = self.padding + (inner_height - child_height).max(0.0) * 0.5;

        ui.arrange_node(
            self.child,
            &Rect::new(child_x, child_y, child_width, child_height),
        );

        final_size
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        let bounds = self.widget.bounding_rect();

        drawing_context.push_rounded_rect_filled(&bounds, self.radius, 16);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.stage_color),
            CommandTexture::None,
            &self.material,
            None,
        );
        drawing_context.push_rounded_rect(&bounds, 1.0, self.radius, 16);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.border_color),
            CommandTexture::None,
            &self.material,
            None,
        );
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);
    }
}

/// Builder for a [`PlaygroundPreviewControl`].
#[derive(Clone)]
pub struct PlaygroundPreview {
    child: Handle<UiNode>,
    padding: f32,
    max_width: Option<f32>,
    max_height: Option<f32>,
    stage_color: FyroxColor,
    border_color: FyroxColor,
    radius: f32,
    stage_color_explicit: bool,
    border_color_explicit: bool,
}

impl PlaygroundPreview {
    /// Creates a new preview stage around the given content.
    pub fn new(child: impl Into<Handle<UiNode>>) -> Self {
        Self {
            child: child.into(),
            padding: 32.0,
            max_width: None,
            max_height: None,
            stage_color: to_fyrox_color(Color::new(0.97, 0.98, 1.0, 1.0)),
            border_color: to_fyrox_color(Color::new(0.86, 0.88, 0.92, 1.0)),
            radius: 8.0,
            stage_color_explicit: false,
            border_color_explicit: false,
        }
    }

    /// Sets the padding between the card edge and the content (default 32).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
        self
    }

    /// Caps the maximum size of the centred content.
    pub fn content_max_size(mut self, width: f32, height: f32) -> Self {
        self.max_width = Some(width.max(1.0));
        self.max_height = Some(height.max(1.0));
        self
    }

    /// Sets the stage card fill color.
    pub fn stage_color(mut self, color: impl Into<Color>) -> Self {
        self.stage_color = to_fyrox_color(color.into());
        self.stage_color_explicit = true;
        self
    }

    /// Sets the stage card border color.
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = to_fyrox_color(color.into());
        self.border_color_explicit = true;
        self
    }

    /// Sets the stage card corner radius (default 8).
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    /// Builds the preview stage, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();
        let stage_color = if self.stage_color_explicit {
            self.stage_color
        } else {
            token(&theme, "surface.elevated", Color::new(1.0, 1.0, 1.0, 1.0))
        };
        let border_color = if self.border_color_explicit {
            self.border_color
        } else {
            token(
                &theme,
                "fluent.transient.border",
                Color::new(0.0, 0.0, 0.0, 0.14),
            )
        };

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let control = PlaygroundPreviewControl {
                widget: WidgetBuilder::new()
                    .with_name("raikou_playground_preview")
                    .with_child(self.child)
                    .build(&ctx),
                child: self.child,
                padding: self.padding,
                max_width: self.max_width,
                max_height: self.max_height,
                stage_color,
                border_color,
                radius: self.radius.min(8.0),
            };
            ctx.add(control).transmute()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built preview stage.
pub type PlaygroundPreviewHandle = Handle<UiNode>;

// ---------------------------------------------------------------------------
// PlaygroundCodePanel / PlaygroundCodeBlock
// ---------------------------------------------------------------------------

/// A custom fyrox control that draws a titled card with a divider and hosts a
/// scrollable code area below the title.
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "1a2b3c4d-5e6f-4a8b-9c0d-1e2f3a4b5c6d")]
#[reflect(derived_type = "UiNode")]
pub struct PlaygroundCodePanelControl {
    widget: Widget,
    title_text: Handle<UiNode>,
    scroll: Handle<UiNode>,
    height: f32,
    card_fill: FyroxColor,
    card_border: FyroxColor,
    divider: FyroxColor,
    pill_fill: FyroxColor,
    pill_border: FyroxColor,
}

fyrox::gui::define_widget_deref!(PlaygroundCodePanelControl);

impl Control for PlaygroundCodePanelControl {
    fn measure_override(&self, ui: &UserInterface, available_size: Vector2<f32>) -> Vector2<f32> {
        let available_size = bounded(available_size);
        let panel_height = self.height.min(available_size.y);
        let body_height = (panel_height - 69.0).max(0.0);
        ui.measure_node(self.title_text, Vector2::new(104.0, 22.0));
        ui.measure_node(
            self.scroll,
            Vector2::new((available_size.x - 40.0).max(1.0), body_height),
        );
        Vector2::new(available_size.x, panel_height)
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vector2<f32>) -> Vector2<f32> {
        let final_size = bounded(final_size);
        let body_height = (final_size.y - 69.0).max(0.0);
        ui.arrange_node(self.title_text, &Rect::new(28.0, 15.0, 104.0, 22.0));
        ui.arrange_node(
            self.scroll,
            &Rect::new(20.0, 59.0, (final_size.x - 40.0).max(1.0), body_height),
        );
        final_size
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        let bounds = self.widget.bounding_rect();

        // Card background + border.
        drawing_context.push_rounded_rect_filled(&bounds, 8.0, 12);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.card_fill),
            CommandTexture::None,
            &self.material,
            None,
        );
        drawing_context.push_rounded_rect(&bounds, 1.0, 8.0, 12);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.card_border),
            CommandTexture::None,
            &self.material,
            None,
        );

        // Divider under the title row.
        drawing_context.push_rect_filled(&Rect::new(0.0, 48.0, bounds.size.x, 1.0), None);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.divider),
            CommandTexture::None,
            &self.material,
            None,
        );

        // Title pill.
        let pill = Rect::new(16.0, 12.0, 120.0, 28.0);
        drawing_context.push_rounded_rect_filled(&pill, 6.0, 12);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.pill_fill),
            CommandTexture::None,
            &self.material,
            None,
        );
        drawing_context.push_rounded_rect(&pill, 1.0, 6.0, 12);
        drawing_context.commit(
            self.clip_bounds(),
            Brush::Solid(self.pill_border),
            CommandTexture::None,
            &self.material,
            None,
        );
    }

    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);
    }
}

/// Builder for a [`PlaygroundCodePanelControl`].
#[derive(Clone)]
pub struct PlaygroundCodePanel {
    title: String,
    code: Handle<UiNode>,
    height: f32,
}

impl PlaygroundCodePanel {
    /// Creates a new code panel with the given title and code content.
    pub fn new(title: impl Into<String>, code: impl Into<Handle<UiNode>>) -> Self {
        Self {
            title: title.into(),
            code: code.into(),
            height: 280.0,
        }
    }

    /// Sets the panel height (default 280, clamped to >= 120).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(120.0);
        self
    }

    /// Builds the code panel, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();
        let title_color = theme
            .color("text.secondary")
            .unwrap_or(Color::new(0.14, 0.16, 0.19, 1.0));
        let card_fill = token(&theme, "surface.elevated", Color::new(1.0, 1.0, 1.0, 1.0));
        let card_border = token(&theme, "border.subtle", Color::new(0.0, 0.0, 0.0, 0.14));
        let divider = token(&theme, "border.subtle", Color::new(0.0, 0.0, 0.0, 0.10));
        let pill_fill = token(&theme, "surface.panel", Color::new(0.98, 0.98, 0.99, 1.0));
        let pill_border = token(&theme, "border.subtle", Color::new(0.0, 0.0, 0.0, 0.12));

        let title_text: Handle<UiNode> = Label::new(&self.title)
            .font_size(13.0)
            .color(title_color)
            .build(cx)
            .into();

        let scroll: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            ScrollViewerBuilder::new(WidgetBuilder::new())
                .with_content(self.code)
                .with_vertical_scroll_allowed(true)
                .with_horizontal_scroll_allowed(false)
                .with_v_scroll_speed(48.0)
                .build(&mut ctx)
                .to_base()
        };

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let control = PlaygroundCodePanelControl {
                widget: WidgetBuilder::new()
                    .with_name("raikou_playground_code")
                    .with_children(vec![title_text, scroll])
                    .build(&ctx),
                title_text,
                scroll,
                height: self.height,
                card_fill,
                card_border,
                divider,
                pill_fill,
                pill_border,
            };
            ctx.add(control).transmute()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built code panel.
pub type PlaygroundCodePanelHandle = Handle<UiNode>;

/// A code text node whose contents are generated by a closure.
pub struct PlaygroundCodeBlock {
    code: Rc<dyn Fn() -> String>,
}

impl PlaygroundCodeBlock {
    /// Creates a new code block that renders the output of the given closure.
    pub fn new(code: impl Fn() -> String + 'static) -> Self {
        Self {
            code: Rc::new(code),
        }
    }

    /// Builds the code block text node and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Handle<UiNode> {
        let fg = cx
            .theme()
            .color("text.primary")
            .unwrap_or(Color::new(0.1, 0.1, 0.1, 1.0));
        let mut ctx = cx.ctx();
        TextBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_code")
                .with_foreground(Brush::Solid(to_fyrox_color(fg)).into()),
        )
        .with_text((self.code)())
        .build(&mut ctx)
        .to_base()
    }
}

/// Re-renders a code block from a generator closure.
///
/// The closure is invoked immediately and its output is pushed into the code
/// text node. Control handlers typically call this whenever the relevant state
/// changes.
pub fn update_code(ui: &UserInterface, code: Handle<UiNode>, code_fn: &dyn Fn() -> String) {
    set_label_text(ui, code, code_fn());
}

// ---------------------------------------------------------------------------
// Control palette helpers
// ---------------------------------------------------------------------------

/// Returns a `Slider` builder pre-tuned for control palettes (0..100, step 1).
///
/// The reference raikou playground additionally configured thumb size, track
/// height and fill color; those knobs do not map onto fyrox's `ScrollBar`-backed
/// `Slider`, so they are intentionally omitted here.
pub fn control_slider() -> Slider {
    Slider::new().min(0.0).max(100.0).step(1.0)
}

/// Builds a `Stack` of a heading followed by explanatory note lines.
pub fn playground_notes(cx: &mut BuildCx, title: impl Into<String>, lines: &[&str]) -> Stack {
    let theme = cx.theme().clone();
    let title_color = theme
        .color("text.primary")
        .unwrap_or(Color::new(0.12, 0.14, 0.17, 1.0));
    let line_color = theme
        .color("text.muted")
        .unwrap_or(Color::new(0.34, 0.39, 0.46, 1.0));

    let mut notes = Stack::new().spacing(10.0).child(
        Label::new(title)
            .font_size(20.0)
            .color(title_color)
            .wrap(true)
            .build(cx),
    );

    for line in lines {
        notes = notes.child(
            Label::new(*line)
                .font_size(13.0)
                .color(line_color)
                .wrap(true)
                .build(cx),
        );
    }

    notes
}
