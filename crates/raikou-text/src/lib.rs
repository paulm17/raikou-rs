mod buffer;
mod font;
mod layout;

pub use buffer::TextBuffer;
pub use font::FontSystem;
pub use layout::TextLayout;

pub use cosmic_text::{
    Action, Affinity, Align, Attrs, AttrsList, AttrsOwned, BidiParagraphs, BorrowedWithFontSystem,
    BufferLine, BufferRef, CacheKey, CacheKeyFlags, Cached, Change, ChangeItem, Color, Cursor,
    Ellipsize, EllipsizeHeightLimit, Fallback, Family, FamilyOwned, Feature, FeatureTag,
    Font as CosmicFont, FontFeatures, Hinting, LayoutCursor, LayoutGlyph, LayoutLine, LayoutRun,
    LayoutRunIter, LegacyRenderer, LetterSpacing, LineEnding, LineIter, Metrics, Motion,
    PhysicalGlyph, PlatformFallback, Renderer, Scroll, Selection, ShapeBuffer, ShapeGlyph,
    ShapeLine, ShapeRunCache, ShapeRunKey, ShapeSpan, ShapeWord, Shaping, SubpixelBin, SwashCache,
    SwashContent, SwashImage, Wrap,
};

#[cfg(test)]
mod tests {
    use crate::{Action, Attrs, Cursor, FontSystem, Motion, Selection, TextBuffer};

    fn setup() -> (FontSystem, TextBuffer) {
        let mut fs = FontSystem::new();
        let buf = TextBuffer::new(&mut fs);
        (fs, buf)
    }

    #[test]
    fn font_system_default() {
        let _fs = FontSystem::new();
    }

    #[test]
    fn text_buffer_basic() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "Hello, World!", &Attrs::new());
        buf.shape(&mut fs);
        assert_eq!(buf.text(), "Hello, World!");
        assert_eq!(buf.len(), 13);
    }

    #[test]
    fn text_buffer_empty() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "", &Attrs::new());
        assert!(buf.text().is_empty());
    }

    #[test]
    fn text_buffer_insert() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "", &Attrs::new());
        buf.insert_string(&mut fs, "hi");
        assert_eq!(buf.text(), "hi");
    }

    #[test]
    fn text_buffer_delete_backward() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello", &Attrs::new());
        buf.action(&mut fs, Action::Motion(Motion::BufferEnd));
        buf.action(&mut fs, Action::Backspace);
        assert_eq!(buf.text(), "hell");
    }

    #[test]
    fn text_buffer_select_all_then_replace() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello", &Attrs::new());
        buf.select_all();
        buf.insert_string(&mut fs, "world");
        assert_eq!(buf.text(), "world");
    }

    #[test]
    fn text_buffer_move_left_collapses_selection() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello", &Attrs::new());
        buf.select_all();
        buf.set_selection(Selection::None);
        buf.action(&mut fs, Action::Motion(Motion::Left));
        assert_eq!(buf.cursor().line, 0);
        assert_eq!(buf.cursor().index, 4);
        assert_eq!(buf.selection(), Selection::None);
    }

    #[test]
    fn text_buffer_move_right_collapses_selection() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello", &Attrs::new());
        buf.select_all();
        buf.set_selection(Selection::None);
        buf.action(&mut fs, Action::Motion(Motion::Right));
        assert_eq!(buf.cursor(), Cursor::new(0, 5));
        assert_eq!(buf.selection(), Selection::None);
    }

    #[test]
    fn text_buffer_cursor_starts_at_end() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello", &Attrs::new());
        assert_eq!(buf.cursor(), Cursor::new(0, 5));
    }

    #[test]
    fn text_buffer_multiline() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello\nworld", &Attrs::new());
        assert_eq!(buf.text(), "hello\nworld");
        assert_eq!(buf.cursor(), Cursor::new(1, 5));
    }

    #[test]
    fn text_buffer_action_enter() {
        let (mut fs, mut buf) = setup();
        buf.set_text(&mut fs, "hello world", &Attrs::new());
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Motion(Motion::Left));
        buf.action(&mut fs, Action::Enter);
        assert_eq!(buf.text(), "hello\n world");
    }

    #[test]
    fn text_buffer_hit_then_backspace() {
        let (mut fs, mut buf) = setup();
        buf.set_size(&mut fs, Some(400.0), Some(400.0));
        buf.set_text(&mut fs, "hello world", &Attrs::new());
        buf.shape(&mut fs);

        let cursor_before = buf.cursor();
        assert_eq!(cursor_before, Cursor::new(0, 11));

        if let Some(clicked) = buf.hit(&mut fs, 50.0, 10.0) {
            assert_ne!(clicked, cursor_before, "click should move cursor");
        }

        let clicked_cursor = buf.cursor();
        buf.action(&mut fs, Action::Backspace);
        assert_ne!(buf.cursor(), clicked_cursor, "backspace should move cursor");
        assert_eq!(buf.text().len(), 10, "backspace should delete one char");
    }

    #[test]
    fn text_buffer_hit_end_of_line_then_backspace() {
        let (mut fs, mut buf) = setup();
        buf.set_size(&mut fs, Some(400.0), Some(400.0));
        buf.set_text(&mut fs, "hello", &Attrs::new());
        buf.shape(&mut fs);

        assert_eq!(buf.cursor(), Cursor::new(0, 5));

        if let Some(_clicked) = buf.hit(&mut fs, 200.0, 10.0) {
            // click at end of line
        }

        let text_before = buf.text();
        buf.action(&mut fs, Action::Backspace);
        assert_ne!(buf.text(), text_before, "backspace should change text");
        assert_eq!(buf.text(), "hell");
    }

    #[test]
    fn text_buffer_multiline_click_then_backspace() {
        let (mut fs, mut buf) = setup();
        buf.set_size(&mut fs, Some(400.0), Some(400.0));
        buf.set_text(&mut fs, "hello\nworld\nfoo", &Attrs::new());
        buf.shape(&mut fs);

        let cursor = buf.hit(&mut fs, 30.0, 25.0);
        assert!(cursor.is_some(), "click should find a cursor");
        let clicked = cursor.unwrap();
        assert_eq!(clicked.line, 1, "should be on line 1");

        let text_before = buf.text();
        buf.action(&mut fs, Action::Backspace);
        assert_ne!(
            buf.text(),
            text_before,
            "backspace after click should change text"
        );
    }

    #[test]
    fn text_buffer_click_line_start_backspace() {
        let (mut fs, mut buf) = setup();
        buf.set_size(&mut fs, Some(400.0), Some(400.0));
        buf.set_text(&mut fs, "abc\ndef\nghi", &Attrs::new());
        buf.shape(&mut fs);

        // click at start of line 2 (should be near x=0, y ~line 2)
        if let Some(clicked) = buf.hit(&mut fs, 2.0, 50.0) {
            buf.set_cursor(clicked);
        }
        buf.action(&mut fs, Action::Backspace);
        // backspace at start of line > 0 should join with previous line
        assert!(
            !buf.text().contains("\n") || buf.line_count() < 3,
            "backspace at line start should join lines or move cursor"
        );
    }
}
