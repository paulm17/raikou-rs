//! Backend-agnostic geometry and paint primitives.

/// A point in 2D space.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A width/height size in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(Point::new(x, y), Size::new(width, height))
    }

    pub const fn x(self) -> f32 {
        self.origin.x
    }

    pub const fn y(self) -> f32 {
        self.origin.y
    }

    pub const fn width(self) -> f32 {
        self.size.width
    }

    pub const fn height(self) -> f32 {
        self.size.height
    }

    pub const fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    pub const fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn inset(self, dx: f32, dy: f32) -> Self {
        Self::from_xywh(
            self.origin.x + dx,
            self.origin.y + dy,
            (self.size.width - dx * 2.0).max(0.0),
            (self.size.height - dy * 2.0).max(0.0),
        )
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.origin.x < other.origin.x + other.size.width
            && self.origin.x + self.size.width > other.origin.x
            && self.origin.y < other.origin.y + other.size.height
            && self.origin.y + self.size.height > other.origin.y
    }

    pub fn intersection(self, other: Rect) -> Option<Rect> {
        let x1 = self.origin.x.max(other.origin.x);
        let y1 = self.origin.y.max(other.origin.y);
        let x2 = (self.origin.x + self.size.width).min(other.origin.x + other.size.width);
        let y2 = (self.origin.y + self.size.height).min(other.origin.y + other.size.height);

        if x2 <= x1 || y2 <= y1 {
            return None;
        }

        Some(Rect::from_xywh(x1, y1, x2 - x1, y2 - y1))
    }

    pub fn union(self, other: Rect) -> Rect {
        let x1 = self.origin.x.min(other.origin.x);
        let y1 = self.origin.y.min(other.origin.y);
        let x2 = (self.origin.x + self.size.width).max(other.origin.x + other.size.width);
        let y2 = (self.origin.y + self.size.height).max(other.origin.y + other.size.height);

        Rect::from_xywh(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x < self.origin.x + self.size.width
            && point.y >= self.origin.y
            && point.y < self.origin.y + self.size.height
    }

    pub fn is_empty(self) -> bool {
        self.size.width <= 0.0 || self.size.height <= 0.0
    }

    pub fn transformed_aabb(self, angle_degrees: f32) -> Rect {
        if angle_degrees == 0.0 {
            return self;
        }

        let angle_rad = angle_degrees.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let cx = self.origin.x + self.size.width * 0.5;
        let cy = self.origin.y + self.size.height * 0.5;

        let corners = [
            (self.origin.x, self.origin.y),
            (self.origin.x + self.size.width, self.origin.y),
            (
                self.origin.x + self.size.width,
                self.origin.y + self.size.height,
            ),
            (self.origin.x, self.origin.y + self.size.height),
        ];

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for (px, py) in corners {
            let dx = px - cx;
            let dy = py - cy;
            let rot_x = dx * cos_a - dy * sin_a;
            let rot_y = dx * sin_a + dy * cos_a;
            let new_x = cx + rot_x;
            let new_y = cy + rot_y;
            min_x = min_x.min(new_x);
            min_y = min_y.min(new_y);
            max_x = max_x.max(new_x);
            max_y = max_y.max(new_y);
        }

        Rect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn scaled_aabb(self, scale_x: f32, scale_y: f32) -> Rect {
        if scale_x == 1.0 && scale_y == 1.0 {
            return self;
        }

        let cx = self.origin.x + self.size.width * 0.5;
        let cy = self.origin.y + self.size.height * 0.5;

        let new_width = self.size.width * scale_x.abs();
        let new_height = self.size.height * scale_y.abs();

        Rect::from_xywh(
            cx - new_width * 0.5,
            cy - new_height * 0.5,
            new_width,
            new_height,
        )
    }
}

/// Per-edge thickness values.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Thickness {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Thickness {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub const fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub const fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    pub fn deflate_size(self, size: Size) -> Size {
        Size::new(
            (size.width - self.horizontal()).max(0.0),
            (size.height - self.vertical()).max(0.0),
        )
    }

    pub fn deflate_rect(self, rect: Rect) -> Rect {
        Rect::from_xywh(
            rect.origin.x + self.left,
            rect.origin.y + self.top,
            (rect.size.width - self.horizontal()).max(0.0),
            (rect.size.height - self.vertical()).max(0.0),
        )
    }
}

/// Corner radius (x/y for elliptical corners).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CornerRadius {
    pub x: f32,
    pub y: f32,
}

impl CornerRadius {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn circular(radius: f32) -> Self {
        Self::new(radius, radius)
    }
}

/// Four-corner radius set.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CornerRadii {
    pub top_left: CornerRadius,
    pub top_right: CornerRadius,
    pub bottom_right: CornerRadius,
    pub bottom_left: CornerRadius,
}

impl CornerRadii {
    pub const fn new(
        top_left: CornerRadius,
        top_right: CornerRadius,
        bottom_right: CornerRadius,
        bottom_left: CornerRadius,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub const fn uniform(radius: f32) -> Self {
        let radius = CornerRadius::circular(radius);
        Self::new(radius, radius, radius, radius)
    }
}

/// A rectangle with corner radii.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radii: CornerRadii,
}

impl RoundedRect {
    pub const fn new(rect: Rect, radii: CornerRadii) -> Self {
        Self { rect, radii }
    }

    pub const fn from_rect(rect: Rect) -> Self {
        Self::new(rect, CornerRadii::uniform(0.0))
    }

    pub const fn from_rect_xy(rect: Rect, radius: f32) -> Self {
        Self::new(rect, CornerRadii::uniform(radius))
    }
}

/// An RGBA color with float channels in the 0.0-1.0 range.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}
