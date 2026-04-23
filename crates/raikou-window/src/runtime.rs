use accesskit::{ActionRequest, TreeUpdate};
use accesskit_winit::Adapter;
use raikou_core::WindowId;
use tracing::debug;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use crate::accessibility::{
    AccessibilityState, ActionBridge, ActivationBridge, DeactivationBridge,
};
use crate::clipboard::{ClipboardBackend, NoopClipboard};
use crate::config::WindowConfig;
use crate::error::RuntimeError;
use crate::events::{RuntimeEvent, WindowEventTranslator};
use crate::window::WindowState;

pub trait RuntimeLifecycle {
    fn on_start(&mut self, _runtime: &mut WindowRuntime) {}

    fn on_event(&mut self, _runtime: &mut WindowRuntime, _event: &RuntimeEvent) {}

    fn on_close_requested(&mut self, runtime: &mut WindowRuntime, window_id: WindowId) {
        let _ = window_id;
        runtime.request_exit();
    }
}

pub struct WindowRuntime {
    config: WindowConfig,
    primary_window: WindowState,
    window: Option<Window>,
    translator: WindowEventTranslator,
    should_exit: bool,
    accessibility_state: AccessibilityState,
    accessibility_adapter: Option<Adapter>,
    needs_redraw: bool,
    needs_animation_frame: bool,
    clipboard: Box<dyn ClipboardBackend>,
}

impl WindowRuntime {
    pub fn new(config: WindowConfig) -> Self {
        let primary_window_id = WindowId::next();
        let (width, height) = config.initial_size;
        Self {
            config,
            primary_window: WindowState::new(
                primary_window_id,
                PhysicalSize::new(width.max(0.0) as u32, height.max(0.0) as u32),
            ),
            window: None,
            translator: WindowEventTranslator::new(),
            should_exit: false,
            accessibility_state: AccessibilityState::new(),
            accessibility_adapter: None,
            needs_redraw: false,
            needs_animation_frame: false,
            clipboard: Box::<NoopClipboard>::default(),
        }
    }

    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    pub fn primary_window(&self) -> &WindowState {
        &self.primary_window
    }

    pub fn primary_window_id(&self) -> WindowId {
        self.primary_window.id()
    }

    pub fn window(&self) -> Option<&Window> {
        self.window.as_ref()
    }

    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub fn request_animation_frame(&mut self) {
        self.needs_animation_frame = true;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub fn clear_animation_frame(&mut self) {
        self.needs_animation_frame = false;
    }

    pub fn clear_redraw(&mut self) {
        self.needs_redraw = false;
    }

    pub fn should_redraw(&self) -> bool {
        self.needs_redraw || self.needs_animation_frame
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn request_exit(&mut self) {
        self.should_exit = true;
    }

    pub fn update_accessibility(&mut self, update: TreeUpdate) {
        self.accessibility_state.set_tree_update(update.clone());
        if let Some(adapter) = &mut self.accessibility_adapter {
            adapter.update_if_active(|| update);
        }
    }

    pub fn drain_accessibility_action_requests(&self) -> Vec<ActionRequest> {
        self.accessibility_state.drain_action_requests()
    }

    pub fn translate_window_event(&mut self, event: &WinitWindowEvent) -> Vec<RuntimeEvent> {
        let window_id = self.primary_window.id();
        let events = self.translator.translate(window_id, event);
        self.apply_runtime_events(&events);
        events
    }

    pub fn apply_scale_factor(
        &mut self,
        scale_factor: f64,
        suggested_inner_size: Option<PhysicalSize<u32>>,
    ) -> RuntimeEvent {
        self.primary_window.set_scale_factor(scale_factor);
        if let Some(size) = suggested_inner_size {
            self.primary_window.set_inner_size(size);
        }

        RuntimeEvent::ScaleFactorChanged(crate::events::ScaleFactorChangedEvent {
            window_id: self.primary_window.id(),
            scale_factor,
            suggested_inner_size,
        })
    }

    pub fn set_primary_window_binding(
        &mut self,
        raw_id: winit::window::WindowId,
        inner_size: PhysicalSize<u32>,
        scale_factor: f64,
    ) {
        self.primary_window.bind_raw_id(raw_id);
        self.primary_window.set_inner_size(inner_size);
        self.primary_window.set_scale_factor(scale_factor);
    }

    pub fn run<L>(self, lifecycle: L) -> Result<(), RuntimeError>
    where
        L: RuntimeLifecycle + 'static,
    {
        self.run_with_clipboard(lifecycle, Box::<NoopClipboard>::default())
    }

    pub fn run_with_clipboard<L>(
        mut self,
        lifecycle: L,
        clipboard: Box<dyn ClipboardBackend>,
    ) -> Result<(), RuntimeError>
    where
        L: RuntimeLifecycle + 'static,
    {
        self.clipboard = clipboard;
        let event_loop = EventLoop::<()>::new()?;
        let mut app = RunningApp {
            runtime: self,
            lifecycle,
            started: false,
        };

        event_loop.run_app(&mut app)?;

        Ok(())
    }

    fn apply_runtime_events(&mut self, events: &[RuntimeEvent]) {
        for event in events {
            match event {
                RuntimeEvent::Resize(resize) => {
                    self.primary_window.set_inner_size(resize.size);
                }
                RuntimeEvent::ScaleFactorChanged(scale) => {
                    self.primary_window.set_scale_factor(scale.scale_factor);
                    if let Some(size) = scale.suggested_inner_size {
                        self.primary_window.set_inner_size(size);
                    }
                }
                RuntimeEvent::CloseRequested(_) => {
                    self.primary_window.mark_close_requested();
                }
                _ => {}
            }
        }
    }
}

struct RunningApp<L> {
    runtime: WindowRuntime,
    lifecycle: L,
    started: bool,
}

impl<L> ApplicationHandler for RunningApp<L>
where
    L: RuntimeLifecycle + 'static,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);

        if self.runtime.window.is_none() {
            let window = event_loop
                .create_window(
                    self.runtime
                        .config
                        .clone()
                        .visible(false)
                        .to_window_attributes(),
                )
                .expect("failed to build primary window");
            let size = window.inner_size();
            let scale_factor = window.scale_factor();
            self.runtime
                .set_primary_window_binding(window.id(), size, scale_factor);
            self.runtime.accessibility_adapter = Some(Adapter::with_direct_handlers(
                &window,
                ActivationBridge::new(self.runtime.accessibility_state.clone()),
                ActionBridge::new(self.runtime.accessibility_state.clone()),
                DeactivationBridge,
            ));
            self.runtime.window = Some(window);
        }

        if !self.started {
            self.lifecycle.on_start(&mut self.runtime);
            self.started = true;
        }

        if let Some(window) = &self.runtime.window {
            window.set_visible(self.runtime.config.visible);
        }

        if self.runtime.should_exit() {
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        if self.runtime.primary_window.raw_id() != Some(window_id) {
            return;
        }

        let translated = self.runtime.translate_window_event(&event);
        if let Some(window) = &self.runtime.window
            && let Some(adapter) = &mut self.runtime.accessibility_adapter
        {
            adapter.process_event(window, &event);
        }
        for runtime_event in &translated {
            if let RuntimeEvent::CloseRequested(id) = runtime_event {
                self.lifecycle.on_close_requested(&mut self.runtime, *id);
            }

            self.lifecycle.on_event(&mut self.runtime, runtime_event);
        }

        if self.runtime.should_exit() {
            event_loop.exit();
            return;
        }

        if let Some(window) = &self.runtime.window
            && matches!(
                event,
                WinitWindowEvent::Resized(_) | WinitWindowEvent::ScaleFactorChanged { .. }
            )
        {
            debug!("requesting redraw after surface-affecting window change");
            window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if (self.runtime.needs_redraw || self.runtime.needs_animation_frame)
            && let Some(window) = &self.runtime.window
        {
            window.request_redraw();
        }

        if self.runtime.should_exit() {
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.runtime.request_exit();
    }
}