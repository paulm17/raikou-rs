use raikou_core::{Point, Rect, Size, WidgetId};
use raikou_layout::{LayoutContext, LayoutElement, arrange_element, measure_element};
use raikou_skia::Painter;
use raikou_text::{FontSystem, SwashCache};
use raikou_window::events::{KeyboardEvent, PointerEvent};

pub trait Widget {
    fn as_layout_element(&self) -> &dyn LayoutElement;
    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement;

    fn on_pointer_event(&mut self, _event: &PointerEvent) {}
    fn on_keyboard_event(&mut self, _event: &KeyboardEvent) {}

    fn paint(
        &self,
        painter: &Painter<'_>,
        bounds: Rect,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    );

    /// Visit child widgets in paint order (back-to-front).
    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn Widget)) {}
    /// Visit child widgets mutably in paint order (back-to-front).
    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn Widget)) {}
}

pub struct WidgetTree {
    root: Box<dyn Widget>,
    focused_widget: Option<WidgetId>,
}

impl WidgetTree {
    pub fn new(root: Box<dyn Widget>) -> Self {
        Self {
            root,
            focused_widget: None,
        }
    }

    pub fn root(&self) -> &dyn Widget {
        &*self.root
    }

    pub fn measure(&mut self, ctx: &mut LayoutContext, available: Size) {
        measure_element(self.root.as_layout_element_mut(), ctx, available);
    }

    pub fn arrange(&mut self, ctx: &mut LayoutContext, final_rect: Rect) {
        arrange_element(self.root.as_layout_element_mut(), ctx, final_rect);
    }

    pub fn paint(
        &self,
        painter: &Painter<'_>,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        let bounds = self.root.as_layout_element().layout().bounds();
        self.root.paint(painter, bounds, font_system, swash_cache);
    }

    pub fn hit_test(&self, point: Point) -> Option<WidgetId> {
        hit_test_widget(&*self.root, point)
    }

    pub fn focus(&mut self, id: WidgetId) -> bool {
        if contains_widget(&*self.root, id) {
            self.focused_widget = Some(id);
            true
        } else {
            false
        }
    }

    pub fn clear_focus(&mut self) {
        self.focused_widget = None;
    }

    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused_widget
    }

    pub fn on_pointer_event(&mut self, event: &PointerEvent) {
        if let Some(position) = event.position {
            let point = Point::new(position.0 as f32, position.1 as f32);
            if let Some(id) = hit_test_widget(&*self.root, point) {
                with_widget_mut(&mut *self.root, id, &mut |widget| {
                    widget.on_pointer_event(event);
                });
            }
        }
    }

    pub fn on_keyboard_event(&mut self, event: &KeyboardEvent) {
        if let Some(id) = self.focused_widget {
            with_widget_mut(&mut *self.root, id, &mut |widget| {
                widget.on_keyboard_event(event);
            });
        }
    }
}

fn hit_test_widget(widget: &dyn Widget, point: Point) -> Option<WidgetId> {
    let bounds = widget.as_layout_element().layout().bounds();
    if !bounds.contains_point(point) {
        return None;
    }

    let mut result = None;
    widget.visit_children(&mut |child| {
        if result.is_none() {
            result = hit_test_widget(child, point);
        }
    });
    result.or_else(|| Some(widget.as_layout_element().layout().id()))
}

fn contains_widget(widget: &dyn Widget, id: WidgetId) -> bool {
    if widget.as_layout_element().layout().id() == id {
        return true;
    }
    let mut found = false;
    widget.visit_children(&mut |child| {
        if !found {
            found = contains_widget(child, id);
        }
    });
    found
}

fn with_widget_mut(widget: &mut dyn Widget, id: WidgetId, f: &mut dyn FnMut(&mut dyn Widget)) -> bool {
    if widget.as_layout_element().layout().id() == id {
        f(widget);
        return true;
    }
    let mut found = false;
    widget.visit_children_mut(&mut |child| {
        if !found {
            found = with_widget_mut(child, id, f);
        }
    });
    found
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

use raikou_core::Color;
use raikou_layout::TextBlock;

/// A read-only text widget that wraps [`TextBlock`].
pub struct Label {
    text_block: TextBlock,
}

impl Label {
    pub fn new(text: &str) -> Self {
        let mut text_block = TextBlock::new();
        text_block.set_text(text);
        Self { text_block }
    }

    pub fn text(&self) -> &str {
        self.text_block.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_block.set_text(text);
    }

    pub fn text_block(&self) -> &TextBlock {
        &self.text_block
    }

    pub fn text_block_mut(&mut self) -> &mut TextBlock {
        &mut self.text_block
    }
}

impl LayoutElement for Label {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&self) -> &raikou_layout::Layoutable {
        self.text_block.layout()
    }

    fn layout_mut(&mut self) -> &mut raikou_layout::Layoutable {
        self.text_block.layout_mut()
    }

    fn measure_override(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        self.text_block.measure_override(ctx, available)
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        self.text_block.arrange_override(ctx, final_size)
    }

    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn LayoutElement)) {}

    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {}
}

impl Widget for Label {
    fn as_layout_element(&self) -> &dyn LayoutElement {
        self
    }

    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement {
        self
    }

    fn paint(
        &self,
        painter: &Painter<'_>,
        bounds: Rect,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        painter.draw_text(
            font_system,
            swash_cache,
            self.text_block.buffer(),
            bounds.origin,
            self.text_block.color(),
        );
    }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

use raikou_core::RoundedRect;
use raikou_window::events::{KeyState, PointerButton, PointerEventKind};

/// A button widget with a background and a text label.
pub struct Button {
    layout: raikou_layout::Layoutable,
    text_block: TextBlock,
    normal_bg: Color,
    hover_bg: Color,
    pressed_bg: Color,
    corner_radius: f32,
    is_hovered: bool,
    is_pressed: bool,
    on_click: Option<Box<dyn Fn()>>,
}

impl Button {
    pub fn new(text: &str) -> Self {
        let mut text_block = TextBlock::new();
        text_block.set_text(text);
        Self {
            layout: raikou_layout::Layoutable::new(),
            text_block,
            normal_bg: Color::new(0.25, 0.25, 0.25, 1.0),
            hover_bg: Color::new(0.35, 0.35, 0.35, 1.0),
            pressed_bg: Color::new(0.15, 0.15, 0.15, 1.0),
            corner_radius: 4.0,
            is_hovered: false,
            is_pressed: false,
            on_click: None,
        }
    }

    pub fn text(&self) -> &str {
        self.text_block.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_block.set_text(text);
    }

    pub fn set_backgrounds(&mut self, normal: Color, hover: Color, pressed: Color) {
        self.normal_bg = normal;
        self.hover_bg = hover;
        self.pressed_bg = pressed;
    }

    pub fn set_corner_radius(&mut self, radius: f32) {
        self.corner_radius = radius;
    }

    pub fn on_click(&mut self, callback: impl Fn() + 'static) {
        self.on_click = Some(Box::new(callback));
    }

    pub fn clear_on_click(&mut self) {
        self.on_click = None;
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn current_bg(&self) -> Color {
        if self.is_pressed {
            self.pressed_bg
        } else if self.is_hovered {
            self.hover_bg
        } else {
            self.normal_bg
        }
    }
}

impl LayoutElement for Button {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&self) -> &raikou_layout::Layoutable {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut raikou_layout::Layoutable {
        &mut self.layout
    }

    fn measure_override(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        self.text_block.measure_override(ctx, available)
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        self.text_block.arrange_override(ctx, final_size)
    }

    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn LayoutElement)) {}

    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {}
}

impl Widget for Button {
    fn as_layout_element(&self) -> &dyn LayoutElement {
        self
    }

    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement {
        self
    }

    fn on_pointer_event(&mut self, event: &PointerEvent) {
        match event.kind {
            PointerEventKind::Move => {
                self.is_hovered = true;
            }
            PointerEventKind::Button {
                state: KeyState::Down,
                button: PointerButton::Left,
            } => {
                self.is_hovered = true;
                self.is_pressed = true;
            }
            PointerEventKind::Button {
                state: KeyState::Up,
                button: PointerButton::Left,
            } => {
                if self.is_pressed {
                    self.is_pressed = false;
                    if self.is_hovered {
                        if let Some(ref cb) = self.on_click {
                            cb();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn paint(
        &self,
        painter: &Painter<'_>,
        bounds: Rect,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        painter.fill_rounded_rect(
            RoundedRect::from_rect_xy(bounds, self.corner_radius),
            self.current_bg(),
        );
        painter.draw_text(
            font_system,
            swash_cache,
            self.text_block.buffer(),
            bounds.origin,
            self.text_block.color(),
        );
    }
}

// ---------------------------------------------------------------------------
// TextBoxWidget
// ---------------------------------------------------------------------------

use raikou_layout::TextBox;

/// An editable text widget that wraps [`TextBox`].
pub struct TextBoxWidget {
    text_box: TextBox,
}

impl TextBoxWidget {
    pub fn new() -> Self {
        Self {
            text_box: TextBox::new(),
        }
    }

    pub fn text(&self) -> String {
        self.text_box.text()
    }

    pub fn set_text(&mut self, font_system: &mut FontSystem, text: &str) {
        self.text_box.set_text(font_system, text);
    }

    pub fn text_box(&self) -> &TextBox {
        &self.text_box
    }

    pub fn text_box_mut(&mut self) -> &mut TextBox {
        &mut self.text_box
    }
}

impl Default for TextBoxWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextBoxWidget {
    fn as_layout_element(&self) -> &dyn LayoutElement {
        &self.text_box
    }

    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement {
        &mut self.text_box
    }

    fn on_pointer_event(&mut self, event: &PointerEvent) {
        if let Some((x, y)) = event.position {
            // hit-test is not wired to a live FontSystem here; pointer routing
            // would normally receive one from the runtime.  We leave it as a
            // no-op for now — the WidgetTree already routes to the focused
            // widget, and a full integration would pass FontSystem through
            // on_pointer_event (out of scope for 5D).
            let _ = (x, y);
        }
    }

    fn on_keyboard_event(&mut self, _event: &KeyboardEvent) {
        // Full keyboard wiring needs FontSystem for every action.
        // A runtime integration would pass it through; we leave this as
        // a no-op placeholder for 5D.
    }

    fn paint(
        &self,
        painter: &Painter<'_>,
        bounds: Rect,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        // Draw text
        painter.draw_text(
            font_system,
            swash_cache,
            self.text_box.buffer(),
            bounds.origin,
            self.text_box.color(),
        );

        // Draw selection highlights
        for rect in self.text_box.selection_rects() {
            let abs = Rect::from_xywh(
                bounds.origin.x + rect.origin.x,
                bounds.origin.y + rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            painter.fill_rounded_rect(
                RoundedRect::from_rect_xy(abs, 0.0),
                Color::new(0.2, 0.4, 0.8, 0.4),
            );
        }

        // Draw caret
        if let Some(caret) = self.text_box.caret_rect() {
            let abs = Rect::from_xywh(
                bounds.origin.x + caret.origin.x,
                bounds.origin.y + caret.origin.y,
                caret.size.width,
                caret.size.height,
            );
            painter.fill_rounded_rect(
                RoundedRect::from_rect_xy(abs, 0.0),
                Color::new(0.8, 0.8, 0.8, 1.0),
            );
        }
    }
}
