# Phase 4 Handoff — raikou-rs (fyrox port)

Read this fully before continuing. This is the authoritative state of the project
as of the last working session. Everything below is verified/known-true.

## Project Layout
- Workspace root: `/Volumes/Data/Users/paul/development/src/github/rustrepos/raikou`
- Members: `app`, `crates/raikou` (facade), `crates/raikou-core`, `crates/raikou-style`,
  `crates/raikou-widgets`, `crates/raikou-playground`, `crates/raikou-demo`,
  `examples/{button_demo, widgets_demo, forms_demo, containers_demo}`.
  `default-members=["app"]`. Root `Cargo.toml` `workspace.dependencies`:
  `fyrox` (git rev `d30f42867f526a07608a56975e1c128872b17d33`), `smol_str=0.2.2`,
  `uuid=1`, plus path deps. **Add new demo crates to workspace members.**
- fyrox checkout: `/Volumes/Data/Users/paul/.cargo/git/checkouts/fyrox-f0b4313f88b4112e/d30f428/fyrox-ui/src/`
- Reference (original raikou-rs, DIFFERENT rendering architecture): `/Volumes/Data/Users/paul/development/src/github/raikou-dunno/raikou-rs-abandoned-again/crates/raikou-widgets/src/` (e.g. `loading_indicator.rs`, `button_widget.rs`)

## Architecture & Established Patterns (MUST follow)
- **Porting philosophy**: raikou builder APIs map onto native fyrox widgets.
- Every component is a **builder**: `new()` + chainable setters returning `Self`,
  final method `build(&mut BuildCx) -> Component`.
- `Component { handle: Handle<UiNode>, kind: ComponentKind }`. `cx.register(&component)`
  takes `&Component`.
- Handlers live in a per-component `XxxHandlers` struct stored inside
  `ComponentKind::Xxx(XxxHandlers)`. Dispatch from `ComponentRegistry` poll loop:
  `ComponentKind::Xxx(handlers) => handlers.dispatch(ui, message)`.
- Message access pattern: `if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>()`
  (use `.data::<T>()`, NOT `.downcast_ref()`).
- Typed fyrox handles → `Handle<UiNode>` via `.to_base()`.
- **FYROX FONT GOTCHA (learned the hard way)**: `FontResource = Resource<Font>` is
  `Clone` but **NOT `Copy`**. You CANNOT capture it in a variable and use it in a loop
  or an `FnMut` closure (moves it). Fix: call `ctx.default_font()` fresh inside each
  loop iteration, OR `.clone()` at each use, OR use a `for` loop (never `.map()` with
  a moved captured `font` + `&mut ctx`, which makes the closure `FnMut`).
- **FYROX GRID GOTCHA**: `GridBuilder` has NO `.with_child()`. Add children to the base
  `WidgetBuilder::new().with_children(Vec<Handle<UiNode>>)` passed into `GridBuilder::new()`.
- Other confirmed fyrox builders: `DropdownListBuilder::new(wb).with_items(Vec<Handle<UiNode>>).with_selected(usize).with_opt_selected(Option<usize>)` → `DropdownListMessage::Selection(Option<usize>)`.
  `TreeBuilder::new(wb).with_content(Handle).with_items(Vec<Handle<Tree>>).with_expanded(bool)`; `TreeRootBuilder::new(wb).with_items(Vec<Handle<Tree>>)` → `TreeRootMessage::Select(Vec<Handle<Tree>>)`.
  `DropdownMenuBuilder::new(wb).with_header(Handle<UiNode>).with_content(Handle<UiNode>)`.
  `ContextMenuBuilder::new(PopupBuilder)`; `PopupMessage::{Open,Close}`.
  `MenuItemBuilder::new(wb).with_content(MenuItemContent::text(&str)).with_items(...)`; `MenuItemMessage::Click`.

## VERIFIED COMPILE STATE (last session)
- `cargo check -p raikou-widgets` → CLEAN (zero errors/warnings).
- `cargo check -p raikou` (facade) → CLEAN.

## Phase 4 — DONE (6 of 8 components, compiled & wired)
1. `crates/raikou-widgets/src/menu.rs` — `MenuBar`/`MenuItem`/`MenuBarHandlers`.
   `MenuItem { label, enabled, on_click: Option<Rc<dyn Fn(&mut UI,usize)>>, children: Vec<MenuItem> }`;
   `new(label)`/`on_click`/`disabled()`/`submenu(Vec<MenuItem>)`.
   `MenuBarHandlers { item_handles: Vec<Handle<UiNode>>, on_item_click }`; dispatch on
   `MenuItemMessage::Click` → index via `message.destination()` position.
   Build: per top-level menu = header Text (font `ctx.default_font()`, margin
   `Thickness::new(8,4,8,4)`) + vertical StackPanel of items; wrapped in
   `DropdownMenuBuilder`. `pub(crate) fn build_items(...)` recursive helper.
2. `crates/raikou-widgets/src/context_menu.rs` — `ContextMenu`/`ContextMenuHandlers`,
   plus free fns `show_context_menu`/`hide_context_menu` (send `PopupMessage::Open/Close`).
   Build: MenuItems (reuse build_items) → `ContextMenuBuilder::new(PopupBuilder)`.
3. `crates/raikou-widgets/src/select.rs` — `Select`/`SelectHandlers`
   (`on_change: Option<Rc<dyn Fn(&mut UI,usize)>>`, dispatch on
   `DropdownListMessage::Selection(Some(index))`). Build: Text per item (font fetched
   INSIDE loop) + `DropdownListBuilder` name `"raikou_select"`, `.with_items`, optional
   `.with_selected` (clamped to len).
4. `crates/raikou-widgets/src/combobox.rs` — `Combobox`/`ComboboxHandlers`, same pattern,
   name `"raikou_combobox"`, placeholder default `"Search..."`.
5. `crates/raikou-widgets/src/tree.rs` — `TreeNode { label, children, expanded, selected }`,
   `new(label)`/`child`/`expanded()`. `TreeHandlers { on_select }` dispatch on
   `TreeRootMessage::Select`. Build: `build_tree_node` recursive (label Text +
   `TreeBuilder.with_items`, `.with_expanded`), collected via **for-loop calling
   `ctx.default_font()` per iteration** (font clone handled — see tree.rs current code);
   `TreeRootBuilder` name `"raikou_tree"`.
6. `crates/raikou-widgets/src/table.rs` — `TableColumn { header, width }` `new(h, w.max(1.0))`.
   `Table` builder: `new()`/`column`/`row(Vec<impl Into<String>>)`/`row_height`/`margin`.
   Build: Grid; header Text on row 0; data rows on rows 1..; ALL child Text nodes pushed to
   `child_nodes` (font via `ctx.default_font()` in each loop), passed via
   `WidgetBuilder::with_children`; then `add_row(Row::strict(row_height))`, one
   `add_column(Column::strict(col.width))` per column, one `add_row` per data row.
   `ComponentKind::Static` (no dispatch).

## Wiring (ALL DONE & verified)
- `crates/raikou-widgets/src/component.rs`: `ComponentKind` variants now include
  `MenuBar(MenuBarHandlers)`, `ContextMenu(ContextMenuHandlers)`, `Select(SelectHandlers)`,
  `Combobox(ComboboxHandlers)`, `Tree(TreeHandlers)`, `Static`. Imports added for all.
  `dispatch()` has an arm per variant.
- `crates/raikou-widgets/src/lib.rs`: modules `menu, context_menu, select, combobox, tree,
  table` declared; re-exports added (MenuBar/MenuBarHandle/MenuItem,
  ContextMenu/ContextMenuHandle/hide_context_menu/show_context_menu, Select/SelectHandle,
  Combobox/ComboboxHandle, Tree/TreeHandle, Table/TableColumn/TableHandle).
- `crates/raikou/src/prelude.rs` (facade): `raikou_widgets::*` re-exports include all Phase 4
  types.

## REMAINING WORK (in order)
### A. Button-ext — extend existing `crates/raikou-widgets/src/button.rs`
Current Button supports: text, variant, size, width, height, padding, margin, corner_radius,
is_default, is_cancel, on_click, on_mouse_over, on_mouse_out. Uses
`DecoratorBuilder` (normal/hover/pressed brushes) + `ButtonBuilder.with_text_and_font_size`.
Reference to port: `button_widget.rs` (931 lines). Add:
- **ClickMode** enum (reference: Release/Press/Hover) — controls when on_click fires.
- **Loading state** — button shows a spinner instead of/in addition to label when loading.
- **Child-widget content** — allow a custom child widget in the button (not just text).
Verify ButtonHandlers dispatch handles ClickMode differences.

### B. LoadingIndicator — NEW file `crates/raikou-widgets/src/loading_indicator.rs`
**IMPORTANT**: The reference `loading_indicator.rs` (383 lines, read in full — see below) is
built on raikou's OWN rendering system (`raikou_layout::layoutable::LayoutElement`,
`PaintCx`, `Widget` trait with `paint`/`update`). This does NOT exist in fyrox. It MUST be
rewritten as a custom fyrox `Control` using `DrawingContext` primitives.
- `LoadingIndicatorMode` enum: `Arc, Arcs, ArcsRing, DoubleBounce, FlipPlane, Pulse, Ring,
  ThreeDots, Wave`; `Default = Pulse`.
- Builder fields/defaults (from reference): `id: String::new()`, `mode: Pulse`,
  `color: Color::new(0.13, 0.39, 0.94, 1.0)`, `size: 24.0` (max(1.0)), `stroke_width: 2.0`
  (max(1.0)), `speed_ratio: 1.0` (clamp 0.1..=5.0), `is_active: true`, `is_visible: true`,
  `width: Length::Fixed(24.0)`, `height: Length::Fixed(24.0)`, `padding` (all 0.0).
- Builder methods: `id`, `mode`, `color`, `size` (also resizes width/height if Fixed),
  `stroke_width`, `speed_ratio`, `is_active`, `is_visible`, `width`, `height`, `padding`.
- Animation: `update()` advances `animation_time += dt * speed_ratio` when `is_active`.
- Paint math per mode (reference lines 142–298): uses center, radius = size/2 - stroke_width,
  `animation_time * 360 % 360` rotations, `.fract()` time, sin/cos wave functions, drawing
  arcs (start/end fractions), circles, rects. **fyrox DrawingContext equivalent needed**:
  investigate `fyrox-ui/src/draw.rs` — this session did NOT confirm the exact draw primitive
  names (methods do NOT use `push_` prefix; need to grep `pub fn` in draw.rs for e.g.
  `push_line`/`push_rect`/`push_circle`/`push_arc` or similar). Custom Control overrides
  `draw(&mut self, ctx: &mut Draw, drawing_ctx: &mut DrawingContext)` (verify exact signature
  in fyrox `Control` trait, `fyrox-ui/src/control.rs`). Register for per-frame update via
  `WidgetMessage::Update` (a Control must subscribe to `Update` in `on_event` to get
  continuous animation, or use the UI `update` cycle).
- Then wire `LoadingIndicator`/`LoadingIndicatorMode` into `ComponentKind` (likely a
  `Static` or dedicated variant), `lib.rs` module + re-export, facade `prelude.rs`.

### C. Phase 4 demos (examples)
Add via `raikou_demo::run(Options { title, width: 900, height: 600 }, Box::new(build_demo_panel))`
(note: `Options` has `Default = 900x600`/"raikou demo"). Create examples for: `context_menu_demo`,
`menu_demo`, `select_demo`, `combobox_demo`, `tree_demo`, `table_demo`, `loading_demo`
(match naming of existing `examples/{button_demo,...}`). Each must be added to the root
`Cargo.toml` workspace members and depends on `raikou` (facade) + `raikou-demo`.
Follow an existing demo (e.g. `examples/widgets_demo`) for the exact `run`/`Options` call.

### D. Final verification
1. `cargo check --workspace`
2. `cargo run -p <each demo>` for ~8 seconds each — confirm GL init, shaders, **no panic**.

## Demo harness facts
`crates/raikou-demo/src/lib.rs` provides `run(Options, Box<PanelBuilder>)`; `Options {
title, width, height }`. `PanelBuilder` = a closure building the demo panel returning a
`Component`.

## Reference button_widget.rs (931 lines)
Read it when implementing Button-ext at:
`/Volumes/Data/Users/paul/development/src/github/raikou-dunno/raikou-rs-abandoned-again/crates/raikou-widgets/src/button_widget.rs`
Extract ClickMode + loading + child-content semantics and map onto the existing fyrox Button
builder (do NOT blindly copy — it uses the different rendering architecture).
