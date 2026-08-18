//! switch_playground — playground demo for the raikou `Switch` component.
//!
//! Port of the reference `switch_demo`: three switches covering on/off/disabled
//! states. The reference `is_enabled` knob is emulated with a runtime message.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"Stack::new()
    .spacing(16.0)
    .child(Switch::new().text("Notifications").toggled(true))
    .child(Switch::new().text("Offline mode").toggled(false))
    .child(Switch::new().text("Scheduled jobs").toggled(true))"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let jobs = Switch::new()
        .text("Scheduled jobs")
        .toggled(true)
        .build(&mut cx);
    let jobs_handle: Handle<UiNode> = jobs.into();
    cx.ui().send(jobs_handle, WidgetMessage::Enabled(false));

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(16.0)
        .child(
            Switch::new()
                .text("Notifications")
                .toggled(true)
                .build(&mut cx),
        )
        .child(
            Switch::new()
                .text("Offline mode")
                .toggled(false)
                .build(&mut cx),
        )
        .child(jobs_handle)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(320.0, 180.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Switch playground",
        &[
            "On, off and disabled states are all visible at once.",
            "A narrow vertical stack keeps the group readable.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Switch.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(960.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — switch playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}