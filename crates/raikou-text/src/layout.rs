use cosmic_text::{Buffer, Cursor, Metrics};
use raikou_core::Rect;

pub struct TextLayout<'a> {
    buffer: &'a Buffer,
    metrics: Metrics,
}

impl<'a> TextLayout<'a> {
    pub fn new(buffer: &'a Buffer) -> Self {
        let metrics = buffer.metrics();
        Self { buffer, metrics }
    }

    pub fn caret_rect(&self, cursor: &Cursor) -> Option<Rect> {
        let line_height = self.metrics.line_height;
        for run in self.buffer.layout_runs() {
            if run.line_i != cursor.line {
                continue;
            }
            for glyph in run.glyphs.iter() {
                if cursor.index == glyph.start {
                    return Some(Rect::from_xywh(glyph.x, run.line_top, 1.0, line_height));
                }
                if cursor.index > glyph.start && cursor.index < glyph.end {
                    let cluster = &run.text[glyph.start..glyph.end];
                    use unicode_segmentation::UnicodeSegmentation;
                    let total = cluster.grapheme_indices(true).count().max(1);
                    let mut before = 0;
                    for (i, _) in cluster.grapheme_indices(true) {
                        if glyph.start + i < cursor.index {
                            before += 1;
                        }
                    }
                    let offset = glyph.w * (before as f32) / (total as f32);
                    return Some(Rect::from_xywh(
                        glyph.x + offset,
                        run.line_top,
                        1.0,
                        line_height,
                    ));
                }
            }
            if let Some(glyph) = run.glyphs.last()
                && cursor.index >= glyph.end
            {
                return Some(Rect::from_xywh(
                    glyph.x + glyph.w,
                    run.line_top,
                    1.0,
                    line_height,
                ));
            }
            if run.glyphs.is_empty() {
                return Some(Rect::from_xywh(0.0, run.line_top, 1.0, line_height));
            }
        }
        None
    }

    pub fn selection_rects(&self, start: &Cursor, end: &Cursor) -> Vec<Rect> {
        let line_height = self.metrics.line_height;
        let mut rects = Vec::new();
        for run in self.buffer.layout_runs() {
            if let Some((x, width)) = run.highlight(*start, *end) {
                rects.push(Rect::from_xywh(x, run.line_top, width, line_height));
            }
        }
        rects
    }

    pub fn line_height(&self) -> f32 {
        self.metrics.line_height
    }

    pub fn metrics(&self) -> Metrics {
        self.metrics
    }
}
