use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use raikou_core::{ImageFit, Rect, Size};
use skia_safe::images;
use skia_safe::{AlphaType, ColorType, Data, ImageInfo};

use crate::error::ImageError;

#[derive(Clone, Debug)]
pub struct Image {
    inner: Arc<ImageInner>,
}

#[derive(Debug)]
struct ImageInner {
    skia: skia_safe::Image,
}

impl Image {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        let data = Data::new_copy(bytes);
        let image =
            images::deferred_from_encoded_data(data, None).ok_or(ImageError::DecodeFailed)?;
        Ok(Self::from_skia(image))
    }

    pub fn from_rgba(width: u32, height: u32, pixels: &[u8]) -> Result<Self, ImageError> {
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            return Err(ImageError::InvalidRgbaDimensions);
        }

        let info = ImageInfo::new(
            (width as i32, height as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let data = Data::new_copy(pixels);
        let image = images::raster_from_data(&info, data, width as usize * 4)
            .ok_or(ImageError::DecodeFailed)?;

        Ok(Self::from_skia(image))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let path = path.as_ref().to_path_buf();
        let cache = file_image_cache();

        if let Some(image) = cache
            .lock()
            .expect("image cache poisoned")
            .get(&path)
            .and_then(Weak::upgrade)
        {
            return Ok(Self { inner: image });
        }

        let bytes = std::fs::read(&path)?;
        let image = Self::from_bytes(&bytes)?;
        cache
            .lock()
            .expect("image cache poisoned")
            .insert(path, Arc::downgrade(&image.inner));
        Ok(image)
    }

    pub fn width(&self) -> i32 {
        self.inner.skia.width()
    }

    pub fn height(&self) -> i32 {
        self.inner.skia.height()
    }

    pub fn size(&self) -> Size {
        Size::new(self.width() as f32, self.height() as f32)
    }

    pub fn shares_resource_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn fit_rect(&self, bounds: Rect, fit: ImageFit) -> Rect {
        if self.width() <= 0 || self.height() <= 0 {
            return bounds;
        }

        let image_aspect = self.width() as f32 / self.height() as f32;
        let bounds_aspect = bounds.width() / bounds.height();

        let (width, height) = match fit {
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

    pub(crate) fn as_skia(&self) -> &skia_safe::Image {
        &self.inner.skia
    }

    pub(crate) fn from_skia(image: skia_safe::Image) -> Self {
        Self {
            inner: Arc::new(ImageInner { skia: image }),
        }
    }
}

fn file_image_cache() -> &'static Mutex<HashMap<PathBuf, Weak<ImageInner>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Weak<ImageInner>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
