use raikou_core::{Point, Rect, Size};

pub fn round_value(value: f32) -> f32 {
    if value.is_finite() {
        value.round()
    } else {
        value
    }
}

pub fn round_size(size: Size) -> Size {
    Size::new(round_value(size.width), round_value(size.height))
}

pub fn round_rect(rect: Rect) -> Rect {
    Rect::new(
        Point::new(round_value(rect.origin.x), round_value(rect.origin.y)),
        round_size(rect.size),
    )
}
