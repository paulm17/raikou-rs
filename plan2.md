# Raikou Layout Foundation — Deep-Dive Plan (3 Hard Items)

This document breaks down the three hardest items from the original plan into concrete, implementable sub-tasks.

---

## #3 — Text Measurement Integration

**Reality check:** This is not "add a TextBlock and call a measure function." `raikou-text` is built on `cosmic-text`, which requires a heavy `FontSystem`, reshapes glyphs on every size change, and has a three-phase lifecycle (measure → arrange → paint). This item is **the hardest on the entire list** and needs its own breakdown.

### 3A — Layout Context (Pass `FontSystem` through the engine)

**Why:** Every `TextBuffer` operation needs `&mut FontSystem`. The current `measure_override(&mut self, available: Size)` has no way to access it.

**Done when:**
- A `LayoutContext` struct exists holding `&mut FontSystem` and any future shared resources.
- `measure_element` and `arrange_element` take `&mut LayoutContext` instead of just `Size`/`Rect`.
- All existing panels compile with the new signature.

**Test:**
- Create a `LayoutContext` with a `FontSystem`, pass it through `measure_element` on a `SizedBox`. Assert no regressions in existing tests.

**Difficulty:** Medium (touches every file in `raikou-layout`).

---

### 3B — Text Measure Cache

**Why:** Calling `buffer.set_size()` + `buffer.shape()` on every measure pass is O(n) over the text. Without caching, resizing a window with 100 text blocks reshapes 100 buffers every frame.

**Done when:**
- A `TextMeasureCache` keyed by `(text_hash, font_family, font_size, line_height, wrap_mode, available_width)` stores the resulting `Size`.
- Cache invalidates when any key component changes.
- Cache lives inside `LayoutContext` (or globally per-`FontSystem`).

**Test:**
- Measure the same text block twice with identical constraints; assert no re-shape (spy on cosmic-text internals or measure time).
- Change the text; assert cache miss and re-shape.

**Difficulty:** Medium.

---

### 3C — `TextBlock` Layout Element (Display-Only)

**Why:** This is the actual widget. It must integrate `raikou-text::TextBuffer` with `LayoutElement`.

**Done when:**
- `TextBlock` struct implements `LayoutElement`.
- Properties: `text: String`, `font_family: String`, `font_size: f32`, `line_height: f32`, `color: Color`, `wrap: Wrap`, `ellipsize: Ellipsize`.
- `measure_override`: uses the cache or calls `set_size` + `shape`, then reads total height/width from `layout_runs()`.
- `arrange_override`: `set_size` with final width, `shape`, stores the shaped buffer for paint.
- Property setters call `invalidate_measure()`.

**Test:**
- Single line "Hello" → measure height ≈ line_height, width ≈ glyph advances.
- Constrain width to half unwrapped width → height doubles.
- Place in `StackPanel` → panel desired size grows.
- Snapshot render and compare.

**Difficulty:** Hard (requires 3A and 3B to be clean).

---

### 3D — `TextBox` Layout Element (Editable)

**Why:** `TextBuffer` already has cursor, selection, and hit testing. Reusing it for an editable text box avoids duplicating all that logic.

**Done when:**
- `TextBox` wraps `TextBuffer` and implements `LayoutElement`.
- Exposes `action()`, `hit()`, `insert_string()`, `caret_rect()`, `selection_rects()`.
- Integrates with `RuntimeEvent::Keyboard` and `RuntimeEvent::Pointer` in the window loop.
- Paint draws the text, caret, and selection highlight.

**Test:**
- Click inside `TextBox` → cursor moves.
- Type "hello" → text appears, layout remeasures.
- Select all → highlight rects cover all text.
- Backspace → text shrinks, layout invalidates.

**Difficulty:** Hard (requires 3C + input event wiring).

---

### 3E — Paint Integration (Draw Glyphs via Skia)

**Why:** You can't just `fill_rect` text. You need to rasterize glyphs using `cosmic-text`'s `SwashCache` and draw them via Skia.

**Done when:**
- `raikou-skia` has a `draw_text(buffer: &TextBuffer, offset: Point)` method.
- Iterates `layout_runs()`, then glyphs, uses `SwashCache` to get glyph images, draws via `Painter::draw_image()` or direct Skia text API.
- Handles color from `Attrs`.

**Test:**
- Render "Hello World" at (0,0), snapshot, compare against reference PNG.
- Render multi-line wrapped text, verify line breaks match.

**Difficulty:** Medium–Hard (font rasterization edge cases).

---

### Text Integration — Recommended Order

1. **3A** — Layout context (blocks everything else).
2. **3B** — Measure cache (performance, not correctness).
3. **3C** — `TextBlock` display-only (proves the integration works).
4. **3E** — Paint integration (makes text visible).
5. **3D** — `TextBox` editable (builds on 3C + 3E + input).

**Overall difficulty:** Harder than everything else on the list combined.

---

## #5 — Widget Bridge

**Reality check:** This is not "add a trait." It is the architectural seam between "layout engine" and "application code." It determines how the window runtime owns the tree, how widgets invalidate layout, and how paint commands are generated.

### 5A — `Widget` Trait Definition

**Why:** Something must own both layout state and widget-specific state (e.g. a Button knows its `is_pressed` bool; the layout engine only knows its bounds).

**Done when:**
- `Widget` trait in `raikou-core` (or new `raikou-widgets`) requires:
  ```rust
  fn as_layout_element(&self) -> &dyn LayoutElement;
  fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement;
  fn on_pointer_event(&mut self, event: &PointerEvent);
  fn on_keyboard_event(&mut self, event: &KeyboardEvent);
  fn paint(&self, painter: &Painter<'_>, bounds: Rect);
  ```
- Default implementations for `on_pointer_event` and `on_keyboard_event` are no-ops.

**Test:**
- Implement `Widget` for a `MinimalWidget` that wraps a `Panel`. Assert `as_layout_element()` returns the panel.

**Difficulty:** Easy.

---

### 5B — Widget Tree Container

**Why:** The window runtime currently holds a raw `Box<dyn LayoutElement>`. It needs to hold a `Box<dyn Widget>` and delegate layout to the underlying element.

**Done when:**
- `WidgetTree` struct owns the root `Box<dyn Widget>`.
- `WidgetTree::measure(ctx: &mut LayoutContext, available: Size)` delegates to the root widget's layout element.
- `WidgetTree::arrange(final_rect: Rect)` delegates similarly.
- `WidgetTree::paint(painter: &Painter<'_>)` calls `widget.paint()` in tree order.

**Test:**
- Build a `WidgetTree` with a `MinimalWidget` root.
- Measure, arrange, paint. Assert no panics and bounds are set.

**Difficulty:** Medium.

---

### 5C — Input Routing

**Why:** Hit testing (item #1) produces a target element. But events need to reach the *widget*, not just the layout element. A `Button` widget needs to know it was clicked so it can fire its `on_click` callback.

**Done when:**
- `WidgetTree::hit_test(point: Point) -> Option<(WidgetId, &dyn Widget)>`.
- Window runtime routes `PointerEvent` to the hit widget via `on_pointer_event()`.
- Keyboard events route to the focused widget (requires focus management).

**Test:**
- Create a `Button` widget with an `on_click` callback.
- Simulate a click at the button's center; assert callback fires.
- Click outside the button; assert callback does not fire.

**Difficulty:** Hard (requires #1 Hit Testing + focus management).

---

### 5D — Concrete Widgets (Button, Label, TextBox)

**Why:** The bridge is useless without widgets that use it.

**Done when:**
- `Label` widget: wraps a `TextBlock`, forwards paint.
- `Button` widget: wraps a `Panel` (background) + `Label` (text), handles pointer down/up to toggle visual state.
- `TextBox` widget: wraps the `TextBox` layout element from 3D, handles keyboard input.

**Test:**
- `Label` renders text inside a `StackPanel`.
- `Button` changes background color on hover/press.
- `TextBox` accepts typed input and shows a caret.

**Difficulty:** Medium–Hard (requires 3C, 3D, 5C).

---

### Widget Bridge — Recommended Order

1. **5A** — Define the trait.
2. **5B** — Build the tree container.
3. **#1 Hit Testing** — Required for 5C.
4. **5C** — Route input events.
5. **5D** — Build concrete widgets.

**Overall difficulty:** Hard (architectural, spans crates).

---

## #6 — ItemsHost Real Virtualization

**Reality check:** The current `ItemsHost` assumes uniform item sizes (`estimated_item_size`) and holds every child in memory. Real virtualization needs variable sizes, an offset cache, and container recycling.

### 6A — Variable Item Size Measurement

**Why:** Chat messages, file lists, and feed items have wildly different heights. Uniform estimation causes visible layout jumps.

**Done when:**
- `ItemsHost` measures every realized child during `measure_override`.
- Stores measured sizes in a `Vec<f32>` (one entry per child, initialized to `estimated_item_size`).
- `arrange_override` uses the stored sizes to compute offsets, not the uniform estimate.
- When a new child is realized, its actual size replaces the estimate in the cache.

**Test:**
- 100 items with alternating heights (48px, 96px).
- Scroll to item 50; assert the offset of item 50 is `24×48 + 25×96`, not `50×48`.
- Scroll to bottom; assert last item is at `sum(all_heights) - last_height`.

**Difficulty:** Medium.

---

### 6B — Offset Cache

**Why:** Computing offsets by summing every item size from 0 to N is O(N) per scroll. For 100,000 items this is unusable.

**Done when:**
- A prefix-sum array or segment tree stores cumulative offsets.
- Updating one item's size updates the cache in O(log N) or O(1) amortized.
- `viewport_offset → start_index` lookup is O(log N) via binary search on the prefix sums.

**Test:**
- Build 100,000 items with random heights.
- Scroll to item 50,000; assert offset lookup completes in <1ms.
- Change item 25,000's height; assert cache updates and subsequent lookups are correct.

**Difficulty:** Medium–Hard.

---

### 6C — Container Recycle Pool

**Why:** Holding 1000 `Box<dyn LayoutElement>` containers for 100,000 items wastes memory. Only ~10–20 visible items need live containers.

**Done when:**
- `RecyclePool` holds `Vec<Box<dyn LayoutElement>>` (unbound from data).
- As viewport scrolls, departing containers are returned to the pool.
- Arriving items pull a container from the pool (or create one if pool is empty).
- A `DataTemplate` trait (or closure) creates new containers when the pool is exhausted.

**Test:**
- 10,000 items, viewport shows ~10.
- Assert `ItemsHost.children().len() == ~10` regardless of total item count.
- Scroll from top to bottom; assert pool reuses container instances (track via internal ID or pointer).
- Scroll back to top; assert the same containers are reused.

**Difficulty:** Hard.

---

### 6D — Total Extent with Variable Sizes

**Why:** `ScrollContentPresenter` needs to know the total scrollable height. With variable sizes and virtualization, the total is `sum(all known sizes) + (unrealized_count × average_known_size)`.

**Done when:**
- `ItemsHost` reports `extent` as the sum of all cached sizes plus estimates for uncached items.
- As more items are realized, the estimate converges to the true total.
- `ScrollContentPresenter` clamps scroll offset against this extent.

**Test:**
- 1000 items with random heights.
- Scroll to bottom; assert `extent` equals sum of all actual heights (within epsilon).
- Scroll to middle; assert `extent` is consistent (does not jump).

**Difficulty:** Medium.

---

### 6E — Integration with `ScrollContentPresenter`

**Why:** Virtualization is useless without a scrollable container. The two must communicate: `ScrollContentPresenter` provides viewport size and offset; `ItemsHost` provides extent and realized range.

**Done when:**
- `ScrollContentPresenter` exposes `viewport_offset` and `viewport_size`.
- `ItemsHost` reads these values (or receives them via `set_viewport_offset`).
- `ItemsHost` calls `invalidate_measure()` when viewport offset changes.
- A full test: `ScrollContentPresenter` → `ItemsHost` with 10,000 items, scrollable and clipped.

**Test:**
- Build a `ScrollContentPresenter` containing an `ItemsHost` with 10,000 items.
- Scroll down; assert only visible items are realized.
- Assert overflow items are clipped (requires #2 Clipping).
- Assert total extent allows scrolling to the bottom.

**Difficulty:** Medium–Hard (cross-panel integration).

---

### ItemsHost Virtualization — Recommended Order

1. **6A** — Variable sizes (fixes correctness).
2. **6D** — Total extent (enables scrolling).
3. **6B** — Offset cache (enables performance).
4. **6C** — Recycle pool (enables memory efficiency).
5. **6E** — Integration with scroll (makes it usable).

**Overall difficulty:** Hard (algorithmic complexity, cross-panel coupling).

---

## Revised Difficulty Ranking (All Items)

| Rank | Item | Difficulty | Notes |
|------|------|------------|-------|
| 1 | #10 Panel Base Behavior | Trivial | Rename / doc change |
| 2 | #7 Scroll Viewport Bug | Easy | Move one line, add clamp |
| 3 | #2 Clipping | Easy | Add field, wire into paint |
| 4 | #1 Hit Testing | Easy–Medium | Tree walk, coordinate math |
| 5 | #8 Paint Layer Consumption | Medium | Sort + paint loop wiring |
| 6 | #9 Targeted Update Recursion | Medium | Algorithmic fix in manager |
| 7 | #4 Invalidation Bubbling | Medium | Parent pointers + queue ordering |
| 8 | #5 Widget Bridge | Hard | Architectural, cross-crate |
| 9 | #6 ItemsHost Virtualization | Hard | Algorithmic, offset cache, recycle pool |
| 10 | #3 Text Integration | Very Hard | 5 sub-items, FontSystem lifecycle, paint rasterization |

**Key insight:** #3 is not just hard — it is an order of magnitude more work than everything else. It should be treated as a mini-milestone with its own plan (3A–3E) rather than a single task.
