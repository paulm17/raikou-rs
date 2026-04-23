use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error(transparent)]
    Surface(#[from] SurfaceError),
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error(transparent)]
    Image(#[from] ImageError),
}

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("backend selection failed: {0}")]
    BackendSelection(String),
    #[error("no supported graphics backend is available on this platform")]
    UnsupportedPlatform,
    #[error("failed to create a Metal device")]
    MetalDeviceUnavailable,
    #[error("window handle is not compatible with the Metal backend")]
    IncompatibleWindowHandle,
    #[error("window surface is zero-sized")]
    ZeroSizedSurface,
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("window surface is zero-sized")]
    ZeroSizedSurface,
    #[error("failed to acquire a Metal drawable")]
    DrawableUnavailable,
    #[error("failed to create a Skia surface for the current drawable")]
    SurfaceCreationFailed,
}

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("image bytes could not be decoded")]
    DecodeFailed,
    #[error("rgba image dimensions do not match pixel data length")]
    InvalidRgbaDimensions,
    #[error("failed to read image file: {0}")]
    Io(#[from] std::io::Error),
}
