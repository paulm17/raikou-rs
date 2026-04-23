use std::any::Any;

use raikou_core::{Color as CoreColor, Size};
use raikou_text::{Attrs, Color as CosmicColor, Ellipsize, Family, FontSystem, Metrics, TextBuffer, Wrap};

use crate::layoutable::{LayoutContext, LayoutElement, Layoutable};

pub struct TextBlock {
    layout: Layoutable,
    buffer: TextBuffer,
    text: String,
    font_family: String,
    font_size: f32,
    line_height: f32,
    color: CoreColor,
    wrap: Wrap,
    ellipsize: Ellipsize,
}

impl TextBlock {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        let buffer = TextBuffer::new(&mut font_system);
        Self {
            layout: Layoutable::new(),
            buffer,
            text: String::new(),
            font_family: String::new(),
            font_size: 16.0,
            line_height: 20.0,
            color: CoreColor::new(0.0, 0.0, 0.0, 1.0),
            wrap: Wrap::Word,
            ellipsize: Ellipsize::None,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self.layout.invalidate_measure();
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        if self.text != text {
            self.text = text.to_string();
            self.layout.invalidate_measure();
        }
    }

    pub fn font_family(&self) -> &str {
        &self.font_family
    }

    pub fn set_font_family(&mut self, family: &str) {
        if self.font_family != family {
            self.font_family = family.to_string();
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
        self.color = color;
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

    fn ensure_buffer_ready(&mut self, font_system: &mut FontSystem, width: Option<f32>, height: Option<f32>) {
        let metrics = Metrics::new(self.font_size, self.line_height);
        self.buffer
            .set_metrics_and_size(font_system, metrics, width, height);
        self.buffer.set_wrap(font_system, self.wrap);
        self.buffer.set_ellipsize(font_system, self.ellipsize);

        let attrs = if self.font_family.is_empty() {
            Attrs::new()
        } else {
            Attrs::new().family(Family::Name(&self.font_family))
        };
        let attrs = attrs.color(to_cosmic_color(self.color));

        self.buffer.set_text(font_system, &self.text, &attrs);
        self.buffer.shape(font_system);
    }

    fn measure_text(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        if let Some(cached) = ctx.text_measure_cache.get(
            &self.text,
            &self.font_family,
            self.font_size,
            self.line_height,
            self.wrap,
            self.ellipsize,
            available,
        ) {
            return cached;
        }

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

        self.ensure_buffer_ready(ctx.font_system, width, height);

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

        let size = Size::new(max_width, total_height);
        ctx.text_measure_cache.insert(
            &self.text,
            &self.font_family,
            self.font_size,
            self.line_height,
            self.wrap,
            self.ellipsize,
            available,
            size,
        );
        size
    }
}

impl Default for TextBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for TextBlock {
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
        self.ensure_buffer_ready(ctx.font_system, width, height);
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
