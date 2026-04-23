use raikou_core::{Color, Rect, Size};
use raikou_layout::{LayoutContext, StackPanel, TextBlock, arrange_element, measure_element};
use raikou_text::Ellipsize;

#[test]
fn text_block_measures_hello() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello");

    let size = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));

    assert!(size.width > 0.0, "width should be positive, got {}", size.width);
    assert!(
        size.height > 0.0,
        "height should be positive, got {}",
        size.height
    );
    // Single line height should be approximately line_height.
    let height_diff = (size.height - 16.0).abs();
    assert!(
        height_diff < 4.0,
        "height should be approximately 16.0, got {} (diff {})",
        size.height,
        height_diff
    );
}

#[test]
fn text_block_wraps_when_constrained() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello World");

    let unwrapped = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    assert!(
        unwrapped.width > 0.0,
        "unwrapped width should be positive"
    );

    let wrapped = measure_element(&mut text, &mut ctx, Size::new(unwrapped.width / 2.0, f32::INFINITY));

    assert!(
        wrapped.width <= unwrapped.width || wrapped.width < unwrapped.width * 1.1,
        "wrapped width ({}) should not exceed unwrapped width ({}) significantly",
        wrapped.width,
        unwrapped.width
    );
    assert!(
        wrapped.height >= unwrapped.height,
        "wrapped height ({}) should be >= unwrapped height ({})",
        wrapped.height,
        unwrapped.height
    );
    // Constraining to roughly half width should cause at least one extra line.
    assert!(
        wrapped.height >= unwrapped.height * 1.5,
        "wrapped height should increase significantly when width is halved"
    );
}

#[test]
fn text_block_in_stack_panel() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut panel = StackPanel::new();
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello");
    panel.push_child(Box::new(text));

    let size = measure_element(&mut panel, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    assert!(
        size.width > 0.0,
        "panel width should accommodate text, got {}",
        size.width
    );
    assert!(
        size.height > 0.0,
        "panel height should accommodate text, got {}",
        size.height
    );

    arrange_element(&mut panel, &mut ctx, Rect::from_xywh(0.0, 0.0, size.width, size.height));
    let text_bounds = panel.children()[0].layout().bounds();
    assert!(
        text_bounds.size.width > 0.0,
        "text bounds width should be positive"
    );
    assert!(
        text_bounds.size.height > 0.0,
        "text bounds height should be positive"
    );
}

#[test]
fn text_block_empty_text_has_line_height() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("");

    let size = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));

    assert_eq!(size.width, 0.0, "empty text should have zero width");
    let height_diff = (size.height - 16.0).abs();
    assert!(
        height_diff < 4.0,
        "empty text height should be approximately line_height, got {} (diff {})",
        size.height,
        height_diff
    );
}

#[test]
fn text_block_measure_cache_hit() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Cache me");

    let available = Size::new(f32::INFINITY, f32::INFINITY);

    // First measure populates the cache.
    let size1 = measure_element(&mut text, &mut ctx, available);
    assert!(
        !ctx.text_measure_cache.is_empty(),
        "cache should have an entry after first measure"
    );

    // Second measure with identical constraints should hit the cache.
    let size2 = measure_element(&mut text, &mut ctx, available);
    assert_eq!(size1, size2, "cached size should match first measure");
}

#[test]
fn text_block_measure_cache_miss_on_text_change() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Original");

    let available = Size::new(f32::INFINITY, f32::INFINITY);

    let size1 = measure_element(&mut text, &mut ctx, available);
    assert_eq!(
        ctx.text_measure_cache.len(),
        1,
        "cache should have one entry"
    );

    text.set_text("Changed text");
    let size2 = measure_element(&mut text, &mut ctx, available);

    assert!(
        size2.width != size1.width || size2.height != size1.height || ctx.text_measure_cache.len() >= 1,
        "changing text should produce a different cache entry or different size"
    );
    assert_eq!(
        ctx.text_measure_cache.len(),
        2,
        "cache should now have two entries"
    );
}

#[test]
fn text_block_measure_cache_miss_on_constraint_change() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello World");

    let size1 = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    assert_eq!(
        ctx.text_measure_cache.len(),
        1,
        "cache should have one entry after unbounded measure"
    );

    let size2 = measure_element(&mut text, &mut ctx, Size::new(10.0, f32::INFINITY));
    assert_eq!(
        ctx.text_measure_cache.len(),
        2,
        "cache should have a second entry for constrained width"
    );
    assert!(
        size2.height >= size1.height,
        "constrained width should produce equal or greater height"
    );
}

#[test]
fn text_block_color_property() {
    let mut text = TextBlock::new();
    assert_eq!(text.color(), Color::new(0.0, 0.0, 0.0, 1.0), "default color should be black");

    let red = Color::new(1.0, 0.0, 0.0, 1.0);
    text.set_color(red);
    assert_eq!(text.color(), red, "color should be red after setter");
}

#[test]
fn text_block_font_family_property() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello");

    assert_eq!(text.font_family(), "", "default font family should be empty");

    let _size1 = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    let cache_len_before = ctx.text_measure_cache.len();

    text.set_font_family("Monospace");
    assert_eq!(text.font_family(), "Monospace");

    let size2 = measure_element(&mut text, &mut ctx, Size::new(f32::INFINITY, f32::INFINITY));
    assert_eq!(
        ctx.text_measure_cache.len(),
        cache_len_before + 1,
        "changing font family should create a new cache entry"
    );
    // Sizes may differ or be the same depending on the font, but the cache should record it.
    assert!(
        size2.width > 0.0,
        "measured width should be positive after font family change"
    );
}

#[test]
fn text_block_ellipsize_property() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut text = TextBlock::new();
    text.set_font_size(12.0);
    text.set_line_height(16.0);
    text.set_text("Hello World this is a longer text");

    assert_eq!(text.ellipsize(), Ellipsize::None, "default ellipsize should be None");

    let _size1 = measure_element(&mut text, &mut ctx, Size::new(50.0, f32::INFINITY));
    let cache_len_before = ctx.text_measure_cache.len();

    text.set_ellipsize(Ellipsize::End(raikou_text::EllipsizeHeightLimit::Lines(1)));
    assert_eq!(
        text.ellipsize(),
        Ellipsize::End(raikou_text::EllipsizeHeightLimit::Lines(1))
    );

    let size2 = measure_element(&mut text, &mut ctx, Size::new(50.0, f32::INFINITY));
    assert_eq!(
        ctx.text_measure_cache.len(),
        cache_len_before + 1,
        "changing ellipsize should create a new cache entry"
    );
    // Ellipsizing to a single line should limit height to roughly one line.
    let height_diff = (size2.height - 16.0).abs();
    assert!(
        height_diff < 4.0,
        "ellipsized height should be approximately one line, got {} (diff {})",
        size2.height,
        height_diff
    );
}
