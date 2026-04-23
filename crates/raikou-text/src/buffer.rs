use cosmic_text::{
    Action, Attrs, Buffer, BufferLine, Cursor, Edit, Editor, Ellipsize, Hinting, Metrics, Scroll,
    Selection, Shaping, Wrap,
};
use raikou_core::Rect;

use crate::FontSystem;

pub struct TextBuffer {
    buffer: Buffer,
    cursor: Cursor,
    selection: Selection,
}

impl TextBuffer {
    pub fn new(font_system: &mut FontSystem) -> Self {
        let metrics = font_system.metrics();
        let buffer = Buffer::new(font_system.inner_mut(), metrics);
        Self {
            buffer,
            cursor: Cursor::default(),
            selection: Selection::None,
        }
    }

    pub fn with_metrics(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        let buffer = Buffer::new(font_system.inner_mut(), metrics);
        Self {
            buffer,
            cursor: Cursor::default(),
            selection: Selection::None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn set_text(&mut self, font_system: &mut FontSystem, text: &str, attrs: &Attrs) {
        self.buffer.set_text(
            font_system.inner_mut(),
            text,
            attrs,
            Shaping::Advanced,
            None,
        );
        self.cursor = Cursor::new(
            self.buffer.lines.len().saturating_sub(1),
            self.buffer.lines.last().map_or(0, |line| line.text().len()),
        );
        self.selection = Selection::None;
    }

    pub fn set_size(
        &mut self,
        font_system: &mut FontSystem,
        width: Option<f32>,
        height: Option<f32>,
    ) {
        self.buffer.set_size(font_system.inner_mut(), width, height);
    }

    pub fn set_metrics_and_size(
        &mut self,
        font_system: &mut FontSystem,
        metrics: Metrics,
        width: Option<f32>,
        height: Option<f32>,
    ) {
        self.buffer
            .set_metrics_and_size(font_system.inner_mut(), metrics, width, height);
    }

    pub fn shape(&mut self, font_system: &mut FontSystem) {
        self.buffer
            .shape_until_scroll(font_system.inner_mut(), false);
    }

    pub fn action(&mut self, font_system: &mut FontSystem, action: Action) {
        let mut editor = Editor::new(&mut self.buffer);
        editor.set_cursor(self.cursor);
        editor.set_selection(self.selection);
        editor.action(font_system.inner_mut(), action);
        editor.shape_as_needed(font_system.inner_mut(), false);
        self.cursor = editor.cursor();
        self.selection = editor.selection();
    }

    pub fn insert_string(&mut self, font_system: &mut FontSystem, data: &str) {
        let mut editor = Editor::new(&mut self.buffer);
        editor.set_cursor(self.cursor);
        editor.set_selection(self.selection);
        editor.insert_string(data, None);
        editor.shape_as_needed(font_system.inner_mut(), false);
        self.cursor = editor.cursor();
        self.selection = editor.selection();
    }

    pub fn delete_selection(&mut self, font_system: &mut FontSystem) -> bool {
        let mut editor = Editor::new(&mut self.buffer);
        editor.set_cursor(self.cursor);
        editor.set_selection(self.selection);
        let deleted = editor.delete_selection();
        if deleted {
            editor.shape_as_needed(font_system.inner_mut(), false);
            self.cursor = editor.cursor();
            self.selection = editor.selection();
        }
        deleted
    }

    pub fn copy_selection(&self) -> Option<String> {
        let (start, end) = self.selection_bounds()?;
        let mut result = String::new();
        if start.line == end.line {
            let line = self.buffer.lines.get(start.line)?;
            let text = line.text();
            if start.index <= end.index && end.index <= text.len() {
                result.push_str(&text[start.index..end.index]);
            }
        } else {
            let first_line = self.buffer.lines.get(start.line)?;
            let text = first_line.text();
            if start.index <= text.len() {
                result.push_str(&text[start.index..]);
                result.push('\n');
            }
            for line_i in (start.line + 1)..end.line {
                let line = self.buffer.lines.get(line_i)?;
                result.push_str(line.text());
                result.push('\n');
            }
            let last_line = self.buffer.lines.get(end.line)?;
            let text = last_line.text();
            if end.index <= text.len() {
                result.push_str(&text[..end.index]);
            }
        }
        Some(result)
    }

    pub fn hit(&mut self, font_system: &mut FontSystem, x: f32, y: f32) -> Option<Cursor> {
        self.buffer
            .shape_until_scroll(font_system.inner_mut(), false);
        let cursor = self.buffer.hit(x, y)?;
        self.cursor = cursor;
        self.selection = Selection::None;
        Some(cursor)
    }

    pub fn drag(&mut self, font_system: &mut FontSystem, x: f32, y: f32) -> Option<Cursor> {
        self.buffer
            .shape_until_scroll(font_system.inner_mut(), false);
        let cursor = self.buffer.hit(x, y)?;
        if cursor != self.cursor {
            if self.selection == Selection::None {
                self.selection = Selection::Normal(self.cursor);
            }
            self.cursor = cursor;
        }
        Some(cursor)
    }

    pub fn layout_runs(&self) -> cosmic_text::LayoutRunIter<'_> {
        self.buffer.layout_runs()
    }

    pub fn text(&self) -> String {
        let mut text = String::new();
        for (i, line) in self.buffer.lines.iter().enumerate() {
            text.push_str(line.text());
            if i + 1 < self.buffer.lines.len() {
                text.push('\n');
            }
        }
        text
    }

    pub fn len(&self) -> usize {
        self.text().len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.lines.iter().all(|line| line.text().is_empty())
    }

    pub fn caret_rect(&self) -> Option<Rect> {
        let line_height = self.buffer.metrics().line_height;
        for run in self.buffer.layout_runs() {
            if run.line_i != self.cursor.line {
                continue;
            }
            for glyph in run.glyphs.iter() {
                if self.cursor.index == glyph.start {
                    return Some(Rect::from_xywh(glyph.x, run.line_top, 1.0, line_height));
                }
                if self.cursor.index > glyph.start && self.cursor.index < glyph.end {
                    let cluster = &run.text[glyph.start..glyph.end];
                    use unicode_segmentation::UnicodeSegmentation;
                    let total = cluster.grapheme_indices(true).count().max(1);
                    let mut before = 0;
                    for (i, _) in cluster.grapheme_indices(true) {
                        if glyph.start + i < self.cursor.index {
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
                && self.cursor.index >= glyph.end
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

    pub fn selection_rects(&self) -> Vec<Rect> {
        let (start, end) = match self.selection_bounds() {
            Some(bounds) => bounds,
            None => return Vec::new(),
        };
        let line_height = self.buffer.metrics().line_height;
        let mut rects = Vec::new();
        for run in self.buffer.layout_runs() {
            if let Some((x, width)) = run.highlight(start, end) {
                rects.push(Rect::from_xywh(x, run.line_top, width, line_height));
            }
        }
        rects
    }

    pub fn selection_bounds(&self) -> Option<(Cursor, Cursor)> {
        match self.selection {
            Selection::None => None,
            Selection::Normal(select) => {
                let (start, end) = if select.line < self.cursor.line
                    || (select.line == self.cursor.line && select.index < self.cursor.index)
                {
                    (select, self.cursor)
                } else {
                    (self.cursor, select)
                };
                Some((start, end))
            }
            Selection::Line(select) => {
                let start_line = select.line.min(self.cursor.line);
                let end_line = select.line.max(self.cursor.line);
                let end_index = self.buffer.lines.get(end_line)?.text().len();
                Some((Cursor::new(start_line, 0), Cursor::new(end_line, end_index)))
            }
            Selection::Word(select) => {
                use unicode_segmentation::UnicodeSegmentation;
                let (mut start, mut end) = if select.line < self.cursor.line
                    || (select.line == self.cursor.line && select.index < self.cursor.index)
                {
                    (select, self.cursor)
                } else {
                    (self.cursor, select)
                };
                if let Some(line) = self.buffer.lines.get(start.line) {
                    start.index = line
                        .text()
                        .unicode_word_indices()
                        .rev()
                        .map(|(i, _)| i)
                        .find(|&i| i < start.index)
                        .unwrap_or(0);
                }
                if let Some(line) = self.buffer.lines.get(end.line) {
                    end.index = line
                        .text()
                        .unicode_word_indices()
                        .map(|(i, word)| i + word.len())
                        .find(|&i| i > end.index)
                        .unwrap_or_else(|| line.text().len());
                }
                Some((start, end))
            }
        }
    }

    pub fn select_all(&mut self) {
        let last_line = self.buffer.lines.len().saturating_sub(1);
        let last_index = self.buffer.lines.last().map_or(0, |line| line.text().len());
        self.cursor = Cursor::new(last_line, last_index);
        self.selection = Selection::Normal(Cursor::new(0, 0));
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    pub fn redraw(&self) -> bool {
        self.buffer.redraw()
    }

    pub fn set_redraw(&mut self, redraw: bool) {
        self.buffer.set_redraw(redraw);
    }

    pub fn metrics(&self) -> Metrics {
        self.buffer.metrics()
    }

    pub fn lines(&self) -> &[BufferLine] {
        &self.buffer.lines
    }

    pub fn line(&self, index: usize) -> Option<&BufferLine> {
        self.buffer.lines.get(index)
    }

    pub fn line_count(&self) -> usize {
        self.buffer.lines.len()
    }

    pub fn set_wrap(&mut self, font_system: &mut FontSystem, wrap: Wrap) {
        self.buffer.set_wrap(font_system.inner_mut(), wrap);
    }

    pub fn set_ellipsize(&mut self, font_system: &mut FontSystem, ellipsize: Ellipsize) {
        self.buffer
            .set_ellipsize(font_system.inner_mut(), ellipsize);
    }

    pub fn set_hinting(&mut self, font_system: &mut FontSystem, hinting: Hinting) {
        self.buffer.set_hinting(font_system.inner_mut(), hinting);
    }

    pub fn set_tab_width(&mut self, font_system: &mut FontSystem, tab_width: u16) {
        self.buffer
            .set_tab_width(font_system.inner_mut(), tab_width);
    }

    pub fn set_scroll(&mut self, scroll: Scroll) {
        self.buffer.set_scroll(scroll);
    }

    pub fn line_text(&self, line: usize) -> Option<&str> {
        self.buffer.lines.get(line).map(|l| l.text())
    }
}
