//! button_demo — a standalone window exercising the raikou `Button` builder.
//!
//! Shows one button per variant, the three click modes (release/press/hover),
//! a loading state, and a button with custom child content — all reporting to
//! a shared counter label. A trimmed version of the app's tool-loop bootstrap:
//! fyrox engine with a single `UserInterface` in `RenderMode::OnChanges`, a
//! `ControlFlow::Wait` loop and a message poll that dispatches into the raikou
//! registry.

#![allow(deprecated)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fyrox::asset::{io::FsResourceIo, manager::ResourceManager};
use fyrox::core::algebra::{Matrix3, Vector2};
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
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::text::{TextBuilder, TextMessage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{Orientation, RenderMode, Thickness, UiNode, UserInterface};
use fyrox::utils::translate_event;
use fyrox::window::WindowAttributes;
use raikou::prelude::*;
use raikou::{Color, Length};

/// Builds a small demo panel driven entirely through the raikou builder API:
/// one button per variant, all reporting to a shared counter label.
fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let counter: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new()
            .with_name("raikou_counter")
            .with_margin(Thickness::uniform(4.0)),
    )
    .with_text("raikou clicks: 0")
    .build(&mut cx.ui().build_ctx())
    .transmute();

    let count = Rc::new(RefCell::new(0u32));

    let mut buttons: Vec<Handle<UiNode>> = Vec::new();
    for variant in [
        ButtonVariant::Filled,
        ButtonVariant::Outline,
        ButtonVariant::Ghost,
        ButtonVariant::Subtle,
        ButtonVariant::Link,
    ] {
        let count = Rc::clone(&count);
        let button = Button::new()
            .text(variant_label(variant))
            .variant(variant)
            .size(ControlSize::Medium)
            .on_click(move |ui, _event| {
                *count.borrow_mut() += 1;
                let total = *count.borrow();
                ui.send(
                    counter,
                    TextMessage::Text(format!("raikou clicks: {total}")),
                );
            })
            .build(&mut cx);
        buttons.push(button.into());
    }

    let row: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(4.0))
            .with_children(buttons),
    )
    .with_orientation(Orientation::Horizontal)
    .build(&mut cx.ui().build_ctx())
    .transmute();

    // Click modes: Release (default), Press and Hover.
    let mut click_buttons: Vec<Handle<UiNode>> = Vec::new();
    for (label, mode) in [
        ("Release", ClickMode::Release),
        ("Press", ClickMode::Press),
        ("Hover", ClickMode::Hover),
    ] {
        let count = Rc::clone(&count);
        let button = Button::new()
            .text(label)
            .variant(ButtonVariant::Subtle)
            .click_mode(mode)
            .on_click(move |ui, _event| {
                *count.borrow_mut() += 1;
                let total = *count.borrow();
                ui.send(
                    counter,
                    TextMessage::Text(format!("raikou clicks: {total}")),
                );
            })
            .build(&mut cx);
        click_buttons.push(button.into());
    }

    let click_row: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(4.0))
            .with_children(click_buttons),
    )
    .with_orientation(Orientation::Horizontal)
    .build(&mut cx.ui().build_ctx())
    .transmute();

    // Loading and custom-content states.
    let loading_button = Button::new().text("Save").is_loading(true).build(&mut cx);
    let loading_button_handle: Handle<UiNode> = loading_button.into();

    let content_box = BoxWidget::new()
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .color(Color::new(0.13, 0.39, 0.94, 1.0))
        .corner_radius(3.0)
        .build(&mut cx);
    let content_box_handle: Handle<UiNode> = content_box.into();
    let content_button = Button::new()
        .variant(ButtonVariant::Outline)
        .content(content_box_handle)
        .build(&mut cx);
    let content_button_handle: Handle<UiNode> = content_button.into();

    let state_row: Handle<UiNode> = StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_margin(Thickness::uniform(4.0))
            .with_children(vec![loading_button_handle, content_button_handle]),
    )
    .with_orientation(Orientation::Horizontal)
    .build(&mut cx.ui().build_ctx())
    .transmute();

    StackPanelBuilder::new(
        WidgetBuilder::new()
            .with_name("raikou_panel")
            .with_margin(Thickness::uniform(8.0))
            .with_child(counter)
            .with_child(row)
            .with_child(click_row)
            .with_child(state_row),
    )
    .build(&mut cx.ui().build_ctx())
    .transmute()
}

fn variant_label(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "Filled",
        ButtonVariant::Outline => "Outline",
        ButtonVariant::Ghost => "Ghost",
        ButtonVariant::Subtle => "Subtle",
        ButtonVariant::Link => "Link",
    }
}

fn main() {
    let mut window_attributes = WindowAttributes::default();
    window_attributes.inner_size = Some(PhysicalSize::new(900, 600).into());
    window_attributes.resizable = true;
    window_attributes.title = "raikou — button demo".to_string();

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
    let theme = Theme::light();
    let panel = build_demo_panel(engine.user_interfaces.first_mut(), &theme, &mut registry);
    {
        let ui = engine.user_interfaces.first_mut();
        let root = ui.root();
        let mut ctx = ui.build_ctx();
        ctx.link(panel, root);
        ui.need_render = true;
    }

    let event_loop = EventLoop::new().unwrap();

    event_loop
        .run(move |event, active_event_loop| {
            active_event_loop.set_control_flow(ControlFlow::Wait);

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
                                registry.dispatch(ui, &message);
                            } else {
                                break;
                            }
                        }
                    }
                    // engine.update drives UserInterface::update -> update_layout, which
                    // measures/arranges every node and processes pending messages. Without
                    // it the UI has no geometry and renders as a blank screen.
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
                                ui.root(),
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
                            // Under RenderMode::OnChanges the UI is only actually drawn
                            // when `need_render` is set; rendering otherwise would wipe
                            // the visible UI with the clear color.
                            let ui = engine.user_interfaces.first();
                            if ui.need_render {
                                engine.render().unwrap();
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
