# Raikou Layout Foundation — Gap Analysis & Plan

## Overview

The `raikou-layout` crate provides a working measure/arrange engine with multiple panel types, layout constraints, rounding, and caching. Thirty-five tests pass. However, ten foundational gaps remain that block the engine from being usable in an interactive UI framework.

This document describes each gap, why it matters, what "done" looks like, how to verify it, and the relative difficulty of implementation.

---

## 1. Hit Testing

**Why it's needed:**  
Without hit testing there is no way to map a window coordinate (e.g. a mouse click at `(x, y)`) to the layout element that occupies that point. Every interactive widget — buttons, text inputs, scrollbars — depends on this.

**What completed looks like:**  
- A `fn hit_test(point: Point) -> Option<&dyn LayoutElement>` that walks the tree top-down.
- Each element tests whether the point falls inside its `layout().bounds()`.
- The walk must account for parent-relative coordinates (accumulate offsets as you descend).
- Elements with `Visibility::Collapsed` are skipped.
- The function returns the deepest leaf that contains the point.

**How to test:**  
- Build a `DockPanel` with a header, sidebar, and content area.
- Call `hit_test` with points inside each region.
- Assert the returned element is the expected leaf.
- Test points outside all elements return `None`.
- Test that a collapsed child is not returned even when the point overlaps its zeroed bounds.

---

## 2. Clipping

**Why it's needed:**  
`ScrollContentPresenter` scrolls content by applying a negative offset, but the overflowing content still paints outside the viewport because there is no clip. Any container with a fixed size (scroll panes, clipped panels) needs to restrict its children's paint region.

**What completed looks like:**  
- `Layoutable` stores an optional `clip_rect: Option<Rect>`.
- `arrange_element` sets the clip rect to the element's final arranged bounds (or allows panels to override it, e.g. `ScrollContentPresenter` clips to its viewport size, not its extent).
- The paint loop reads `clip_rect` and applies it via `Painter::clip_rect()` before drawing children.
- Nested clips compose correctly (intersection of parent and child clips).

**How to test:**  
- Place a `Panel` (200×200) inside a `ScrollContentPresenter` sized to 100×100.
- Offset the child so half of it is outside the viewport.
- Render and assert (via pixel readback or snapshot) that the overflow region is transparent/background-colored, not child-colored.
- Verify nested clips: a clipped child inside a clipped parent respects both bounds.

---

## 3. Text Measurement Integration

**Why it's needed:**  
`raikou-text` exists but is disconnected from layout. A UI framework without text is useless. Text elements must measure their intrinsic size based on font metrics and available width (for wrapping).

**What completed looks like:**  
- A `TextBlock` layout element that wraps `raikou-text`.
- `measure_override` calls into the text engine with the available width and receives a `Size` back.
- `arrange_override` passes the final width to the text engine for final line breaking.
- The text engine must support at minimum: single-line unwrapped, multi-line wrapped, and constrained-width measurement.

**How to test:**  
- Measure a `TextBlock` with "Hello" in a 12px font; assert height ≈ font ascent + descent and width ≈ glyph advances.
- Constrain width to half the unwrapped width; assert height doubles (two lines).
- Place `TextBlock` inside a `StackPanel` and verify the panel's desired size grows to accommodate the text.
- Snapshot test: render the `TextBlock` and compare against a reference image.

---

### More information:

The paint layer system is fully implemented in data structures but completely unwired to rendering. Here's exactly what's broken and what must change.
---
The Current State (Broken)
What exists:
- PaintLayer enum with Content and Overlay(AfterContent)
- WindowPaintList that splits entries into content: Vec<PaintEntry> and overlay: Vec<PaintEntry>
- LayoutManager::collect_window_paint_list(&self, root: &dyn LayoutElement) -> WindowPaintList
- PaintLayer::order_key() so Content (0) < Overlay (1)
What the example actually does (ignoring all of it):
fn paint_element(painter, element, parent_offset) {
    // Hard-coded background drawing
    if let Some(bg) = background_color(element) {
        painter.fill_rounded_rect(..., bg);
    }
    // Recursive tree walk — paints children immediately after parent
    element.visit_children(|child| paint_element(painter, child, new_offset));
}
This is a single-pass tree-order walk. It has three fatal problems:
1. Layer order is ignored. An overlay child paints immediately after its parent, interleaved with content. A tooltip inside a button paints before the button's sibling panel.
2. No consumer of WindowPaintList. The list is built but thrown away. No code path calls collect_window_paint_list.
3. No element lookup by ID. WindowPaintList stores WidgetIds. There is no fn find_element_by_id(id) -> &dyn LayoutElement anywhere in the codebase.
---
What Needs To Change (Deep Dive)
1. Add Absolute Bounds Storage
Problem: LayoutElement::layout().bounds() is parent-relative. When you paint from a flat list (WindowPaintList), you lose the tree structure that let you accumulate offsets.
Current: During arrange, DockPanel positions its header child at (0, 0) and its sidebar at (0, 40). These are local coordinates.
What you need: Layoutable must store an additional absolute_bounds: Rect field that gets updated during arrange_element().
pub struct Layoutable {
    // ... existing fields ...
    pub(crate) bounds: Rect,        // parent-relative (current)
    pub(crate) absolute_bounds: Rect, // window-relative (NEW)
}
Where to set it: In arrange_element(), after computing bounds:
let bounds = Rect::new(arranged_rect.origin, arranged_size);
let absolute_bounds = if let Some(parent) = parent_absolute_bounds {
    Rect::from_xywh(
        parent.origin.x + bounds.origin.x,
        parent.origin.y + bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    )
} else {
    bounds // root is already absolute
};
layout.absolute_bounds = absolute_bounds;
Why this matters: When WindowPaintList says "paint widget ID 42", you need to know its window-coordinate position without walking the tree. Overlays especially need absolute coordinates because they float above the content and should not move when parents scroll.
---
2. Add Element Lookup By ID
Problem: WindowPaintList gives you WidgetIds, not element references.
Current: There is zero mapping from WidgetId → &dyn LayoutElement.
What you need: LayoutManager should build and maintain a HashMap<WidgetId, *mut dyn LayoutElement> (or a typed equivalent) during the layout pass.
Implementation approach:
pub struct LayoutManager {
    dirty_measure: BTreeSet<WidgetId>,
    dirty_arrange: BTreeSet<WidgetId>,
    last_available: Option<Size>,
    element_map: HashMap<WidgetId, WeakElementRef>, // NEW
}
// During measure/arrange, populate the map
fn register_element(&mut self, element: &dyn LayoutElement) {
    self.element_map.insert(element.layout().id(), WeakElementRef::from(element));
}
// For painting
pub fn element_by_id(&self, id: WidgetId) -> Option<&dyn LayoutElement> {
    self.element_map.get(&id).and_then(|w| w.upgrade())
}
Alternative (simpler): Instead of a global map, collect_window_paint_list could return a new type that holds direct references:
pub struct PaintList<'a> {
    content: Vec<&'a dyn LayoutElement>,
    overlay: Vec<&'a dyn LayoutElement>,
}
impl LayoutManager {
    pub fn collect_paint_list<'a>(&self, root: &'a dyn LayoutElement) -> PaintList<'a> {
        let mut content = Vec::new();
        let mut overlay = Vec::new();
        self.walk_and_collect(root, &mut content, &mut overlay);
        PaintList { content, overlay }
    }
}
This avoids the ID→pointer indirection entirely. The list holds &dyn LayoutElement directly.
---
3. Change The Paint Loop From Tree-Walk To Layer-Ordered Passes
Current (broken single pass):
fn paint(root, painter) {
    paint_recursive(painter, root, (0.0, 0.0)); // tree order
}
New (correct two-pass):
fn paint(root: &dyn LayoutElement, painter: &Painter, manager: &LayoutManager) {
    painter.clear(background_color);
    
    // Build the paint list from layout manager
    let list = manager.collect_paint_list(root);
    
    // PASS 1: Content layer — paints all normal widgets in tree order
    for element in list.content {
        paint_single_element(painter, element);
    }
    
    // PASS 2: Overlay layer — paints tooltips, modals, drag shadows ON TOP
    for element in list.overlay {
        paint_single_element(painter, element);
    }
}
Why two passes: If a Panel contains a Button, and that Button contains a Tooltip marked as PaintLayer::Overlay(AfterContent), the tree walk would paint:
1. Panel background
2. Button background
3. Tooltip ← WRONG — paints before Panel's sibling
4. Button text
5. Panel's sibling ← WRONG — should paint before Tooltip
With layer-ordered passes:
1. Panel background
2. Button background
3. Button text
4. Panel's sibling
5. Tooltip ← CORRECT — overlays paint after all content
---
4. Add paint() To LayoutElement Trait
Problem: The example hard-codes if let Some(panel) = element.as_any().downcast_ref::<Panel>() { panel.background() }. This only works for Panel. TextBlock, Button, Image etc. have no paint logic.
What you need: Add fn paint(&self, painter: &Painter<'_>, absolute_offset: (f32, f32)) to the LayoutElement trait.
pub trait LayoutElement {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn layout(&self) -> &Layoutable;
    fn layout_mut(&mut self) -> &mut Layoutable;
    fn measure_override(&mut self, available: Size) -> Size;
    fn arrange_override(&mut self, final_size: Size) -> Size;
    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn LayoutElement));
    fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&mut dyn LayoutElement));
    fn paint_layer(&self) -> PaintLayer { PaintLayer::Content }
    
    // NEW: Each element knows how to draw itself
    fn paint(&self, painter: &Painter<'_>, offset: (f32, f32)) {
        // Default: no-op. Panels override to draw background.
        // TextBlock overrides to draw text.
        // Image overrides to draw image.
    }
}
Panel implementation:
impl LayoutElement for Panel {
    // ... existing methods ...
    
    fn paint(&self, painter: &Painter<'_>, offset: (f32, f32)) {
        let bounds = self.layout().absolute_bounds;
        if let Some(bg) = self.background {
            let rect = Rect::from_xywh(
                bounds.origin.x + offset.0,
                bounds.origin.y + offset.1,
                bounds.size.width,
                bounds.size.height,
            );
            painter.fill_rounded_rect(RoundedRect::from_rect_xy(rect, 6.0), bg);
        }
    }
}
---
5. Overlay Elements Paint In Absolute Coordinates
Problem: If a scrolled ScrollContentPresenter contains a Button with a Tooltip overlay, the tooltip should not move when the content scrolls.
Current: The paint loop accumulates parent_offset + child_offset. An overlay inside a scrolled panel gets the scroll offset applied.
What you need: For overlay elements, use absolute_bounds directly — do NOT add additional parent offset.
fn paint_single_element(painter: &Painter<'_>, element: &dyn LayoutElement) {
    let layer = element.paint_layer();
    let bounds = element.layout().absolute_bounds;
    
    match layer {
        PaintLayer::Content => {
            // Content paints at absolute_bounds (already includes parent offsets)
            element.paint(painter, (0.0, 0.0));
        }
        PaintLayer::Overlay(_) => {
            // Overlays paint at absolute_bounds, ignoring any ongoing transform stack
            // If using Skia save/restore, overlays should NOT inherit parent transforms
            painter.with_save(|p| {
                // Reset any parent transforms — overlays are in window space
                // (Implementation depends on whether you use Skia's matrix stack)
                element.paint(p, (0.0, 0.0));
            });
        }
    }
}
---
6. Connect It To The Window Runtime
Problem: The example's RuntimeLifecycle::on_event hand-rolls everything. A real framework needs the runtime to own the paint loop.
What you need: The window runtime (or a PaintEngine type) should:
1. Own the LayoutManager
2. On RedrawRequested, call layout_manager.update(&mut root, window_size)
3. Then call paint(&root, frame.painter(), &layout_manager)
Current example architecture:
impl RuntimeLifecycle for App {
    fn on_event(&mut self, runtime: &mut WindowRuntime, event: &RuntimeEvent) {
        match event {
            RuntimeEvent::RedrawRequested(_) => {
                self.layout(); // calls measure_element + arrange_element directly
                // ... hand-rolled paint ...
            }
        }
    }
}
New architecture:
struct App {
    renderer: Option<SkiaRenderer>,
    layout_manager: LayoutManager, // NEW
    root: Box<dyn Widget>,         // assumes widget bridge done
}
impl RuntimeLifecycle for App {
    fn on_event(&mut self, runtime: &mut WindowRuntime, event: &RuntimeEvent) {
        match event {
            RuntimeEvent::RedrawRequested(_) => {
                // LayoutManager handles both layout AND paint list collection
                let size = self.window_size;
                self.layout_manager.update(&mut *self.root, size);
                
                if let Some(mut renderer) = self.renderer.take() {
                    let mut frame = renderer.begin_frame().unwrap();
                    
                    // PaintEngine consumes WindowPaintList from LayoutManager
                    paint_from_manager(
                        &*self.root,
                        frame.painter(),
                        &self.layout_manager,
                    );
                    
                    frame.present().unwrap();
                    self.renderer = Some(renderer);
                }
                runtime.request_redraw();
            }
        }
    }
}
---
Summary Of Changes
File	Change
raikou-core/src/layoutable.rs (or wherever Layoutable is)	Add absolute_bounds: Rect field
raikou-layout/src/layoutable.rs	Update arrange_element to compute and store absolute_bounds
raikou-layout/src/layoutable.rs	Add fn paint(&self, painter, offset) to LayoutElement trait with default no-op
raikou-layout/src/panels/panel.rs	Implement paint() to draw background using absolute_bounds
raikou-layout/src/layout_manager.rs	Change collect_window_paint_list to return a list holding &dyn LayoutElement references instead of IDs, OR add element lookup map
raikou-layout/src/layout_manager.rs	Ensure update() and update_targeted() populate absolute_bounds during arrange
examples/layout_demo.rs (or runtime)	Replace recursive tree-walk paint with two-pass layer-ordered paint using LayoutManager::collect_window_paint_list
---
The Test You Need
Build this exact tree:
Root (DockPanel)
├── Header (Panel, Content layer, dock=Top, 40px)
├── Sidebar (Panel, Content layer, dock=Left, 150px)
└── Content (Panel, Content layer, fill)
    └── Button (Panel, Content layer)
        ├── Background (Panel, Content layer)
        └── Tooltip (Panel, Overlay layer) ← THIS IS THE KEY
Paint it with the old tree-walk method: the tooltip paints immediately after the button background, before the sidebar finishes.
Paint it with the new layer-ordered method: all content (header, sidebar, content, button) paints first. The tooltip paints last, visually floating above everything.
Snapshot both. The old method shows the tooltip clipped or overlapped by the sidebar. The new method shows the tooltip on top of everything.

---

## 4. Parent Invalidation Bubbling

**Why it's needed:**  
When a child changes (e.g. a label's text updates), it calls `invalidate_measure()`. Today only that child is marked dirty. The parent is never re-measured, so the parent's `desired_size` remains stale and the layout does not update. In WPF/Flutter, invalidation propagates up to the root.

**What completed looks like:**  
- `Layoutable` stores an optional `parent_id: Option<WidgetId>`.
- `invalidate_measure()` inserts itself into the dirty queue **and** calls `invalidate_measure()` on its parent (if any).
- `invalidate_arrange()` inserts itself and calls `invalidate_arrange()` on its parent.
- `LayoutManager` walks the dirty queue from leaves to root so that parents are measured after their children.
- The existing `queue_measure` / `queue_arrange` API on `LayoutManager` is updated to support this.

**How to test:**  
- Build a `StackPanel` containing a `SizedBox` (100×100).
- Call `invalidate_measure()` on the `SizedBox`.
- Run `LayoutManager::update()` and assert the `StackPanel`'s `desired_size` is re-computed (e.g. via `LayoutPassSummary` showing `measured == true` and the root's size updated).
- Test three levels deep: `Panel` → `StackPanel` → `SizedBox`; invalidating the leaf causes all ancestors to remeasure.

---

## 5. Widget Bridge

**Why it's needed:**  
`LayoutElement` is a trait island. The widget types in `raikou-core` know nothing about layout. You cannot say "this `Button` widget uses a `DockPanel` for its internal layout" because there is no connection. Every real UI framework has a bridge: widgets own a layout subtree, or the widget tree itself implements the layout interface.

**What completed looks like:**  
- Define a `Widget` trait in `raikou-core` (or a new `raikou-widgets` crate) that requires `fn as_layout_element(&self) -> &dyn LayoutElement` and `fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement`.
- Concrete widgets (`Button`, `Label`, `TextBox`) implement `Widget` and internally compose `Panel`, `StackPanel`, `TextBlock`, etc.
- The window runtime holds a `Box<dyn Widget>` root instead of a raw `Box<dyn LayoutElement>`.
- `LayoutManager` operates on `&mut dyn Widget` (or delegates to the underlying layout element).

**How to test:**  
- Create a minimal `Button` widget with a background `Panel` and a `TextBlock` label.
- Measure and arrange it; assert the background fills the button bounds and the text is centered.
- Change the button label text; assert `invalidate_measure()` bubbles and the button resizes.
- Render the button and snapshot-test the visual output.

---

## 6. ItemsHost — Real Virtualization

**Why it's needed:**  
The current `ItemsHost` assumes uniform item sizes and does not recycle containers. If items vary in height (e.g. a chat list with mixed message lengths), the estimate is wrong and items paint at incorrect offsets. Real virtualization also requires reusing a small pool of child elements rather than holding thousands in memory.

**What completed looks like:**  
- `ItemsHost` supports variable item sizes by measuring realized children and building an offset cache (similar to Flutter's `SliverList` or WPF's `VirtualizingStackPanel`).
- A `RecyclePool` type holds a fixed number of `Box<dyn LayoutElement>` containers.
- As the viewport scrolls, containers are detached from old data items and re-attached to new ones.
- `realized_range` is computed from the offset cache, not a uniform estimate.
- The total extent is the sum of all item sizes (or an estimate for unrealized items), not `item_count × estimate`.

**How to test:**  
- Create 1000 items with alternating heights (48px and 96px).
- Scroll to the middle; assert only ~10 items are realized.
- Assert the first realized item is at the correct offset (not `index × 48`).
- Scroll back to top; assert the pool reuses the same container instances (check pointer equality or an internal counter).
- Memory test: assert the number of live children stays constant regardless of item count.

---

## 7. ScrollContentPresenter Viewport Bug

**Why it's needed:**  
In `measure_override`, `self.viewport = available`. If the presenter is inside a `StackPanel` with `Orientation::Vertical`, the available height is `INFINITY`. This makes `max_y = extent - infinity = -infinity`, and `clamp(0.0, -infinity)` produces `-infinity` on some paths, breaking scroll math.

**What completed looks like:**  
- `ScrollContentPresenter` stores `viewport` separately from the `available` passed to `measure_override`.
- The viewport is only updated during `arrange_override` (where `final_size` is always finite).
- `measure_override` does not touch `self.viewport`.
- `set_scroll_offset` handles the case where `viewport > extent` gracefully (max offset = 0).

**How to test:**  
- Place a `ScrollContentPresenter` inside a `StackPanel`.
- Set a child larger than the presenter.
- Call `measure_element` and assert no NaN or negative infinities appear in `scroll_offset()`.
- Arrange to a finite size; assert clamping works correctly.
- Test `set_scroll_offset(9999, 9999)` when viewport > extent; assert offset is clamped to `(0, 0)`.

---

## 8. Z-Order / Paint Layer Consumption

**Why it's needed:**  
`PaintLayer`, `WindowPaintList`, and `collect_window_paint_list()` are implemented but nothing uses them. Overlay layers (tooltips, modals, drag shadows) must paint after all content. Without a consumer, the paint loop is a naive recursive walk that draws in tree order.

**What completed looks like:**  
- The window runtime (or a new `PaintEngine`) calls `LayoutManager::collect_window_paint_list()` before each frame.
- It sorts elements by layer order: `Content` first, then `Overlay(AfterContent)`.
- Elements in the same layer paint in tree order.
- Overlay layers paint in absolute coordinates (no parent-offset accumulation).
- A `set_overlay_layer(&mut self, layer: PaintLayer)` API exists on `Layoutable`.

**How to test:**  
- Build a tree with a normal `Panel` (Content) and a child `Panel` marked `Overlay(AfterContent)`.
- Render and assert the overlay draws on top even though it is a child in the tree.
- Snapshot test: the overlay should visually cover the content panel.
- Test multiple overlays; assert they paint in insertion order.

---

## 9. LayoutManager Targeted Updates — Child Recursion Bug

**Why it's needed:**  
`measure_element_targeted` does not recurse into children after calling `measure_override`. If `measure_override` (e.g. in `DockPanel`) already measured children, this is fine. But if a child is dirty and the parent is clean, the parent's `measure_override` is skipped, and the dirty child is never re-measured. This causes stale measurements in targeted update mode.

**What completed looks like:**  
- After `measure_element_targeted` computes the parent's desired size, it walks children and calls `measure_element_targeted` on any child whose `id` is in `dirty_measure` or whose subtree needs measure.
- The `force_measure` flag propagates down correctly.
- Same fix applies to `arrange_element_targeted`.
- `update_targeted()` produces the same bounds as `update()` for any dirty tree.

**How to test:**  
- Build a `DockPanel` → `Panel` → `SizedBox` tree.
- Run a full `update()` to establish baseline bounds.
- Call `queue_measure()` only on the `SizedBox`.
- Run `update_targeted()` and assert the `SizedBox`, `Panel`, and `DockPanel` all have fresh `desired_size` and `bounds` matching the baseline.
- Benchmark: targeted update should touch fewer elements than full update.

---

## 10. Panel Base Behavior — Footgun Documentation / Fix

**Why it's needed:**  
`Panel::arrange_override` places every child at `(0, 0, final_size.width, final_size.height)`. This is the WPF `Panel` default (no layout logic), but it is a footgun because users expect `Panel` to at least stack children. In practice every child overlaps. The current tests may pass because they only have one child per `Panel`.

**What completed looks like:**  
Option A (explicit): Rename `Panel` to `Canvas` (which has explicit positioning) and create a new `Panel` that is an abstract base with no `arrange_override` logic (forcing subclasses to implement it).  
Option B (safe default): Change `Panel` to behave like `StackPanel` with `Orientation::Vertical` as the default, and introduce a new `CustomPanel` base for subclasses.  
Option C (document): Keep behavior as-is but document loudly that `Panel` is a base class for custom panels and does not position children.

**Recommendation:** Option A. Rename current `Panel` to `Canvas` (it already has no layout logic, which matches `Canvas`), and make `Panel` an abstract base with `unimplemented!()` or `panic!()` in `arrange_override`.

**How to test:**  
- Create a `Panel` with two `SizedBox` children.
- If Option A: assert `arrange_override` panics or is abstract.
- If Option B: assert children are stacked vertically with no overlap.
- If Option C: no code change needed, but add a doc-test showing the overlap behavior.

---

## Difficulty Ranking (Easy → Hard)

| Rank | Gap | Difficulty | Rationale |
|------|-----|------------|-----------|
| 1 | **#10 Panel Base Behavior** | Trivial | Rename + doc change only. No algorithmic work. |
| 2 | **#7 ScrollContentPresenter Viewport Bug** | Easy | Remove one assignment from `measure_override`, move to `arrange_override`. Clamp fix is one line. |
| 3 | **#2 Clipping** | Easy | Add a `clip_rect` field, set it during arrange, consume it in paint. Skia already supports `clip_rect`. |
| 4 | **#1 Hit Testing** | Easy–Medium | Tree walk + point-in-rect. Coordinate accumulation is straightforward but easy to get wrong with nested transforms. |
| 5 | **#8 Paint Layer Consumption** | Medium | Requires a paint engine / runtime integration. Sorting layers is easy; wiring it into the frame loop is cross-crate work. |
| 6 | **#4 Parent Invalidation Bubbling** | Medium | Requires parent pointers in `Layoutable`, careful cycle avoidance, and updating `LayoutManager` dirty-queue ordering. |
| 7 | **#9 Targeted Update Recursion** | Medium | Algorithmic fix in `LayoutManager`. Need to ensure targeted and full paths produce identical results without regressing performance. |
| 8 | **#3 Text Measurement Integration** | Medium–Hard | Requires designing the `TextBlock` API, understanding `raikou-text` metrics, and handling line wrapping correctly. |
| 9 | **#5 Widget Bridge** | Hard | Architectural change spanning multiple crates. Needs new traits, crate dependencies, and integration with the window runtime. |
| 10 | **#6 ItemsHost Real Virtualization** | Hard | Most complex algorithmically. Offset cache, recycle pool, variable item sizes, and correct extent calculation are all non-trivial. |

---

## Recommended Execution Order

1. **#10** — Remove footgun immediately (renaming is cheap).
2. **#7** — Fix the math bug before it bites someone.
3. **#2** + **#1** — Clipping and hit testing are tightly coupled (both need coordinate transforms) and unlock interactivity.
4. **#4** — Invalidation bubbling unlocks dynamic content.
5. **#3** — Text makes the framework usable.
6. **#9** — Fix targeted updates once the tree is stable.
7. **#8** — Wire up paint layers after the render loop is mature.
8. **#5** — Widget bridge comes after layout is fully functional.
9. **#6** — Real virtualization is an optimization for large lists; it can wait.
