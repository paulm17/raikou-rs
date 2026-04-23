use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::backend::BackendSelection;
use crate::error::{FrameError, RendererError, SurfaceError};
use crate::frame::Frame;

pub struct SkiaRenderer {
    surfaces: SkiaSurfaceManager,
}

impl SkiaRenderer {
    pub fn new(window: &Window) -> Result<Self, RendererError> {
        Ok(Self {
            surfaces: SkiaSurfaceManager::new(window)?,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.surfaces.resize(size, scale_factor);
    }

    pub fn canvas_size(&self) -> PhysicalSize<u32> {
        self.surfaces.size()
    }

    pub fn begin_frame(&mut self) -> Result<Frame<'_>, FrameError> {
        self.surfaces.begin_frame()
    }
}

pub struct SkiaSurfaceManager {
    #[allow(dead_code)]
    backend: BackendSelection,
    metrics: SurfaceMetrics,
    backend_state: PlatformSurfaceState,
}

impl SkiaSurfaceManager {
    pub fn new(window: &Window) -> Result<Self, SurfaceError> {
        let backend = BackendSelection::for_current_platform()?;
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        Ok(Self {
            backend,
            metrics: SurfaceMetrics { size, scale_factor },
            backend_state: PlatformSurfaceState::new(window, size)?,
        })
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.metrics.size
    }

    pub fn scale_factor(&self) -> f64 {
        self.metrics.scale_factor
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.metrics.update(size, scale_factor);
        self.backend_state.resize(size);
    }

    pub(crate) fn acquire_frame(&mut self) -> Result<ActiveFrame<'_>, FrameError> {
        self.metrics.validate_frame_size()?;
        self.backend_state.acquire_frame(self.metrics.size)
    }
}

pub struct ActiveFrame<'a> {
    pub(crate) surface: skia_safe::Surface,
    presenter: FramePresenter<'a>,
}

impl ActiveFrame<'_> {
    pub(crate) fn present(self) -> Result<(), FrameError> {
        self.presenter.present(self.surface);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SurfaceMetrics {
    size: PhysicalSize<u32>,
    scale_factor: f64,
}

impl SurfaceMetrics {
    fn update(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.size = size;
        self.scale_factor = scale_factor;
    }

    fn validate_frame_size(&self) -> Result<(), FrameError> {
        if self.size.width == 0 || self.size.height == 0 {
            return Err(FrameError::ZeroSizedSurface);
        }

        Ok(())
    }
}

enum FramePresenter<'a> {
    Platform(PlatformFramePresenter<'a>),
    #[cfg(test)]
    Test(&'a std::cell::Cell<usize>),
}

impl FramePresenter<'_> {
    fn present(self, surface: skia_safe::Surface) {
        match self {
            Self::Platform(presenter) => presenter.present(surface),
            #[cfg(test)]
            Self::Test(counter) => {
                counter.set(counter.get() + 1);
                drop(surface);
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::NSView;
    use objc2_core_foundation::CGSize;
    use objc2_metal::{
        MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable,
        MTLPixelFormat,
    };
    use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use skia_safe::gpu::{
        backend_render_targets, direct_contexts, mtl, DirectContext, SurfaceOrigin,
    };
    use skia_safe::{ColorType, Surface};
    use winit::dpi::PhysicalSize;
    use winit::window::Window;

    use crate::error::{FrameError, SurfaceError};

    pub(super) struct PlatformSurfaceState {
        metal_layer: Retained<CAMetalLayer>,
        command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        direct_context: DirectContext,
        window: *const Window,
    }

    pub(super) struct PlatformFramePresenter<'a> {
        window: &'a Window,
        command_queue: &'a ProtocolObject<dyn MTLCommandQueue>,
        direct_context: &'a mut DirectContext,
        drawable: Retained<ProtocolObject<dyn CAMetalDrawable>>,
    }

    impl PlatformSurfaceState {
        pub(super) fn new(window: &Window, size: PhysicalSize<u32>) -> Result<Self, SurfaceError> {
            let device =
                MTLCreateSystemDefaultDevice().ok_or(SurfaceError::MetalDeviceUnavailable)?;
            let metal_layer = attach_metal_layer(window, &device, size)?;
            let command_queue = device
                .newCommandQueue()
                .ok_or(SurfaceError::MetalDeviceUnavailable)?;

            let backend = unsafe {
                mtl::BackendContext::new(
                    Retained::as_ptr(&device) as mtl::Handle,
                    Retained::as_ptr(&command_queue) as mtl::Handle,
                )
            };
            let direct_context = direct_contexts::make_metal(&backend, None).ok_or_else(|| {
                SurfaceError::BackendSelection("failed to create Metal context".into())
            })?;

            Ok(Self {
                metal_layer,
                command_queue,
                direct_context,
                window: window as *const Window,
            })
        }

        pub(super) fn resize(&mut self, size: PhysicalSize<u32>) {
            self.metal_layer
                .setDrawableSize(CGSize::new(size.width as f64, size.height as f64));
        }

        pub(super) fn acquire_frame(
            &mut self,
            size: PhysicalSize<u32>,
        ) -> Result<super::ActiveFrame<'_>, FrameError> {
            if size.width == 0 || size.height == 0 {
                return Err(FrameError::ZeroSizedSurface);
            }

            let drawable = self
                .metal_layer
                .nextDrawable()
                .ok_or(FrameError::DrawableUnavailable)?;

            let texture_info = unsafe {
                mtl::TextureInfo::new(Retained::as_ptr(&drawable.texture()) as mtl::Handle)
            };
            let backend_render_target = backend_render_targets::make_mtl(
                (size.width as i32, size.height as i32),
                &texture_info,
            );

            let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                &mut self.direct_context,
                &backend_render_target,
                SurfaceOrigin::TopLeft,
                ColorType::BGRA8888,
                None,
                None,
            )
            .ok_or(FrameError::SurfaceCreationFailed)?;

            Ok(super::ActiveFrame {
                surface,
                presenter: super::FramePresenter::Platform(PlatformFramePresenter {
                    window: unsafe { &*self.window },
                    command_queue: &self.command_queue,
                    direct_context: &mut self.direct_context,
                    drawable,
                }),
            })
        }
    }

    impl PlatformFramePresenter<'_> {
        pub(super) fn present(self, surface: Surface) {
            self.window.pre_present_notify();
            self.direct_context.flush_and_submit();
            drop(surface);

            let command_buffer = self
                .command_queue
                .commandBuffer()
                .expect("failed to create Metal command buffer");
            let drawable: Retained<ProtocolObject<dyn MTLDrawable>> = (&self.drawable).into();
            command_buffer.presentDrawable(&drawable);
            command_buffer.commit();
        }
    }

    fn attach_metal_layer(
        window: &Window,
        device: &ProtocolObject<dyn MTLDevice>,
        size: PhysicalSize<u32>,
    ) -> Result<Retained<CAMetalLayer>, SurfaceError> {
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setPresentsWithTransaction(false);
        layer.setFramebufferOnly(false);
        layer.setDrawableSize(CGSize::new(size.width as f64, size.height as f64));
        layer.setContentsScale(window.scale_factor());

        match window
            .window_handle()
            .map_err(|_| SurfaceError::IncompatibleWindowHandle)?
            .as_raw()
        {
            RawWindowHandle::AppKit(appkit) => {
                let view = unsafe { (appkit.ns_view.as_ptr() as *mut NSView).as_ref() }
                    .ok_or(SurfaceError::IncompatibleWindowHandle)?;
                view.setWantsLayer(true);
                view.setLayer(Some(&layer.clone().into_super()));
                Ok(layer)
            }
            _ => Err(SurfaceError::IncompatibleWindowHandle),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use winit::dpi::PhysicalSize;
    use winit::window::Window;

    use crate::error::{FrameError, SurfaceError};

    pub(super) struct PlatformSurfaceState;
    pub(super) struct PlatformFramePresenter<'a> {
        _phantom: core::marker::PhantomData<&'a ()>,
    }

    impl PlatformSurfaceState {
        pub(super) fn new(
            _window: &Window,
            _size: PhysicalSize<u32>,
        ) -> Result<Self, SurfaceError> {
            Err(SurfaceError::UnsupportedPlatform)
        }

        pub(super) fn resize(&mut self, _size: PhysicalSize<u32>) {}

        pub(super) fn acquire_frame(
            &mut self,
            _size: PhysicalSize<u32>,
        ) -> Result<super::ActiveFrame<'_>, FrameError> {
            Err(FrameError::DrawableUnavailable)
        }
    }

    impl PlatformFramePresenter<'_> {
        pub(super) fn present(self, _surface: skia_safe::Surface) {}
    }
}

use platform::{PlatformFramePresenter, PlatformSurfaceState};

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use skia_safe::surfaces;
    use winit::dpi::PhysicalSize;

    use super::{ActiveFrame, FramePresenter, SurfaceMetrics};
    use crate::frame::Frame;
    use crate::FrameError;

    #[test]
    fn zero_sized_surface_cannot_begin_frame() {
        let metrics = SurfaceMetrics {
            size: PhysicalSize::new(0, 480),
            scale_factor: 2.0,
        };

        assert!(matches!(
            metrics.validate_frame_size(),
            Err(FrameError::ZeroSizedSurface)
        ));
    }

    #[test]
    fn resize_updates_surface_metrics() {
        let mut metrics = SurfaceMetrics {
            size: PhysicalSize::new(800, 600),
            scale_factor: 1.0,
        };

        metrics.update(PhysicalSize::new(1440, 900), 2.0);

        assert_eq!(metrics.size, PhysicalSize::new(1440, 900));
        assert_eq!(metrics.scale_factor, 2.0);
    }

    #[test]
    fn frame_present_calls_presenter_once() {
        let mut surface = surfaces::raster_n32_premul((32, 32)).expect("raster surface");
        surface.canvas().clear(skia_safe::Color::BLUE);

        let counter = Cell::new(0);
        let active = ActiveFrame {
            surface,
            presenter: FramePresenter::Test(&counter),
        };

        let mut frame = Frame::new(active);
        frame.canvas().clear(skia_safe::Color::RED);
        frame.present().expect("present succeeds");

        assert_eq!(counter.get(), 1);
    }
}
