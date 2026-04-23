use raikou_core::{Color as CoreColor, Point, Rect};
use raikou_text::{FontSystem, SwashCache, TextBuffer};
use skia_safe::{images, AlphaType, ColorType, Data, ImageInfo};

use crate::image::Image;
use crate::painter::Painter;

impl<'a> Painter<'a> {
    /// Draw a [`TextBuffer`] at the given offset using Skia.
    ///
    /// Iterates the buffer's layout runs, rasterizes each glyph via [`SwashCache`],
    /// and blits the resulting images onto the canvas.
    pub fn draw_text(
        &self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        buffer: &TextBuffer,
        offset: Point,
        default_color: CoreColor,
    ) {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((offset.x, offset.y + run.line_y), 1.0);

                let glyph_color = glyph
                    .color_opt
                    .map_or(default_color, cosmic_color_to_core);

                let image_opt = swash_cache
                    .get_image(font_system.inner_mut(), physical.cache_key);

                let Some(image) = image_opt else { continue };
                let width = image.placement.width;
                let height = image.placement.height;
                if width == 0 || height == 0 {
                    continue;
                }

                let pixels: Vec<u8> = match image.content {
                    raikou_text::SwashContent::Mask => {
                        let base_r = (glyph_color.red * 255.0).round() as u8;
                        let base_g = (glyph_color.green * 255.0).round() as u8;
                        let base_b = (glyph_color.blue * 255.0).round() as u8;
                        let base_a = (glyph_color.alpha * 255.0).round() as u8;

                        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
                        for &alpha in &image.data {
                            let a = ((alpha as u32) * (base_a as u32) / 255).min(255) as u8;
                            let a_f = a as f32 / 255.0;
                            let r = ((base_r as f32) * a_f).round() as u8;
                            let g = ((base_g as f32) * a_f).round() as u8;
                            let b = ((base_b as f32) * a_f).round() as u8;
                            rgba.extend_from_slice(&[r, g, b, a]);
                        }
                        rgba
                    }
                    raikou_text::SwashContent::Color => image.data.clone(),
                    raikou_text::SwashContent::SubpixelMask => {
                        // TODO: subpixel rendering
                        continue;
                    }
                };

                let info = ImageInfo::new(
                    (width as i32, height as i32),
                    ColorType::RGBA8888,
                    AlphaType::Premul,
                    None,
                );
                let data = Data::new_copy(&pixels);
                let skia_image = images::raster_from_data(&info, data, width as usize * 4)
                    .expect("valid raster data");
                let glyph_image = Image::from_skia(skia_image);

                let dst = Rect::from_xywh(
                    (physical.x + image.placement.left) as f32,
                    (physical.y - image.placement.top) as f32,
                    width as f32,
                    height as f32,
                );

                self.draw_image(&glyph_image, dst);
            }
        }
    }
}

fn cosmic_color_to_core(color: raikou_text::Color) -> CoreColor {
    let (r, g, b, a) = color.as_rgba_tuple();
    CoreColor::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use raikou_text::{Attrs, FontSystem, SwashCache, TextBuffer};

    #[test]
    fn draw_text_hello_world_produces_pixels() {
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();
        let mut buffer = TextBuffer::new(&mut font_system);
        buffer.set_text(&mut font_system, "Hello World", &Attrs::new());
        buffer.set_size(&mut font_system, Some(400.0), Some(400.0));
        buffer.shape(&mut font_system);

        let mut surface = skia_safe::surfaces::raster_n32_premul((200, 100)).expect("surface");
        let painter = Painter::new(surface.canvas());
        painter.clear(CoreColor::TRANSPARENT);
        painter.draw_text(
            &mut font_system,
            &mut swash_cache,
            &buffer,
            Point::new(10.0, 30.0),
            CoreColor::new(1.0, 1.0, 1.0, 1.0),
        );

        let pixmap = surface.peek_pixels().expect("pixmap");
        let bytes = pixmap.bytes().expect("bytes");
        let mut has_opaque = false;
        for chunk in bytes.chunks_exact(4) {
            if chunk[3] > 0 {
                has_opaque = true;
                break;
            }
        }
        assert!(
            has_opaque,
            "expected some non-transparent pixels after drawing text"
        );
    }

    #[test]
    fn draw_text_multiline_wraps() {
        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();
        let mut buffer = TextBuffer::new(&mut font_system);
        buffer.set_text(&mut font_system, "Hello World this is a long line", &Attrs::new());
        buffer.set_size(&mut font_system, Some(80.0), Some(400.0));
        buffer.shape(&mut font_system);

        // Count layout runs — with a narrow width, text should wrap to multiple lines
        let run_count = buffer.layout_runs().count();
        assert!(
            run_count >= 2,
            "expected text to wrap to multiple lines, got {run_count}"
        );

        let mut surface = skia_safe::surfaces::raster_n32_premul((200, 200)).expect("surface");
        let painter = Painter::new(surface.canvas());
        painter.clear(CoreColor::TRANSPARENT);
        painter.draw_text(
            &mut font_system,
            &mut swash_cache,
            &buffer,
            Point::new(10.0, 20.0),
            CoreColor::new(1.0, 1.0, 1.0, 1.0),
        );

        let pixmap = surface.peek_pixels().expect("pixmap");
        let bytes = pixmap.bytes().expect("bytes");
        let mut has_opaque = false;
        for chunk in bytes.chunks_exact(4) {
            if chunk[3] > 0 {
                has_opaque = true;
                break;
            }
        }
        assert!(
            has_opaque,
            "expected some non-transparent pixels after drawing multi-line text"
        );
    }
}
