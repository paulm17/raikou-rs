# raikou

A component API for [fyrox](https://fyrox.rs). Each raikou component is a
builder (`Button::new()...build(cx)`) that constructs the equivalent native
fyrox widget, wraps it in a `Component`, and registers its callbacks into a
`ComponentRegistry` for dispatch from the app's message poll loop.

This workspace is organised the same way as the abandoned
`raikou-rs` project, so the upcoming component and playground ports can land
in the right place from day one.

## Workspace layout

Layered crates, each with a single ownership concern:

| Crate | Path | Owns |
|-------|------|------|
| `raikou` | `crates/raikou` | Facade: re-exports the layers; consumers use `raikou::prelude::*` |
| `raikou-core` | `crates/raikou-core` | Sizing primitives (`Length`, `ControlSize`) |
| `raikou-style` | `crates/raikou-style` | Theme tokens + style resolution (`Theme`, `ButtonStyle`, `ButtonVariant`) |
| `raikou-widgets` | `crates/raikou-widgets` | Component builders + dispatch seam (`Button`, `BuildCx`, `Component`, `ComponentRegistry`) |
| `raikou-playground` | `crates/raikou-playground` | Interactive playground (scaffold) |
| `raikou-app` | `app` | The fyrox tool-loop demo app (no raikou dependency) |
| `button_demo` | `examples/button_demo` | Standalone demo window using the builder API |

The dispatch seam lives in `raikou-widgets`, so adding a component only touches
`raikou-widgets` plus a new `ComponentKind` variant.

## fyrox as a git dependency

The workspace pins fyrox to a git commit instead of the crates.io registry:

```toml
fyrox = { git = "https://github.com/FyroxEngine/Fyrox", rev = "d30f42867f526a07608a56975e1c128872b17d33" }
```

The pinned build corresponds to `2.0.0-rc.1`, which is **not published** on
crates.io (the newest published release is `1.0.1`, 175 commits behind). The
root `Cargo.lock` is committed so the exact commit stays locked. The first
build fetches the fyrox repository and compiles it (this is slow).

## Running

```sh
cargo run -p raikou-app              # the tool-loop demo app
cargo run -p button_demo             # the button builder demo window
```
