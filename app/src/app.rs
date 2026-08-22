use crate::hotreload::{ApplyKind, HotReload};
use crate::screens::main_screen::MainScreen;
use fyrox::{
    asset::{io::FsResourceIo, manager::ResourceManager},
    core::{algebra::Matrix3, task::TaskPool},
    engine::{
        Engine, EngineInitParams, GraphicsContext, GraphicsContextParams, SerializationContext,
    },
    gui::{
        constructor::new_widget_constructor_container, font::BUILT_IN_FONT, message::UiMessage,
        widget::WidgetMessage, RenderMode, UserInterface,
    },
    window::WindowAttributes,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

pub struct UpdateLoopState(u32);

impl Default for UpdateLoopState {
    fn default() -> Self {
        // Run at least a second from the start to ensure that all OS-specific stuff was done.
        Self(60)
    }
}

impl UpdateLoopState {
    pub fn request_update_in_next_frame(&mut self) {
        if !self.is_warming_up() {
            self.0 = 3;
        }
    }

    pub fn request_update_in_current_frame(&mut self) {
        if !self.is_warming_up() {
            self.0 = 1;
        }
    }

    pub fn is_warming_up(&self) -> bool {
        self.0 > 2
    }

    pub fn decrease_counter(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    pub fn is_suspended(&self) -> bool {
        self.0 == 0
    }
}

/// Counters used to prove the redraw discipline of the tool-loop + `RenderMode::OnChanges`.
#[derive(Default, Clone, Copy, Debug)]
pub struct Stats {
    /// Number of update passes that actually ran.
    pub updates: u64,
    /// Number of times `engine.render()` was called (frame swaps).
    pub renders: u64,
    /// Number of times the UI was actually re-uploaded to the GPU. Under
    /// `RenderMode::OnChanges` this is driven solely by `UserInterface::need_render`.
    pub ui_renders: u64,
    /// Number of UI messages processed by the poll loop.
    pub messages: u64,
    /// Number of non-redraw window events received.
    pub events: u64,
}

pub struct App {
    pub engine: Engine,
    pub state: AppState,
}

pub struct AppState {
    pub update_loop_state: UpdateLoopState,
    pub focused: bool,
    pub stats: Stats,
    pub stats_enabled: bool,
    pub stats_last: Instant,
    pub screen: Option<MainScreen>,
    /// True when the app was launched with `--ui <path>`: the UI is loaded from a
    /// `.ui` file and watched for live edits.
    pub ui_mode: bool,
    /// Path of the `.ui` file loaded in `--ui` mode.
    pub ui_path: Option<PathBuf>,
    /// Active hot-reload watcher (present in `--ui` mode).
    pub hotreload: Option<HotReload>,
    /// Set when the 3D scene changed this frame and needs a re-render. Drives the
    /// RedrawRequested gate alongside `ui.need_render` (scene-only frames render
    /// the scene behind the UI without re-uploading the UI texture).
    pub scene_need_render: bool,
}

impl App {
    pub fn new(
        window_attributes: WindowAttributes,
    ) -> Result<Self, fyrox::engine::error::EngineError> {
        let serialization_context = Arc::new(SerializationContext::new());
        let task_pool = Arc::new(TaskPool::new());
        let mut engine = Engine::new(EngineInitParams {
            graphics_context_params: GraphicsContextParams {
                window_attributes,
                vsync: true,
                msaa_sample_count: Some(2),
                graphics_server_constructor: Default::default(),
                named_objects: false,
            },
            resource_manager: ResourceManager::new(Arc::new(FsResourceIo), task_pool.clone()),
            serialization_context,
            task_pool,
            widget_constructors: Arc::new(new_widget_constructor_container()),
            dyn_type_constructors: Arc::new(Default::default()),
        })?;

        engine
            .user_interfaces
            .add(UserInterface::new(fyrox::core::algebra::Vector2::repeat(
                100.0,
            )));

        Ok(Self {
            engine,
            state: AppState {
                update_loop_state: Default::default(),
                focused: true,
                stats: Default::default(),
                stats_enabled: false,
                stats_last: Instant::now(),
                screen: None,
                ui_mode: false,
                ui_path: None,
                hotreload: None,
                scene_need_render: false,
            },
        })
    }

    pub fn set_default_font(ui: &mut UserInterface) {
        // Use Fyrox's built-in font (same built_in_font.ttf the app used before).
        // A font built from raw memory has a random UUID and no file path, so it
        // serializes into a `.ui` file as an opaque embedded resource that cannot be
        // resolved on load (uninitialized Font -> "Font reader must be initialized!").
        // The built-in font has a stable, registered UUID, so exported `.ui` files
        // round-trip cleanly.
        ui.default_font = BUILT_IN_FONT.resource();
    }

    pub fn set_ui_scaling(ui: &UserInterface, scale: f32) {
        // High-DPI screen support
        ui.send(
            ui.root(),
            WidgetMessage::RenderTransform(Matrix3::new_scaling(scale)),
        );
    }

    pub fn render_mode_is_on_changes(ui: &mut UserInterface) {
        ui.render_mode = RenderMode::OnChanges;
    }
}

impl AppState {
    pub fn is_active(&self, ui: &UserInterface) -> bool {
        // Keep the loop alive while the 3D scene is animating: the scene-render flag
        // is cleared by the render, so without this the idle gate would stop the
        // loop after the first animated frame.
        if self
            .screen
            .as_ref()
            .map(|s| s.spin || s.orbit || s.always_spin)
            .unwrap_or(false)
        {
            return true;
        }
        !self.update_loop_state.is_suspended() && self.focused || ui.captured_node().is_some()
    }

    pub fn handle_ui_message(&mut self, message: &UiMessage, ui: &mut UserInterface) {
        if let Some(screen) = self.screen.as_mut() {
            screen.handle_ui_message(message, ui);
        }
    }

    /// Drives the hot-reload watcher: pulls filesystem events, applies the new UI
    /// when a load or reload completes. Called every update in `--ui` mode.
    pub fn pump_hotreload(&mut self, engine: &mut Engine) {
        if !self.ui_mode {
            return;
        }
        if self.hotreload.is_none() {
            self.hotreload = self.setup_hotreload(engine);
        }
        let kind = self.hotreload.as_mut().map(|hr| hr.pump(engine));
        if matches!(kind, Some(ApplyKind::Load) | Some(ApplyKind::Reload)) {
            let new_ui = self.hotreload.as_mut().and_then(|hr| hr.take_ui());
            if let Some(new_ui) = new_ui {
                self.apply_loaded_ui(engine, new_ui);
            }
        }
    }

    fn setup_hotreload(&mut self, engine: &mut Engine) -> Option<HotReload> {
        let path = self.ui_path.clone()?;
        HotReload::setup(engine, path, true).ok()
    }

    /// Saves the current UI to a `.ui` file. Swaps the default font to the built-in
    /// font first: our runtime-installed embedded font has no resource path and
    /// would not resolve when the file is loaded back.
    pub fn export_ui(engine: &mut Engine, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let ui = engine.user_interfaces.first_mut();
        ui.default_font = BUILT_IN_FONT.resource();
        ui.save(path)?;
        Ok(())
    }

    /// Swaps a freshly loaded `UserInterface` into the engine, rebuilding the
    /// `MainScreen` handle map from widget names and pushing the preserved runtime
    /// state back into the widgets via messages.
    fn apply_loaded_ui(&mut self, engine: &mut Engine, new_ui: UserInterface) {
        let (name, clicks) = self
            .screen
            .as_ref()
            .map(|s| (s.name.clone(), s.clicks))
            .unwrap_or(("Alice".to_string(), 0));

        engine.user_interfaces.clear();
        engine.user_interfaces.add(new_ui);

        let ui = engine.user_interfaces.first_mut();
        App::set_default_font(ui);
        App::render_mode_is_on_changes(ui);
        let screen = MainScreen::from_loaded_ui(ui, &name, clicks);
        screen.push_state(ui);
        ui.need_render = true;
        self.screen = Some(screen);

        let scale = if let GraphicsContext::Initialized(ctx) = &engine.graphics_context {
            Some(ctx.window.scale_factor() as f32)
        } else {
            None
        };
        if let Some(scale) = scale {
            App::set_ui_scaling(engine.user_interfaces.first_mut(), scale);
        }
    }
}
