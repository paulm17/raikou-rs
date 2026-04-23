use cosmic_text::{FontSystem as CtFontSystem, Metrics};

pub struct FontSystem {
    inner: CtFontSystem,
    default_size: f32,
    default_line_height: f32,
}

impl FontSystem {
    pub fn new() -> Self {
        Self {
            inner: CtFontSystem::new(),
            default_size: 16.0,
            default_line_height: 20.0,
        }
    }

    pub fn with_default_size(mut self, size: f32) -> Self {
        self.default_size = size;
        self
    }

    pub fn with_default_line_height(mut self, line_height: f32) -> Self {
        self.default_line_height = line_height;
        self
    }

    pub fn default_size(&self) -> f32 {
        self.default_size
    }

    pub fn default_line_height(&self) -> f32 {
        self.default_line_height
    }

    pub fn inner(&self) -> &CtFontSystem {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut CtFontSystem {
        &mut self.inner
    }

    pub fn metrics(&self) -> Metrics {
        Metrics::new(self.default_size, self.default_line_height)
    }
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}
