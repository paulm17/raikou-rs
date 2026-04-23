use raikou_core::{Color, LinearGradient, PaintLayer, PaintOrder, Rect, RoundedRect};
#[cfg(test)]
use raikou_core::{CornerRadii, GradientStop};
use skia_safe::canvas::SaveLayerRec;
use skia_safe::gradient_shader;
use skia_safe::paint::Style as PaintStyle;
use skia_safe::{
    BlendMode, BlurStyle, Canvas, ClipOp, Color4f, MaskFilter, Matrix, Paint, Path, RRect,
    SamplingOptions, TileMode, Vector,
};

use crate::image::Image;

pub struct Painter<'a> {
    canvas: &'a Canvas,
}

impl<'a> Painter<'a> {
    pub fn new(canvas: &'a Canvas) -> Self {
        Self { canvas }
    }

    pub fn canvas(&self) -> &Canvas {
        self.canvas
    }

    pub fn paint_order(&self) -> PaintOrder {
        PaintOrder::ParentBeforeChildren
    }

    pub fn paint_layer_order(&self, layer: PaintLayer) -> u8 {
        layer.order_key()
    }

    pub fn clear(&self, color: Color) {
        self.canvas.clear(to_skia_color(color));
    }

    pub fn fill_rounded_rect(&self, rect: RoundedRect, color: Color) {
        let paint = fill_paint(color);
        self.canvas.draw_rrect(to_skia_rrect(rect), &paint);
    }

    pub fn stroke_rounded_rect(&self, rect: RoundedRect, color: Color, stroke_width: f32) {
        let mut paint = stroke_paint(color, stroke_width);
        paint.set_anti_alias(true);
        self.canvas.draw_rrect(to_skia_rrect(rect), &paint);
    }

    pub fn fill_path(&self, path: &Path, color: Color) {
        let paint = fill_paint(color);
        self.canvas.draw_path(path, &paint);
    }

    pub fn stroke_path(&self, path: &Path, color: Color, stroke_width: f32) {
        let mut paint = stroke_paint(color, stroke_width);
        paint.set_anti_alias(true);
        self.canvas.draw_path(path, &paint);
    }

    pub fn clip_rect(&self, rect: Rect) {
        self.canvas
            .clip_rect(to_skia_rect(rect), Some(ClipOp::Intersect), Some(true));
    }

    pub fn clip_rounded_rect(&self, rect: RoundedRect) {
        self.canvas
            .clip_rrect(to_skia_rrect(rect), Some(ClipOp::Intersect), Some(true));
    }

    pub fn fill_linear_gradient(&self, rect: Rect, gradient: &LinearGradient) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(PaintStyle::Fill);
        paint.set_shader(make_linear_gradient(gradient));
        self.canvas.draw_rect(to_skia_rect(rect), &paint);
    }

    pub fn translate(&self, dx: f32, dy: f32) {
        self.canvas.translate((dx, dy));
    }

    pub fn scale(&self, sx: f32, sy: f32) {
        self.canvas.scale((sx, sy));
    }

    pub fn rotate(&self, degrees: f32) {
        self.canvas.rotate(degrees, None);
    }

    pub fn concat(&self, matrix: &Matrix) {
        self.canvas.concat(matrix);
    }

    pub fn with_save<T>(&self, draw: impl FnOnce(&Painter<'_>) -> T) -> T {
        let count = self.canvas.save();
        let result = draw(&Painter::new(self.canvas));
        self.canvas.restore_to_count(count);
        result
    }

    pub fn with_opacity_layer<T>(
        &self,
        bounds: Option<Rect>,
        opacity: f32,
        draw: impl FnOnce(&Painter<'_>) -> T,
    ) -> T {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_alpha_f(opacity.clamp(0.0, 1.0));
        paint.set_blend_mode(BlendMode::SrcOver);

        let skia_bounds;
        let mut layer = SaveLayerRec::default().paint(&paint);
        if let Some(bounds) = bounds {
            skia_bounds = to_skia_rect(bounds);
            layer = layer.bounds(&skia_bounds);
        }

        let count = self.canvas.save_layer(&layer);
        let result = draw(&Painter::new(self.canvas));
        self.canvas.restore_to_count(count);
        result
    }

    pub fn draw_image(&self, image: &Image, dst: Rect) {
        let paint = Paint::default();
        self.canvas
            .draw_image_rect(image.as_skia(), None, to_skia_rect(dst), &paint);
    }

    pub fn draw_image_with_sampling(&self, image: &Image, dst: Rect, sampling: SamplingOptions) {
        let paint = Paint::default();
        self.canvas.draw_image_rect_with_sampling_options(
            image.as_skia(),
            None,
            to_skia_rect(dst),
            sampling,
            &paint,
        );
    }

    pub fn draw_shadow(&self, rect: RoundedRect, color: Color, offset: Vector, blur_sigma: f32) {
        let mut paint = fill_paint(color);
        paint.set_mask_filter(MaskFilter::blur(
            BlurStyle::Normal,
            blur_sigma.max(0.0),
            Some(true),
        ));

        self.with_save(|painter| {
            painter.translate(offset.x, offset.y);
            painter.canvas.draw_rrect(to_skia_rrect(rect), &paint);
        });
    }
}

fn fill_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_style(PaintStyle::Fill);
    paint.set_anti_alias(true);
    paint.set_color4f(to_skia_color(color), None);
    paint
}

fn stroke_paint(color: Color, stroke_width: f32) -> Paint {
    let mut paint = fill_paint(color);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(stroke_width.max(0.0));
    paint
}

fn make_linear_gradient(gradient: &LinearGradient) -> skia_safe::Shader {
    let colors: Vec<Color4f> = gradient
        .stops
        .iter()
        .map(|stop| to_skia_color(stop.color))
        .collect();
    let positions: Vec<f32> = gradient.stops.iter().map(|stop| stop.position).collect();
    gradient_shader::linear(
        (
            (gradient.start.x, gradient.start.y),
            (gradient.end.x, gradient.end.y),
        ),
        colors.as_slice(),
        Some(positions.as_slice()),
        TileMode::Clamp,
        None,
        None,
    )
    .expect("linear gradient requires at least two stops")
}

pub(crate) fn to_skia_rect(rect: Rect) -> skia_safe::Rect {
    skia_safe::Rect::from_xywh(rect.x(), rect.y(), rect.width(), rect.height())
}

fn to_skia_rrect(rect: RoundedRect) -> RRect {
    let radii = rect.radii;
    RRect::new_rect_radii(
        to_skia_rect(rect.rect),
        &[
            to_skia_radius(radii.top_left),
            to_skia_radius(radii.top_right),
            to_skia_radius(radii.bottom_right),
            to_skia_radius(radii.bottom_left),
        ],
    )
}

fn to_skia_radius(radius: raikou_core::CornerRadius) -> skia_safe::Vector {
    skia_safe::Vector::new(radius.x, radius.y)
}

fn to_skia_color(color: Color) -> Color4f {
    Color4f::new(color.red, color.green, color.blue, color.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_contract_reserves_overlay_after_content() {
        let mut surface = skia_safe::surfaces::raster_n32_premul((4, 4)).expect("surface");
        let painter = Painter::new(surface.canvas());

        assert_eq!(painter.paint_order(), PaintOrder::ParentBeforeChildren);
        assert!(
            painter.paint_layer_order(PaintLayer::Content)
                < painter.paint_layer_order(PaintLayer::Overlay(
                    raikou_core::OverlayPaintPhase::AfterContent
                ))
        );
    }

    #[test]
    fn skia_rrect_preserves_individual_corner_radii() {
        let rounded = RoundedRect::new(
            Rect::from_xywh(0.0, 0.0, 20.0, 20.0),
            CornerRadii::new(
                raikou_core::CornerRadius::new(1.0, 2.0),
                raikou_core::CornerRadius::new(3.0, 4.0),
                raikou_core::CornerRadius::new(5.0, 6.0),
                raikou_core::CornerRadius::new(7.0, 8.0),
            ),
        );
        let rrect = to_skia_rrect(rounded);

        assert_eq!(
            rrect.radii(skia_safe::rrect::Corner::UpperLeft),
            skia_safe::Vector::new(1.0, 2.0)
        );
        assert_eq!(
            rrect.radii(skia_safe::rrect::Corner::LowerLeft),
            skia_safe::Vector::new(7.0, 8.0)
        );
    }

    #[test]
    fn gradient_stop_helper_keeps_position_and_color() {
        let stop = GradientStop::new(0.25, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(stop.position, 0.25);
        assert_eq!(stop.color.red, 1.0);
    }
}
