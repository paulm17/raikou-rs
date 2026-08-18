//! text_area_playground — playground demo for the raikou `TextArea`.
//!
//! Port of the reference `text_area_demo`: a multi-line text area. The
//! reference `placeholder` knob is not part of the fyrox port.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"TextArea::new()
    .text("Ship examples for every public widget.\nKeep the preview readable.")
    .rows(6)"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let text_area = TextArea::new()
        .text("Ship examples for every public widget.\nKeep the preview readable.")
        .rows(6)
        .build(&mut cx);

    let preview = PlaygroundPreview::new(text_area)
        .content_max_size(420.0, 220.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "TextArea playground",
        &[
            "Multi-line content shows row sizing and line wrapping immediately.",
            "The text area fills the available width and grows with its rows.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("TextArea.rs", code).build(&mut cx);

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
            title: "raikou — text area playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}