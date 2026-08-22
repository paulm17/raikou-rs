//! The Image component — a bitmap rendered with the raikou `ImageFit` model
//! (Fill / Contain / Cover) on top of fyrox's `Image` widget.
//!
//! fyrox's `Image` always stretches the texture to the widget's full bounds and
//! only supports aspect-ratio preservation for its *measured* size. To get
//! Contain/Cover semantics raikou wraps the fyrox image in a clipping
//! `Canvas`: the canvas is sized to the requested bounds and the inner image is
//! sized/positioned to the rect produced by [`raikou_core::ImageFit::fit_rect`].
//!
//! When no explicit width/height is given the component falls back to fyrox's
//! native behavior: a single image node that sizes itself to the texture.

use fyrox::asset::untyped::ResourceKind;
use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::graph::SceneGraph;
use fyrox::gui::canvas::CanvasBuilder;
use fyrox::gui::image::{Image as FyroxImage, ImageBuilder, ImageMessage};
use fyrox::gui::texture::{
    TextureKind, TexturePixelKind, TextureResource, TextureResourceExtension,
};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};
use uuid::Uuid;

use raikou_core::{ImageFit, Length, Rect, Size, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

/// Handlers of an Image component.
///
/// An image fires no events of its own; the handlers exist to track the current
/// fit mode and the inner fyrox image node so the helpers can re-fit at runtime.
#[derive(Clone, Debug)]
pub struct ImageHandlers {
    /// Fit mode used to (re)size the inner image.
    pub fit: ImageFit,
    /// Handle of the fyrox image node that actually draws the texture.
    pub inner: Handle<UiNode>,
}

/// Builder for an [`crate::Image`] component.
///
/// ```rust,ignore
/// let image = Image::from_rgba(400, 100, pixels)
///     .with_fit(ImageFit::Contain)
///     .with_width(200.0)
///     .with_height(150.0)
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct Image {
    texture: Option<TextureResource>,
    fit: ImageFit,
    width: Length,
    height: Length,
    margin: Thickness,
    flip: bool,
    checkerboard: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

impl Image {
    /// Creates a new image builder.
    pub fn new() -> Self {
        Self {
            texture: None,
            fit: ImageFit::Fill,
            width: Length::Auto,
            height: Length::Auto,
            margin: Thickness::ZERO,
            flip: false,
            checkerboard: false,
        }
    }

    /// Creates a builder with a procedural RGBA texture.
    ///
    /// `pixels` must contain exactly `width * height * 4` bytes in RGBA order.
    /// If the length does not match, the image is left without a texture.
    pub fn from_rgba(width: u32, height: u32, pixels: impl Into<Vec<u8>>) -> Self {
        Self::new().with_rgba(width, height, pixels)
    }

    /// Sets a procedural RGBA texture.
    ///
    /// `pixels` must contain exactly `width * height * 4` bytes in RGBA order.
    /// If the length does not match, the image is left without a texture.
    pub fn with_rgba(mut self, width: u32, height: u32, pixels: impl Into<Vec<u8>>) -> Self {
        self.texture = TextureResource::from_bytes(
            Uuid::new_v4(),
            TextureKind::Rectangle { width, height },
            TexturePixelKind::RGBA8,
            pixels.into(),
            ResourceKind::Embedded,
        );
        self
    }

    /// Sets the texture to draw.
    pub fn with_texture(mut self, texture: TextureResource) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Sets the fit mode (Fill / Contain / Cover). Defaults to [`ImageFit::Fill`].
    pub fn with_fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Sets an explicit width. When set (alone or with a height) the image is
    /// fitted into the resulting bounds; otherwise it sizes itself to the texture.
    pub fn with_width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets an explicit height.
    pub fn with_height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the outer margin.
    pub fn with_margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Vertically flips the texture (useful for render targets).
    pub fn with_flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    /// Enables the checkerboard background behind transparent pixels.
    pub fn with_checkerboard(mut self, checkerboard: bool) -> Self {
        self.checkerboard = checkerboard;
        self
    }

    /// Builds the image, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let source_size = texture_source_size(self.texture.as_ref());
        let width = self.width.resolve();
        let height = self.height.resolve();

        if width.is_none() && height.is_none() {
            // Native behavior: a single image node sized to the texture.
            let handle: Handle<UiNode> = {
                let mut ctx = cx.ctx();
                let mut builder = ImageBuilder::new(
                    WidgetBuilder::new()
                        .with_name("raikou_image")
                        .with_margin(to_fyrox_thickness(self.margin)),
                )
                .with_flip(self.flip)
                .with_checkerboard_background(self.checkerboard)
                .with_keep_aspect_ratio(true)
                .with_sync_with_texture_size(true);
                if let Some(texture) = &self.texture {
                    builder = builder.with_texture(texture.clone());
                }
                builder.build(&mut ctx).to_base()
            };
            let component = Component {
                handle,
                kind: ComponentKind::Image(ImageHandlers {
                    fit: self.fit,
                    inner: handle,
                }),
            };
            cx.register(&component);
            return component;
        }

        // Explicit bounds: fit the inner image and clip it with a canvas.
        let (bounds_w, bounds_h) = resolve_bounds(width, height, source_size);
        let bounds = Rect::from_xywh(0.0, 0.0, bounds_w, bounds_h);
        let fitted = source_size.map_or(bounds, |source| self.fit.fit_rect(source, bounds));

        let inner: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            ImageBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_image_content")
                    .with_width(fitted.width())
                    .with_height(fitted.height())
                    .with_desired_position(Vector2::new(fitted.x(), fitted.y())),
            )
            .with_opt_texture(self.texture)
            .with_flip(self.flip)
            .with_checkerboard_background(self.checkerboard)
            .with_keep_aspect_ratio(true)
            .with_sync_with_texture_size(false)
            .build(&mut ctx)
            .to_base()
        };

        let wrapper: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            CanvasBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_image")
                    .with_width(bounds_w)
                    .with_height(bounds_h)
                    .with_clip_to_bounds(true)
                    .with_margin(to_fyrox_thickness(self.margin))
                    .with_child(inner),
            )
            .build(&mut ctx)
            .to_base()
        };

        let component = Component {
            handle: wrapper,
            kind: ComponentKind::Image(ImageHandlers {
                fit: self.fit,
                inner,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built image.
pub type ImageHandle = Handle<UiNode>;

/// Updates the fit mode of a built image and re-fits the inner image to the
/// component's current bounds.
pub fn set_image_fit(component: &mut Component, ui: &mut UserInterface, fit: ImageFit) {
    let ComponentKind::Image(handlers) = &mut component.kind else {
        return;
    };
    handlers.fit = fit;
    let inner = handlers.inner;
    apply_fit(ui, component.handle, inner, fit);
}

/// Replaces the texture of a built image and re-fits the inner image using the
/// currently stored fit mode.
pub fn set_image_texture(
    component: &mut Component,
    ui: &mut UserInterface,
    texture: Option<TextureResource>,
) {
    let ComponentKind::Image(handlers) = &component.kind else {
        return;
    };
    let fit = handlers.fit;
    let inner = handlers.inner;
    ui.send(inner, ImageMessage::Texture(texture));
    apply_fit(ui, component.handle, inner, fit);
}

/// Sizes and positions the inner image to `fit_rect(source_size, wrapper_bounds)`.
fn apply_fit(
    ui: &mut UserInterface,
    wrapper: Handle<UiNode>,
    inner: Handle<UiNode>,
    fit: ImageFit,
) {
    let bounds = Rect::from_xywh(
        0.0,
        0.0,
        ui.node(wrapper).width(),
        ui.node(wrapper).height(),
    );
    let source = ui
        .node(inner)
        .cast::<FyroxImage>()
        .and_then(|image| texture_source_size(image.texture.as_ref()));
    let fitted = source.map_or(bounds, |source| fit.fit_rect(source, bounds));

    ui.send(inner, WidgetMessage::Width(fitted.width()));
    ui.send(inner, WidgetMessage::Height(fitted.height()));
    ui.send(
        inner,
        WidgetMessage::DesiredPosition(Vector2::new(fitted.x(), fitted.y())),
    );
}

/// Reads the pixel size of a rectangle texture, when it is loaded.
fn texture_source_size(texture: Option<&TextureResource>) -> Option<Size> {
    let texture = texture?;
    let state = texture.state();
    let data = state.data_ref()?;
    if let TextureKind::Rectangle { width, height } = data.kind() {
        if width > 0 && height > 0 {
            Some(Size::new(width as f32, height as f32))
        } else {
            None
        }
    } else {
        None
    }
}

/// Resolves the wrapper bounds from the requested lengths. When only one axis
/// is fixed and the texture size is known, the other axis is derived to keep
/// the texture's aspect ratio.
fn resolve_bounds(width: Option<f32>, height: Option<f32>, source: Option<Size>) -> (f32, f32) {
    match (width, height, source) {
        (Some(w), Some(h), _) => (w, h),
        (Some(w), None, Some(source)) if source.width > 0.0 && source.height > 0.0 => {
            (w, w * source.height / source.width)
        }
        (Some(w), None, _) => (w, 0.0),
        (None, Some(h), Some(source)) if source.width > 0.0 && source.height > 0.0 => {
            (h * source.width / source.height, h)
        }
        (None, Some(h), _) => (0.0, h),
        (None, None, _) => (0.0, 0.0),
    }
}
