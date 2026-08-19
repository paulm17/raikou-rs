//! Backend-agnostic paint primitives: gradients and image fitting.

use crate::geometry::{Color, Point, Rect, Size};

/// A single color stop along a gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// Position of the stop along the gradient (0.0 to 1.0).
    pub position: f32,
    /// Color at this stop.
    pub color: Color,
}

impl GradientStop {
    pub const fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

/// A linear gradient defined by two points and a set of color stops.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Start point of the gradient.
    pub start: Point,
    /// End point of the gradient.
    pub end: Point,
    /// Color stops along the gradient.
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    pub fn new(start: Point, end: Point, stops: Vec<GradientStop>) -> Self {
        Self { start, end, stops }
    }
}

/// How an image should be scaled to fit a target rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// Stretch to fill the target, ignoring aspect ratio.
    #[default]
    Fill,
    /// Fit entirely inside the target, preserving aspect ratio.
    Contain,
    /// Fill the target, cropping overflow to preserve aspect ratio.
    Cover,
}

impl ImageFit {
    /// Computes the destination rectangle for an image of `source` size drawn
    /// into `bounds` using this fit mode. The result is centered within
    /// `bounds`. If `source` has a zero dimension, `bounds` is returned
    /// unchanged.
    pub fn fit_rect(self, source: Size, bounds: Rect) -> Rect {
        if source.width <= 0.0 || source.height <= 0.0 {
            return bounds;
        }

        let image_aspect = source.width / source.height;
        let bounds_aspect = bounds.width() / bounds.height();

        let (width, height) = match self {
            ImageFit::Fill => (bounds.width(), bounds.height()),
            ImageFit::Contain if image_aspect > bounds_aspect => {
                let width = bounds.width();
                (width, width / image_aspect)
            }
            ImageFit::Contain => {
                let height = bounds.height();
                (height * image_aspect, height)
            }
            ImageFit::Cover if image_aspect > bounds_aspect => {
                let height = bounds.height();
                (height * image_aspect, height)
            }
            ImageFit::Cover => {
                let width = bounds.width();
                (width, width / image_aspect)
            }
        };

        Rect::from_xywh(
            bounds.x() + (bounds.width() - width) * 0.5,
            bounds.y() + (bounds.height() - height) * 0.5,
            width,
            height,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_stop_construction() {
        let stop = GradientStop::new(0.5, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(stop.position, 0.5);
        assert_eq!(stop.color, Color::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn linear_gradient_construction() {
        let gradient = LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::new(1.0, 0.0, 0.0, 1.0)),
                GradientStop::new(1.0, Color::new(0.0, 0.0, 1.0, 1.0)),
            ],
        );
        assert_eq!(gradient.start, Point::new(0.0, 0.0));
        assert_eq!(gradient.end, Point::new(100.0, 0.0));
        assert_eq!(gradient.stops.len(), 2);
    }

    #[test]
    fn image_fit_defaults_to_fill() {
        assert_eq!(ImageFit::default(), ImageFit::Fill);
    }

    #[test]
    fn fill_stretches_to_bounds() {
        let source = Size::new(200.0, 100.0);
        let bounds = Rect::from_xywh(10.0, 20.0, 300.0, 150.0);
        let fitted = ImageFit::Fill.fit_rect(source, bounds);
        assert_eq!(fitted, bounds);
    }

    #[test]
    fn contain_wide_image_letterboxes() {
        let source = Size::new(200.0, 100.0);
        let bounds = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let fitted = ImageFit::Contain.fit_rect(source, bounds);
        assert_eq!(fitted.width(), 100.0);
        assert!(fitted.height() < 100.0);
        assert_eq!(fitted.height(), 50.0);
        assert_eq!(fitted.y(), 25.0);
    }

    #[test]
    fn contain_tall_image_pillarboxes() {
        let source = Size::new(100.0, 200.0);
        let bounds = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let fitted = ImageFit::Contain.fit_rect(source, bounds);
        assert_eq!(fitted.height(), 100.0);
        assert!(fitted.width() < 100.0);
        assert_eq!(fitted.width(), 50.0);
        assert_eq!(fitted.x(), 25.0);
    }

    #[test]
    fn cover_wide_image_crops() {
        let source = Size::new(200.0, 100.0);
        let bounds = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let fitted = ImageFit::Cover.fit_rect(source, bounds);
        assert_eq!(fitted.height(), 100.0);
        assert!(fitted.width() > 100.0);
        assert_eq!(fitted.width(), 200.0);
        assert_eq!(fitted.x(), -50.0);
    }

    #[test]
    fn cover_tall_image_crops() {
        let source = Size::new(100.0, 200.0);
        let bounds = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let fitted = ImageFit::Cover.fit_rect(source, bounds);
        assert_eq!(fitted.width(), 100.0);
        assert!(fitted.height() > 100.0);
        assert_eq!(fitted.height(), 200.0);
        assert_eq!(fitted.y(), -50.0);
    }

    #[test]
    fn zero_source_size_returns_bounds() {
        let source = Size::new(0.0, 0.0);
        let bounds = Rect::from_xywh(1.0, 2.0, 50.0, 50.0);
        assert_eq!(ImageFit::Contain.fit_rect(source, bounds), bounds);
    }
}
