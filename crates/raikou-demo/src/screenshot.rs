//! Offscreen UI capture for the screenshot harness.
//!
//! Renders the [`UserInterface`] into a fyrox render-target texture and reads
//! the pixels back synchronously from the UI frame buffer. This works without
//! any macOS Screen Recording permission because nothing touches the window
//! server — pixels never leave our own GL context.
//!
//! Env vars handled by the caller (`raikou-demo::run`):
//!
//! * `RAIKOU_SHOT_OUT` — PNG path to write.
//! * `RAIKOU_SHOT_AT_SECS` — seconds after launch to capture (default 2.0).

use fyrox::asset::manager::ResourceManager;
use fyrox::core::color::Color;
use fyrox::graphics::framebuffer::ReadTarget;
use fyrox::gui::UserInterface;
use fyrox::renderer::{ui_renderer::UiRenderInfo, Renderer};
use fyrox::resource::texture::{TextureResource, TextureResourceExtension};

/// Renders `ui` into a fresh render target of `width` x `height` physical
/// pixels and saves it as a PNG at `out_path`.
pub fn capture_ui_to_png(
    renderer: &mut Renderer,
    ui: &UserInterface,
    resource_manager: &ResourceManager,
    width: u32,
    height: u32,
    clear_color: Color,
    out_path: &str,
) -> Result<(), String> {
    let render_target = TextureResource::new_render_target(width, height);

    renderer
        .render_ui(UiRenderInfo {
            ui,
            render_target: Some(render_target.clone()),
            clear_color,
            resource_manager,
        })
        .map_err(|e| format!("render_ui failed: {e:?}"))?;

    let frame_buffer = renderer
        .ui_frame_buffers
        .get(&render_target.key())
        .ok_or_else(|| "renderer did not create a UI frame buffer".to_string())?;

    let pixels = frame_buffer
        .read_pixels(ReadTarget::Color(0))
        .ok_or_else(|| "read_pixels returned None".to_string())?;

    // GL readback is bottom-up; flip rows so the PNG is top-down.
    let stride = width as usize * 4;
    let mut flipped = vec![0u8; pixels.len()];
    for row in 0..height as usize {
        let src_range = row * stride..(row + 1) * stride;
        let dst_row = height as usize - 1 - row;
        flipped[dst_row * stride..(dst_row + 1) * stride].copy_from_slice(&pixels[src_range]);
    }

    image::save_buffer(out_path, &flipped, width, height, image::ColorType::Rgba8)
        .map_err(|e| format!("failed to save {out_path}: {e}"))
}
