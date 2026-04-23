use raikou_core::Size;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutConstraints {
    pub min: Size,
    pub max: Size,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            min: Size::ZERO,
            max: Size::new(f32::INFINITY, f32::INFINITY),
        }
    }
}

impl LayoutConstraints {
    pub fn clamp(self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min.width, self.max.width),
            size.height.clamp(self.min.height, self.max.height),
        )
    }

    pub fn constrain(self, available: Size) -> Size {
        self.clamp(Size::new(
            available.width.min(self.max.width),
            available.height.min(self.max.height),
        ))
    }
}
