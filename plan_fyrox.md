# Plan: Desktop app built on Fyrox

## What this is

You wanted a UI that doesn't redraw constantly. Instead of building a framework from
scratch, this plan builds your desktop app **on top of Fyrox** — a mature, actively
developed Rust engine whose UI library (`fyrox-ui`) already contains the two things you
care about most:

- **Redraw control**: real dirty-flag redraw — only widgets that actually changed get
  re-drawn (`visual_valid` / `measure_valid` / `arrange_valid` flags), plus a
  `RenderMode::OnChanges` that skips re-uploading the UI to the GPU when nothing changed.
- **Live editing**: a WYSIWYG UI editor, plus a path to runtime hot-reload that we wire up
  ourselves.

Fyrox's own shipped desktop tools (the project-manager and the editor) are living proof
that this exact kind of app — windows, dialogs, forms, docking, dense inspectors — works on
this stack, and they sleep at **zero CPU when idle**. We copy their proven "tool loop"
pattern rather than the game-engine loop.

Every step below ends with something you can run and verify.

## Why Fyrox (and why not the alternatives)

| Need | Fyrox gives you | Caveat |
|---|---|---|
| Redraw control / zero-CPU idle | Widget dirty flags + `RenderMode::OnChanges` + event-driven tool loop (proven in project-manager) | Must use the tool-loop pattern, NOT the game `Executor` (which redraws every frame) |
| Live editing | WYSIWYG `.ui` designer in the editor | Runtime hot-reload of a running app's UI is **not** turnkey — we build it (step 6) |
| Windows/dialogs/forms/dense data | ~55 widgets: Window, MessageBox, DropdownList, TextBox, Tree, Grid, Dock, ScrollViewer, Inspector, Menu, TabControl, FileBrowser… | Builder + message-passing API you must learn |
| Real-time 3D scenes later | Full engine (renderer, scenes, camera, PBR) for free | Compiles physics/sound/gltf even if unused |
| Text | glyph atlas (fontdue) + font fallback chain | No shaping/bidi/grapheme — weak for Arabic/Indic/emoji sequences |

Alternatives rejected: **Bevy** — can't be meaningfully slimmed (ECS + schedules + render
pipeline are the core), UI is mid-migration/churning. **From-scratch makepad-mining** —
maximum ownership but the largest effort; chosen when the goal is owning the machinery
rather than shipping the app. (That plan lives in `plan.md`.)

## Decisions

| Decision | Choice |
|---|---|
| Dependency | Full engine (`fyrox` / `fyrox-impl` `Engine`) — gives the battle-tested `UiRenderer` + 3D for later. (Slim alternative: own ~1k-line UI render pass over `fyrox-graphics`, skipping physics/sound — switch if compile weight or footprint matters) |
| Event loop | Copy the **project-manager tool-loop** (`project-manager/src/main.rs`): winit `EventLoop::run` + `UpdateLoopState` counter + `ControlFlow::Wait`; NOT `Executor` |
| UI render mode | `RenderMode::OnChanges` (re-upload UI to GPU only when a UI message arrived) |
| UI authoring | Code-built first (step 3), then `.ui` files via the editor (step 5) |
| Live reload | We build it: file watcher → `reload_resource` → listen for `ResourceEvent::Reloaded` → rebuild/re-link the `UserInterface` (step 6) |
| Text | fontdue atlas is fine for Latin/CJK dense data; accept the complex-script gap for now |
| License | Fyrox is MIT — no legal friction to build on |

## Project layout

New workspace/app (name TBD, adjust freely):

```
app/
  Cargo.toml            # depends on fyrox, winit
  src/
    main.rs             # winit EventLoop + tool-loop (project-manager pattern)
    app.rs              # Engine + UserInterface, per-frame update/render gating
    ui.rs               # widget tree construction, .ui loading, message handling
    hotreload.rs        # file watcher + UI rebuild (step 6)
    screens/
      main_screen.rs    # the dense-data form/list screen
  ui/
    main.ui             # UI scene files (authorable in the Fyrox editor)
    theme.ui
  assets/
    fonts/
    textures/
```

## Steps

### 1. Skeleton + window that sleeps
New app depending on `fyrox`, winit `EventLoop::run`, an empty `Engine` + empty
`UserInterface`, wired with the **tool-loop** (update only while a counter is active,
`ControlFlow::Wait`, no `request_redraw` when idle).
**Milestone:** a window that stays open; CPU goes to ~0% when you stop touching it.

### 2. Redraw discipline proven
Set `RenderMode::OnChanges`; verify with a CPU/activity monitor that a static screen
produces no redraws and that input/state changes produce exactly one update+render.
**Milestone:** interaction only when something changed — this is the "too many redraw
cycles" problem, solved and measured.

### 3. First interactive screen (code-built)
Build a form with buttons, labels, `TextBox`, layout via `Grid`/`StackPanel`; handle
`UiMessage` (button click, text changed) to update state; apply Fyrox's idiom of mutating
widgets via messages, not direct field writes.
**Milestone:** a working form — click a button, type text, see it respond.

### 4. Dense data + virtualization
`ScrollViewer` + list/tree with row reuse / virtualization (the inspector/editor list
pattern); measured scroll perf on a data set of hundreds of rows.
**Milestone:** a dense scrollable list stays smooth (no frame hitch).

### 5. Author UI in the Fyrox editor
Export the screen to `.ui`; open it in the Fyrox editor (WYSIWYG, drag-to-move, property
inspector, undo/redo); load it in-app via `Resource<UserInterface>`.
**Milestone:** change the screen's layout in the editor, and the app shows the same UI.

### 6. Runtime hot-reload of UI
File watcher on `ui/` → `reload_resource` → catch `ResourceEvent::Reloaded` → rebuild /
re-link the live `UserInterface`, preserving runtime state where possible.
**Milestone:** edit a `.ui` file (or in the editor) while the app runs; the window updates
live — makepad-style iteration without the from-scratch cost.

### 7. 3D viewport inside the desktop UI
Add a viewport panel hosting a 3D scene (camera controller, a test object), embedded as a
widget in the existing form.
**Milestone:** a desktop window with both dense data forms and a live 3D view.

### 8. Shell + polish
Window/dialog/message-box flows, docking/tab layout, DPI/hiDPI handling, theming, and a
perf pass confirming idle CPU and redraw scoping.
**Milestone:** a foundation you can keep building the real app on.

## Design principles

- **Never** use the game `Executor` for this app — always the tool-loop.
- `RenderMode::OnChanges` for the UI.
- Mutate widgets via messages (`MessageDirection::ToWidget`), per Fyrox's documented model.
- Hot-reload is ours to build — the primitives (resource watcher, `.ui` load/save, prefab
  instantiation) all exist; we assemble them.
- Accept the fontdue text-shaping limits unless your data is heavily Arabic/Indic/emoji —
  if it is, revisit the text stack before step 4.

## Risks

- **Text shaping gap** for complex scripts (biggest real limitation).
- **Opinionated API** — builder + message-passing + custom `Visit` serialization; budget
  learning time.
- **Runtime UI hot-reload is custom work** (step 6) — not turnkey.
- **Engine compile weight** from `fyrox-impl` (physics/sound/gltf pulled in even if unused).
- **Version churn** — repo is on `2.0.0-rc.1` (active daily development); pin the version
  you build against.