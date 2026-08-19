# Plan: fyrox `Image` widget component (`Image`/`ImageView`)

## Status
Planned — not yet implemented. This is the consumer for the `ImageFit` +
`fit_rect` core primitives added to `raikou-core`.

## Goal

Add a raikou component that renders a bitmap via the fyrox `Image` widget,
exposing raikou's `ImageFit` (Fill / Contain / Cover) and sizing model so
raikou apps can display images with controllable fitting.

## Why it's separate from `fit_rect`

`fit_rect` (in `raikou-core::paint`) is a pure, backend-agnostic geometry
computation that makes `ImageFit` meaningful on its own. A fyrox `Image`
widget component is a separate, larger task: it introduces a new builder into
the widget catalog, needs a `ComponentKind` variant + handlers, a texture
source abstraction, and layout/sizing integration with fyrox. It should only
be built when actually needed to display images.

## fyrox backend facts (verified)

- fyrox image widget: `fyrox::gui::image::Image` / `ImageBuilder`
  (`fyrox-ui/src/image.rs`).
- Key `ImageBuilder` options:
  - `with_texture(TextureResource)` — the bitmap source.
  - `with_uv_rect(Rect<f32>)` — normalized portion of texture to draw (cropping).
  - `with_keep_aspect_ratio(bool)` — preserve aspect vs stretch.
  - `with_sync_with_texture_size(bool)` — widget size follows texture size.
  - `with_flip(bool)`, `with_checkerboard_background(bool)`.
- fyrox has **no** `Fill`/`Contain`/`Cover` enum. It models fitting as the
  boolean `keep_aspect_ratio` plus manual widget sizing. So `ImageFit` is NOT
  a 1:1 field mapping — the component must map `ImageFit` onto fyrox's model:
  - `ImageFit::Fill` -> `keep_aspect_ratio = false` (stretch to bounds).
  - `ImageFit::Contain` / `ImageFit::Cover` -> `keep_aspect_ratio = true` +
    use `ImageFit::fit_rect(source_size, bounds)` to compute the destination
    rect (Contain letterboxes, Cover crops).
- Texture is a `TextureResource` (loads async; may be `fyrox_texture::TextureResource`).

## Component surface

Builder (mirrors existing raikou component style, e.g. `Button`):

```rust
Image::new()
    .with_texture(texture)          // TextureResource
    .with_fit(ImageFit::Contain)    // defaults to Fill
    .with_width(w).with_height(h)   // via BoxWidget / widget sizing
    .build(cx) -> Component         // ComponentKind::Image(ImageHandlers)
```

- `ComponentKind::Image(ImageHandlers)` variant + dispatch arm in
  `raikou-widgets/src/component.rs`.
- New module `raikou-widgets/src/image_widget.rs` (or `image.rs`).
- `ImageHandle` type + `set_texture` / `set_fit` update helpers.

## Layout / sizing strategy

Preferred: default fyrox behaviour (`sync_with_texture_size = true`) so the
widget sizes to the texture; when an explicit width/height is given, set them
on the widget and compute the fitted rect with `fit_rect`:
- `Fill` -> stretch to given bounds.
- `Contain` -> fit inside bounds, centered.
- `Cover` -> fill bounds, cropping overflow.

If exact `Cover` cropping is required (fyrox `Image` only supports uniform
stretch, not per-axis crop), either:
- (a) accept fyrox's uniform-scale limitation and expose `Contain`-style
  letterboxing for `Cover`, or
- (b) compute the `Cover` rect via `fit_rect` and set the widget size to that
  rect's size (cropping handled by clip/parent). Document whichever is chosen.

## Handlers / messages

Reuse fyrox `ImageMessage`:
- `ImageMessage::Texture(Option<TextureResource>)`
- plus raikou-level `set_fit` recomputing size via `fit_rect`.

## Verification

- `cargo build -p raikou-widgets -p raikou`.
- Add a `examples/image_playground` (or reuse a demo) exercising
  Fill / Contain / Cover on a real texture.
- Unit-test `ImageFit::fit_rect` against known aspect-ratio cases (already
  covered by the `fit_rect` port in `raikou-core`).

## Open decisions

1. Component name: `Image` vs `ImageView` (fyrox uses `Image`).
2. Whether `Cover` is implemented via exact `fit_rect` sizing or the simpler
   fyrox `keep_aspect_ratio` boolean.
3. Texture source ergonomics: accept `TextureResource` directly, or a path /
   `Rgba` convenience constructor that wraps `TextureResource` loading.

## Out of scope

- Vector images (`fyrox::gui::vector_image`) — separate widget.
- Async texture loading pipeline beyond what `TextureResource` provides.
