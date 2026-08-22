//! raikou-demo — a shared tool-loop bootstrap for the per-component example
//! binaries.
//!
//! Every `examples/*_demo` shows a single raikou component in a standalone
//! window. Rather than copy the ~270 lines of fyrox engine/winit bootstrap
//! into each demo, this crate wraps it once and lets each demo supply only:
//!
//! * a `title` / window size, and
//! * a closure that builds the component tree and returns the root handle.
//!
//! The shared loop creates a `Theme` + `ComponentRegistry`, links the built
//! panel to the UI root, polls the message queue and dispatches into the
//! registry on every frame (mirroring the app's tool-loop pattern).

#![allow(deprecated)]

use std::sync::Arc;

use fyrox::asset::{io::FsResourceIo, manager::ResourceManager};
use fyrox::core::algebra::{Matrix3, Vector2};
use fyrox::core::color::Color;
use fyrox::core::pool::Handle;
use fyrox::core::task::TaskPool;
use fyrox::dpi::PhysicalSize;
use fyrox::engine::{
    ApplicationLoopController, Engine, EngineInitParams, GraphicsContext, GraphicsContextParams,
    SerializationContext,
};
use fyrox::event::{Event, WindowEvent};
use fyrox::event_loop::{ControlFlow, EventLoop};
use fyrox::gui::constructor::new_widget_constructor_container;
use fyrox::gui::font::BUILT_IN_FONT;
use fyrox::gui::message::UiMessage;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{RenderMode, UiNode, UserInterface};
use fyrox::utils::translate_event;
use fyrox::window::WindowAttributes;
use raikou::prelude::*;

#[cfg(target_os = "macos")]
mod screenshot;

/// Configuration for a demo window.
pub struct Options {
    /// Window title.
    pub title: String,
    /// Initial window width.
    pub width: u32,
    /// Initial window height.
    pub height: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            title: "raikou demo".to_string(),
            width: 900,
            height: 600,
        }
    }
}

/// Builds the component tree for a demo. Receives the live `UserInterface`,
/// the resolved `Theme`, and the `ComponentRegistry` (already wired to
/// dispatch on every polled message).
pub type PanelBuilder<'a> = dyn FnOnce(
        &mut UserInterface,
        &Theme,
        &mut ComponentRegistry,
    ) -> Handle<UiNode>
    + 'a;

/// Runs a demo window: boots the fyrox engine with a single `UserInterface`,
/// builds the panel via `build`, and drives the tool loop until the window
/// closes.
pub fn run(options: Options, build: Box<PanelBuilder>) {
    let mut window_attributes = WindowAttributes::default();
    window_attributes.inner_size = Some(PhysicalSize::new(options.width, options.height).into());
    window_attributes.resizable = true;
    window_attributes.title = options.title.clone();

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
    })
    .unwrap();

    engine
        .user_interfaces
        .add(UserInterface::new(Vector2::repeat(100.0)));

    {
        let ui = engine.user_interfaces.first_mut();
        ui.default_font = BUILT_IN_FONT.resource();
        ui.render_mode = RenderMode::OnChanges;
    }

    let mut registry = ComponentRegistry::default();
    // RAIKOU_THEME=light|dark selects the Avalonia Fluent variant (default: light).
    let dark = matches!(std::env::var("RAIKOU_THEME").as_deref(), Ok("dark"));
    let theme = if dark {
        Theme::fluent_dark()
    } else {
        Theme::fluent_light()
    };
    eprintln!(
        "raikou: Avalonia Fluent {} theme (set RAIKOU_THEME={} to switch)",
        if dark { "dark" } else { "light" },
        if dark { "light" } else { "dark" }
    );
    {
        // Map the theme onto fyrox's global style so all natively-styled
        // widgets (text boxes, dropdowns, decorators...) inherit Fluent
        // colors instead of the stock dark palette.
        use raikou_style::theme::fyrox_bridge::fluent_fyrox_style_resource;
        engine
            .user_interfaces
            .first_mut()
            .set_style(fluent_fyrox_style_resource(&theme, dark));
    }
    let panel = build(engine.user_interfaces.first_mut(), &theme, &mut registry);
    {
        let ui = engine.user_interfaces.first_mut();
        let root = ui.root();
        let mut ctx = ui.build_ctx();
        ctx.link(panel, root);
        // The UI root is a free-form Canvas that arranges children at their
        // desired size, so the app panel must be sized explicitly or every
        // unpainted region of the window shows the raw clear color.
        ui.send(panel, WidgetMessage::Width(options.width as f32));
        ui.send(panel, WidgetMessage::Height(options.height as f32));
        ui.need_render = true;
    }
    let app_panel = panel;

    // Screenshot-harness support: exit automatically after N seconds so
    // scripts/shot.sh can capture a window without killing the process.
    let auto_quit_secs: Option<f32> = std::env::var("RAIKOU_AUTO_QUIT_SECS")
        .ok()
        .and_then(|v| v.parse().ok());
    let started = std::time::Instant::now();

    // In-process self-capture (no Screen Recording permission needed):
    // RAIKOU_SHOT_OUT=<path.png> writes a PNG of the window at
    // RAIKOU_SHOT_AT_SECS (default 2.0) after launch.
    let shot_out = std::env::var("RAIKOU_SHOT_OUT").ok();
    let shot_at: f32 = std::env::var("RAIKOU_SHOT_AT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2.0);
    let mut shot_done = false;

    let event_loop = EventLoop::new().unwrap();

    event_loop
        .run(move |event, active_event_loop| {
            active_event_loop.set_control_flow(ControlFlow::Wait);

            if let Some(secs) = auto_quit_secs {
                if started.elapsed().as_secs_f32() >= secs {
                    active_event_loop.exit();
                    return;
                }
            }

            match event {
                Event::Resumed => {
                    engine
                        .initialize_graphics_context(active_event_loop)
                        .expect("Unable to initialize graphics context!");
                    let scale = engine
                        .graphics_context
                        .as_initialized_ref()
                        .window
                        .scale_factor() as f32;
                    let ui = engine.user_interfaces.first_mut();
                    ui.send(
                        ui.root(),
                        WidgetMessage::RenderTransform(Matrix3::new_scaling(scale)),
                    );
                    ui.need_render = true;
                }
                Event::Suspended => {
                    engine
                        .destroy_graphics_context()
                        .expect("Unable to destroy graphics context!");
                }
                Event::AboutToWait => {
                    let time_step = 1.0 / 60.0;
                    {
                        let ui = engine.user_interfaces.first_mut();
                        loop {
                            let poll_result = ui.poll_message_queue();
                            if let Some(message) = poll_result.message {
                                dispatch_message(ui, &mut registry, &message);
                            } else {
                                break;
                            }
                        }
                    }
                    // engine.update drives UserInterface::update -> update_layout,
                    // which measures/arranges every node and processes pending
                    // messages. Without it the UI has no geometry and renders as a
                    // blank screen.
                    engine.update(
                        time_step,
                        ApplicationLoopController::ActiveEventLoop(active_event_loop),
                        &mut 0.0,
                        Default::default(),
                    );
                    if let GraphicsContext::Initialized(ctx) = &engine.graphics_context {
                        ctx.window.request_redraw();
                    }
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Focused(focused) => {
                            if focused {
                                engine.user_interfaces.first_mut().need_render = true;
                            }
                        }
                        WindowEvent::Occluded(occluded) => {
                            if !occluded {
                                engine.user_interfaces.first_mut().need_render = true;
                            }
                        }
                        WindowEvent::CloseRequested => active_event_loop.exit(),
                        WindowEvent::Resized(size) => {
                            if let Err(e) = engine.set_frame_size(size.into()) {
                                eprintln!("Unable to set frame size: {e:?}");
                            }
                            let window = &engine.graphics_context.as_initialized_ref().window;
                            let logical_size = size.to_logical(window.scale_factor());
                            let ui = engine.user_interfaces.first_mut();
                            ui.send_many(
                                app_panel,
                                [
                                    WidgetMessage::Width(logical_size.width),
                                    WidgetMessage::Height(logical_size.height),
                                ],
                            );
                            ui.need_render = true;
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            let ui = engine.user_interfaces.first_mut();
                            ui.send(
                                ui.root(),
                                WidgetMessage::RenderTransform(Matrix3::new_scaling(
                                    scale_factor as f32,
                                )),
                            );
                            ui.need_render = true;
                        }
                        WindowEvent::RedrawRequested => {
                            let need_render = engine.user_interfaces.first().need_render;
                            if need_render {
                                engine.render().unwrap();
                            }

                            // The last rendered frame persists on-screen, so
                            // capture even when this frame didn't re-render.
                            if !shot_done && started.elapsed().as_secs_f32() >= shot_at {
                                shot_done = true;
                                if let Some(path) = &shot_out {
                                    if let GraphicsContext::Initialized(ctx) =
                                        &mut engine.graphics_context
                                    {
                                        let size = ctx.window.inner_size();
                                        let clear = match std::env::var("RAIKOU_CLEAR")
                                            .as_deref()
                                        {
                                            Ok("black") => Color::BLACK,
                                            Ok("dark") => Color::from_rgba(0x20, 0x20, 0x20, 255),
                                            _ => match std::env::var("RAIKOU_THEME")
                                                .as_deref()
                                            {
                                                Ok("dark") => {
                                                    Color::from_rgba(0x20, 0x20, 0x20, 255)
                                                }
                                                _ => Color::WHITE,
                                            },
                                        };
                                        let ui = engine.user_interfaces.first();
                                        if let Err(e) = screenshot::capture_ui_to_png(
                                            &mut ctx.renderer,
                                            ui,
                                            &engine.resource_manager,
                                            size.width,
                                            size.height,
                                            clear,
                                            path,
                                        ) {
                                            eprintln!("screenshot failed: {e}");
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    if let Some(os_event) = translate_event(&event) {
                        for ui in engine.user_interfaces.iter_mut() {
                            ui.process_os_event(&os_event);
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

/// Default dispatcher: routes each polled message into the raikou registry so
/// component handlers fire. Demos that need extra message handling wrap this
/// with their own logic before calling it.
pub fn dispatch_message(ui: &mut UserInterface, registry: &mut ComponentRegistry, message: &UiMessage) {
    registry.dispatch(ui, message);
}
