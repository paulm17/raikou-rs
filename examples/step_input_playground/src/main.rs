//! step_input_playground — playground demo for the raikou `StepInput`.
//!
//! Port of the reference `step_input_demo`: an integer stepper and a decimal
//! stepper. The reference `width` knob is not part of the fyrox port.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"StepInput::new()
    .value(12.0)
    .min(0.0)
    .max(24.0)
    .step(1.0)"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(18.0)
        .child(
            StepInput::new()
                .value(12.0)
                .min(0.0)
                .max(24.0)
                .step(1.0)
                .build(&mut cx),
        )
        .child(
            StepInput::new()
                .value(2.5)
                .min(0.0)
                .max(5.0)
                .step(0.5)
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(220.0, 120.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "StepInput playground",
        &[
            "The top stepper uses a whole-number step, the bottom a fractional one.",
            "Click the arrows or type directly into the field.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("StepInput.rs", code).build(&mut cx);

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
            title: "raikou — step input playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}