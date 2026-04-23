use raikou_core::{Rect, Size};
use raikou_layout::{LayoutContext, LayoutElement, StackPanel, TextBox, arrange_element, measure_element};
use raikou_text::{Action, Motion};

#[test]
fn text_box_measures_empty() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);

    let size = measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));

    assert_eq!(size.width, 0.0, "empty text box should have zero width");
    let height_diff = (size.height - 16.0).abs();
    assert!(
        height_diff < 4.0,
        "empty text box height should be approximately line_height, got {} (diff {})",
        size.height,
        height_diff
    );
}

#[test]
fn text_box_typing_increases_size() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);

    let size_before = {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY))
    };
    assert_eq!(size_before.width, 0.0);

    text_box.insert_string(&mut font_system, "hello");

    let size_after = {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY))
    };
    assert!(
        size_after.width > size_before.width,
        "width should increase after typing, got {} before, {} after",
        size_before.width,
        size_after.width
    );
    assert_eq!(text_box.text(), "hello");
}

#[test]
fn text_box_click_moves_cursor() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "hello world");

    // Measure and arrange so the buffer is shaped.
    let size = {
        let mut ctx = LayoutContext::new(&mut font_system);
        let size = measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
        arrange_element(&mut text_box, &mut ctx, Rect::from_xywh(0.0, 0.0, size.width, size.height));
        size
    };

    let cursor_before = text_box.cursor();
    assert_eq!(cursor_before.line, 0);
    assert_eq!(cursor_before.index, 11); // end of text

    // Click near the start of the text.
    let clicked = text_box.hit(&mut font_system, 2.0, 2.0);
    assert!(clicked.is_some(), "click should return a cursor");
    let clicked = clicked.unwrap();
    assert!(
        clicked.index < cursor_before.index,
        "click near start should move cursor left, got {:?}",
        clicked
    );
}

#[test]
fn text_box_select_all_covers_text() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "hello");

    let size = {
        let mut ctx = LayoutContext::new(&mut font_system);
        let size = measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
        arrange_element(&mut text_box, &mut ctx, Rect::from_xywh(0.0, 0.0, size.width, size.height));
        size
    };

    let rects_before = text_box.selection_rects();
    assert!(rects_before.is_empty(), "no selection should produce no rects");

    text_box.select_all();

    let rects_after = text_box.selection_rects();
    assert!(
        !rects_after.is_empty(),
        "select_all should produce at least one highlight rect"
    );

    let total_width: f32 = rects_after.iter().map(|r| r.size.width).sum();
    assert!(
        total_width > 0.0,
        "selection should cover some text width"
    );
}

#[test]
fn text_box_backspace_shrinks_text() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "hello");

    let size_before = {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY))
    };

    // Move cursor to end and backspace.
    text_box.action(&mut font_system, Action::Motion(Motion::BufferEnd));
    text_box.action(&mut font_system, Action::Backspace);

    assert_eq!(text_box.text(), "hell", "backspace should remove last character");

    let size_after = {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY))
    };
    assert!(
        size_after.width < size_before.width || size_after.height < size_before.height,
        "size should shrink or stay same after backspace, got before {:?} after {:?}",
        size_before,
        size_after
    );
}

#[test]
fn text_box_in_stack_panel() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut panel = StackPanel::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "Hello");
    panel.push_child(Box::new(text_box));

    let mut ctx = LayoutContext::new(&mut font_system);
    let size = measure_element(&mut panel, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    assert!(
        size.width > 0.0,
        "panel width should accommodate text box, got {}",
        size.width
    );
    assert!(
        size.height > 0.0,
        "panel height should accommodate text box, got {}",
        size.height
    );

    arrange_element(&mut panel, &mut ctx, Rect::from_xywh(0.0, 0.0, size.width, size.height));
    let text_bounds = panel.children()[0].layout().bounds();
    assert!(
        text_bounds.size.width > 0.0,
        "text box bounds width should be positive"
    );
    assert!(
        text_bounds.size.height > 0.0,
        "text box bounds height should be positive"
    );
}

#[test]
fn text_box_cursor_only_invalidation() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "hello");

    // Measure once to make it valid.
    {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    }
    assert!(text_box.layout().is_measure_valid());

    // Move cursor left — text does not change, so measure should stay valid.
    text_box.action(&mut font_system, Action::Motion(Motion::Left));
    assert!(
        text_box.layout().is_measure_valid(),
        "cursor motion should not invalidate measure"
    );
    assert!(
        !text_box.layout().is_arrange_valid(),
        "cursor motion should invalidate arrange for repaint"
    );
}

#[test]
fn text_box_text_change_invalidation() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "hi");

    // Measure once to make it valid.
    {
        let mut ctx = LayoutContext::new(&mut font_system);
        measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    }
    assert!(text_box.layout().is_measure_valid());

    // Insert text — measure should be invalidated.
    text_box.insert_string(&mut font_system, " there");
    assert!(
        !text_box.layout().is_measure_valid(),
        "inserting text should invalidate measure"
    );
}

#[test]
fn text_box_caret_rect_after_typing() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "ab");

    let size = {
        let mut ctx = LayoutContext::new(&mut font_system);
        let size = measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
        arrange_element(&mut text_box, &mut ctx, Rect::from_xywh(0.0, 0.0, size.width, size.height));
        size
    };

    let caret = text_box.caret_rect();
    assert!(caret.is_some(), "caret rect should be available for non-empty text");
    let caret = caret.unwrap();
    assert!(caret.size.height > 0.0, "caret should have positive height");
}

#[test]
fn text_box_wraps_when_constrained() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut text_box = TextBox::new();
    text_box.set_font_size(12.0);
    text_box.set_line_height(16.0);
    text_box.set_text(&mut font_system, "Hello World");

    let mut ctx = LayoutContext::new(&mut font_system);
    let unwrapped = measure_element(&mut text_box, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    let wrapped = measure_element(&mut text_box, &mut ctx, Size::new(unwrapped.width / 2.0, f32::INFINITY));

    assert!(
        wrapped.height >= unwrapped.height,
        "wrapped height ({}) should be >= unwrapped height ({})",
        wrapped.height,
        unwrapped.height
    );
    assert!(
        wrapped.height >= unwrapped.height * 1.5,
        "wrapped height should increase significantly when width is halved"
    );
}
