#![allow(deprecated)]

mod app;
mod hotreload;
mod scene;
mod screens;
mod ui;

use crate::app::App;
use fyrox::{
    dpi::PhysicalSize,
    engine::{ApplicationLoopController, GraphicsContext},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    utils::translate_event,
    window::WindowAttributes,
};
use std::{path::PathBuf, time::Instant};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut export_path: Option<String> = None;
    let mut ui_path: Option<String> = None;
    let mut stats_enabled = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stats" => stats_enabled = true,
            "--export-ui" => export_path = args.next(),
            "--ui" => ui_path = args.next(),
            _ => {}
        }
    }

    let mut window_attributes = WindowAttributes::default();
    window_attributes.inner_size = Some(PhysicalSize::new(900, 600).into());
    window_attributes.resizable = true;
    window_attributes.title = "raikou".to_string();

    let mut app = App::new(window_attributes).unwrap();
    app.state.stats_enabled = stats_enabled;

    App::set_default_font(app.engine.user_interfaces.first_mut());
    App::render_mode_is_on_changes(app.engine.user_interfaces.first_mut());
    let screen = ui::build(app.engine.user_interfaces.first_mut());
    app.state.screen = Some(screen);

    if let Some(path) = export_path {
        let path = std::path::PathBuf::from(path);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = crate::app::AppState::export_ui(&mut app.engine, &path) {
            eprintln!("export failed: {e}");
            std::process::exit(1);
        }
        println!("exported ui to {}", path.display());
        return;
    }

    if let Some(path) = ui_path {
        app.state.ui_mode = true;
        app.state.ui_path = Some(PathBuf::from(path));
    }

    let mut scene3d = crate::scene::Scene3D::build(&mut app.engine);

    let event_loop = EventLoop::new().unwrap();

    let mut previous = Instant::now();

    event_loop
        .run(move |event, active_event_loop| {
            let App { engine, state } = &mut app;

            let time_step = 1.0 / 60.0;
            // Wait for an event instead of redrawing every frame.
            active_event_loop.set_control_flow(ControlFlow::Wait);

            match event {
                Event::Resumed => {
                    engine
                        .initialize_graphics_context(active_event_loop)
                        .expect("Unable to initialize graphics context!");
                    let graphics_context = engine.graphics_context.as_initialized_mut();
                    App::set_ui_scaling(
                        engine.user_interfaces.first(),
                        graphics_context.window.scale_factor() as f32,
                    );
                    engine.user_interfaces.first_mut().need_render = true;
                }
                Event::Suspended => {
                    engine
                        .destroy_graphics_context()
                        .expect("Unable to destroy graphics context!");
                }
                Event::AboutToWait => {
                    let ui = engine.user_interfaces.first_mut();

                    if state.is_active(ui) {
                        let elapsed = previous.elapsed();
                        if elapsed.as_secs_f32() >= time_step {
                            let mut processed = 0;
                            loop {
                                let poll_result = ui.poll_message_queue();
                                if let Some(message) = poll_result.message {
                                    state.handle_ui_message(&message, ui);
                                } else {
                                    break;
                                }
                                processed += poll_result.processed_messages;
                            }
                            if processed > 0 {
                                state.update_loop_state.request_update_in_next_frame();
                            }
                            state.stats.messages += processed as u64;

                            engine.update(
                                time_step,
                                ApplicationLoopController::ActiveEventLoop(active_event_loop),
                                &mut 0.0,
                                Default::default(),
                            );

                            if let Some(screen) = state.screen.as_mut() {
                                screen.update(engine.user_interfaces.first_mut());
                            }
                            // Push the UI toggle state into the 3D scene, then advance the
                            // scene only if something is animating. The scene-render flag
                            // keeps the loop alive and re-renders the scene (which is drawn
                            // behind the UI) when it changes, while the UI itself stays
                            // untouched (OnChanges keeps the previous UI texture).
                            if let Some(screen) = state.screen.as_ref() {
                                scene3d.spin = screen.spin;
                                scene3d.orbit = screen.orbit;
                                scene3d.always_spin = screen.always_spin;
                            }
                            if scene3d.update(time_step) {
                                state.scene_need_render = true;
                            }
                            if state.scene_need_render {
                                scene3d.apply(engine);
                            }
                            state.pump_hotreload(engine);
                            state.stats.updates += 1;

                            state.update_loop_state.decrease_counter();

                            // Log a single line at the exact moment the loop goes idle, so the
                            // frozen-counters state is observable in the log.
                            if state.stats_enabled && state.update_loop_state.is_suspended() {
                                let s = &state.stats;
                                let clicks = state
                                    .screen
                                    .as_ref()
                                    .map(|screen| screen.clicks)
                                    .unwrap_or(0);
                                println!(
                                    "idle: updates={} renders={} ui_renders={} messages={} events={} clicks={}",
                                    s.updates, s.renders, s.ui_renders, s.messages, s.events,
                                    clicks
                                );
                            }

                            if state.stats_enabled && state.stats_last.elapsed().as_secs_f32() >= 2.0
                            {
                                let s = &state.stats;
                                let clicks = state
                                    .screen
                                    .as_ref()
                                    .map(|screen| screen.clicks)
                                    .unwrap_or(0);
                                let line = format!(
                                    "stats: updates={} renders={} ui_renders={} messages={} events={} clicks={}",
                                    s.updates, s.renders, s.ui_renders, s.messages, s.events,
                                    clicks
                                );
                                println!("{line}");
                                state.stats_last = Instant::now();
                            }

                            previous = Instant::now();
                        }

                        if let GraphicsContext::Initialized(ref ctx) = engine.graphics_context {
                            let window = &ctx.window;
                            window.set_cursor_icon(fyrox::utils::translate_cursor_icon(
                                engine.user_interfaces.first_mut().cursor(),
                            ));
                            ctx.window.request_redraw();
                        }
                    }
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Focused(focused) => {
                            state.focused = focused;
                            if focused {
                                // Repaint after regaining focus (the OS may have cleared or
                                // invalidated the window content while it was unfocused).
                                engine.user_interfaces.first_mut().need_render = true;
                            }
                        }
                        WindowEvent::Occluded(occluded) => {
                            if !occluded {
                                // The window became visible again; force a repaint even though no
                                // UI message triggered it.
                                engine.user_interfaces.first_mut().need_render = true;
                            }
                        }
                        WindowEvent::CloseRequested => {
                            if state.stats_enabled {
                                let s = &state.stats;
                                let clicks = state
                                    .screen
                                    .as_ref()
                                    .map(|screen| screen.clicks)
                                    .unwrap_or(0);
                                println!(
                                    "final stats: updates={} renders={} ui_renders={} messages={} events={} clicks={}",
                                    s.updates, s.renders, s.ui_renders, s.messages, s.events,
                                    clicks
                                );
                            }
                            active_event_loop.exit();
                        }
                        WindowEvent::Resized(size) => {
                            if let Err(e) = engine.set_frame_size(size.into()) {
                                eprintln!("Unable to set frame size: {e:?}");
                            }
                            let window = &engine.graphics_context.as_initialized_ref().window;
                            let logical_size = size.to_logical(window.scale_factor());
                            let ui = engine.user_interfaces.first_mut();
                            ui.send_many(
                                ui.root(),
                                [
                                    fyrox::gui::widget::WidgetMessage::Width(logical_size.width),
                                    fyrox::gui::widget::WidgetMessage::Height(logical_size.height),
                                ],
                            );
                            // Bound the app layout to the window so it cannot grow
                            // beyond the window when content changes (e.g. a long
                            // greeting string pushing the list off-screen).
                            if let Some(screen) = state.screen.as_ref() {
                                ui.send_many(
                                    screen.root,
                                    [
                                        fyrox::gui::widget::WidgetMessage::Width(logical_size.width),
                                        fyrox::gui::widget::WidgetMessage::Height(
                                            logical_size.height,
                                        ),
                                    ],
                                );
                            }
                            ui.need_render = true;
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            let ui = engine.user_interfaces.first_mut();
                            App::set_ui_scaling(ui, scale_factor as f32);
                            ui.need_render = true;
                        }
                        WindowEvent::RedrawRequested => {
                            // Under RenderMode::OnChanges the UI is only actually drawn when
                            // `need_render` is set. Rendering when it is NOT set would present a
                            // frame that only contains the clear color, wiping the visible UI with
                            // black. So we only render (and swap buffers) when there is something
                            // new to draw; otherwise the previously presented frame persists.
                            //
                            // The 3D scene is drawn unconditionally by `engine.render()`, so a
                            // scene-only frame (scene_need_render set, UI static) renders the
                            // scene but skips the UI re-upload — the previous UI texture stays.
                            let ui = engine.user_interfaces.first();
                            if ui.need_render || state.scene_need_render {
                                state.stats.renders += 1;
                                if ui.need_render {
                                    state.stats.ui_renders += 1;
                                }
                                engine.render().unwrap();
                                state.scene_need_render = false;
                            }
                        }
                        _ => (),
                    }

                    // Any action in the window, other than a redraw request, forces another update
                    // pass which then pushes a redraw request to the event queue. This check
                    // prevents an infinite loop of this kind.
                    if !matches!(event, WindowEvent::RedrawRequested) {
                        state.stats.events += 1;
                        state.update_loop_state.request_update_in_current_frame();
                    }

                    if let Some(os_event) = translate_event(&event) {
                        for ui in engine.user_interfaces.iter_mut() {
                            ui.process_os_event(&os_event);
                        }
                    }
                }
                _ => (),
            }
        })
        .unwrap();
}