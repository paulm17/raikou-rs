use raikou_core::WindowId;
use winit::dpi::PhysicalSize;
use winit::window::WindowId as RawWindowId;

#[derive(Debug)]
pub struct WindowState {
    framework_id: WindowId,
    raw_id: Option<RawWindowId>,
    inner_size: PhysicalSize<u32>,
    scale_factor: f64,
    close_requested: bool,
}

impl WindowState {
    pub(crate) fn new(framework_id: WindowId, inner_size: PhysicalSize<u32>) -> Self {
        Self {
            framework_id,
            raw_id: None,
            inner_size,
            scale_factor: 1.0,
            close_requested: false,
        }
    }

    pub fn id(&self) -> WindowId {
        self.framework_id
    }

    pub fn raw_id(&self) -> Option<RawWindowId> {
        self.raw_id
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.inner_size
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub(crate) fn bind_raw_id(&mut self, raw_id: RawWindowId) {
        self.raw_id = Some(raw_id);
    }

    pub(crate) fn set_inner_size(&mut self, inner_size: PhysicalSize<u32>) {
        self.inner_size = inner_size;
    }

    pub(crate) fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    pub(crate) fn mark_close_requested(&mut self) {
        self.close_requested = true;
    }
}
