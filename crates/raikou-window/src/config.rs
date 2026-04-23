use winit::dpi::{LogicalSize, Size};
use winit::window::WindowAttributes;

#[derive(Clone, Debug, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    pub initial_size: (f64, f64),
    pub minimum_size: Option<(f64, f64)>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparency: bool,
    pub visible: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Raikou".to_string(),
            initial_size: (800.0, 600.0),
            minimum_size: None,
            resizable: true,
            decorations: true,
            transparency: false,
            visible: true,
        }
    }
}

impl WindowConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn initial_size(mut self, width: f64, height: f64) -> Self {
        self.initial_size = (width, height);
        self
    }

    pub fn minimum_size(mut self, width: f64, height: f64) -> Self {
        self.minimum_size = Some((width, height));
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    pub fn transparency(mut self, transparency: bool) -> Self {
        self.transparency = transparency;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn to_window_attributes(&self) -> WindowAttributes {
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(Size::Logical(LogicalSize::new(
                self.initial_size.0,
                self.initial_size.1,
            )))
            .with_resizable(self.resizable)
            .with_decorations(self.decorations)
            .with_transparent(self.transparency)
            .with_visible(self.visible);

        if let Some((width, height)) = self.minimum_size {
            attributes =
                attributes.with_min_inner_size(Size::Logical(LogicalSize::new(width, height)));
        }

        attributes
    }
}
