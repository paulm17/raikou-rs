use std::any::Any;

use raikou_core::{Color as CoreColor, Size};
use raikou_text::{
    Action, Attrs, AttrsList, Color as CosmicColor, Cursor, Ellipsize, Family, FontSystem, Metrics,
    Selection, TextBuffer, Wrap,
};

use crate::layoutable::{LayoutContext, LayoutElement, Layoutable};

/// An editable text layout element wrapping [`TextBuffer`].
///
/// `TextBox` implements [`LayoutElement`] and exposes editing operations
/// (`action`, `hit`, `insert_string`) as well as caret and selection
/// geometry for painting.
///
/// Unlike [`TextBlock`](crate::TextBlock), `TextBox` preserves cursor and
/// selection state across layout passes.
pub struct TextBox {
    layout: Layoutable,
    buffer: TextBuffer,
    font_family: String,
    font_size: f32,
    line_height: f32,
    color: CoreColor,
    wrap: Wrap,
    ellipsize: Ellipsize,
    attrs_dirty: bool,
}

impl TextBox {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        let mut buffer = TextBuffer::new(&mut font_system);
        buffer.set_text(&mut font_system, "", &Attrs::new());
        Self {
            layout: Layoutable::new(),
            buffer,
            font_family: String::new(),
            font_size: 16.0,
            line_height: 20.0,
            color: CoreColor::new(0.0, 0.0, 0.0, 1.0),
            wrap: Wrap::Word,
            ellipsize: Ellipsize::None,
            attrs_dirty: true,
        }
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn set_text(&mut self, font_system: &mut FontSystem, text: &str) {
        let attrs_owned = self.build_attrs_owned();
        let attrs = attrs_owned.as_attrs();
        self.buffer.set_text(font_system, text, &attrs);
        self.layout.invalidate_measure();
    }

    pub fn font_family(&self) -> &str {
        &self.font_family
    }

    pub fn set_font_family(&mut self, family: &str) {
        if self.font_family != family {
            self.font_family = family.to_string();
            self.attrs_dirty = true;
            self.layout.invalidate_measure();
        }
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, size: f32) {
        if self.font_size != size {
            self.font_size = size;
            self.layout.invalidate_measure();
        }
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn set_line_height(&mut self, line_height: f32) {
        if self.line_height != line_height {
            self.line_height = line_height;
            self.layout.invalidate_measure();
        }
    }

    pub fn color(&self) -> CoreColor {
        self.color
    }

    pub fn set_color(&mut self, color: CoreColor) {
        if self.color != color {
            self.color = color;
            self.attrs_dirty = true;
            self.layout.invalidate_arrange();
        }
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn set_wrap(&mut self, wrap: Wrap) {
        if self.wrap != wrap {
            self.wrap = wrap;
            self.layout.invalidate_measure();
        }
    }

    pub fn ellipsize(&self) -> Ellipsize {
        self.ellipsize
    }

    pub fn set_ellipsize(&mut self, ellipsize: Ellipsize) {
        if self.ellipsize != ellipsize {
            self.ellipsize = ellipsize;
            self.layout.invalidate_measure();
        }
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    /// Performs an editing action (e.g. backspace, motion, enter).
    pub fn action(&mut self, font_system: &mut FontSystem, action: Action) {
        let text_before = self.buffer.text();
        self.buffer.action(font_system, action);
        if self.buffer.text() != text_before {
            self.layout.invalidate_measure();
        } else {
            self.layout.invalidate_arrange();
        }
    }

    /// Hit-tests a point and moves the cursor (and clears selection).
    pub fn hit(&mut self, font_system: &mut FontSystem, x: f32, y: f32) -> Option<Cursor> {
        self.buffer.hit(font_system, x, y)
    }

    /// Inserts a string at the current cursor position, replacing any selection.
    pub fn insert_string(&mut self, font_system: &mut FontSystem, data: &str) {
        self.buffer.insert_string(font_system, data);
        self.layout.invalidate_measure();
    }

    /// Returns the caret rectangle in local coordinates, if available.
    pub fn caret_rect(&self) -> Option<raikou_core::Rect> {
        self.buffer.caret_rect()
    }

    /// Returns highlight rectangles for the current selection.
    pub fn selection_rects(&self) -> Vec<raikou_core::Rect> {
        self.buffer.selection_rects()
    }

    pub fn select_all(&mut self) {
        self.buffer.select_all();
        self.layout.invalidate_arrange();
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.buffer.set_cursor(cursor);
        self.layout.invalidate_arrange();
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.buffer.set_selection(selection);
        self.layout.invalidate_arrange();
    }

    pub fn cursor(&self) -> Cursor {
        self.buffer.cursor()
    }

    pub fn selection(&self) -> Selection {
        self.buffer.selection()
    }

    fn build_attrs(&self) -> Attrs<'_> {
        let attrs = if self.font_family.is_empty() {
            Attrs::new()
        } else {
            Attrs::new().family(Family::Name(&self.font_family))
        };
        attrs.color(to_cosmic_color(self.color))
    }

    fn build_attrs_owned(&self) -> raikou_text::AttrsOwned {
        raikou_text::AttrsOwned::new(&self.build_attrs())
    }

    fn ensure_buffer_ready(
        &mut self,
        ctx: &mut LayoutContext,
        width: Option<f32>,
        height: Option<f32>,
    ) {
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.buffer
            .set_metrics_and_size(ctx.font_system, metrics, width, height);
        self.buffer.set_wrap(ctx.font_system, self.wrap);
        self.buffer.set_ellipsize(ctx.font_system, self.ellipsize);

        if self.attrs_dirty {
            let attrs_owned = self.build_attrs_owned();
            let attrs = attrs_owned.as_attrs();
            for line in self.buffer.buffer_mut().lines.iter_mut() {
                line.set_attrs_list(AttrsList::new(&attrs));
            }
            self.attrs_dirty = false;
        }

        self.buffer.shape(ctx.font_system);
    }

    fn measure_text(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        let width = if available.width.is_infinite() || available.width <= 0.0 {
            None
        } else {
            Some(available.width)
        };
        let height = if available.height.is_infinite() || available.height <= 0.0 {
            None
        } else {
            Some(available.height)
        };

        self.ensure_buffer_ready(ctx, width, height);

        let metrics = self.buffer.metrics();
        let mut max_width = 0.0f32;
        let mut min_line_top = f32::MAX;
        let mut max_line_bottom = f32::MIN;
        let mut has_runs = false;

        for run in self.buffer.layout_runs() {
            has_runs = true;
            let line_width = run.glyphs.last().map_or(0.0, |g| g.x + g.w);
            max_width = max_width.max(line_width);
            min_line_top = min_line_top.min(run.line_top);
            max_line_bottom = max_line_bottom.max(run.line_top + metrics.line_height);
        }

        let total_height = if has_runs {
            max_line_bottom - min_line_top
        } else {
            metrics.line_height
        };

        Size::new(max_width, total_height)
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for TextBox {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout(&self) -> &Layoutable {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layoutable {
        &mut self.layout
    }

    fn measure_override(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        self.measure_text(ctx, available)
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        let width = if final_size.width.is_infinite() || final_size.width <= 0.0 {
            None
        } else {
            Some(final_size.width)
        };
        let height = if final_size.height.is_infinite() || final_size.height <= 0.0 {
            None
        } else {
            Some(final_size.height)
        };
        self.ensure_buffer_ready(ctx, width, height);
        final_size
    }

    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn LayoutElement)) {}

    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {}
}

fn to_cosmic_color(color: CoreColor) -> CosmicColor {
    CosmicColor(
        ((color.alpha * 255.0) as u32) << 24
            | ((color.red * 255.0) as u32) << 16
            | ((color.green * 255.0) as u32) << 8
            | ((color.blue * 255.0) as u32),
    )
}
