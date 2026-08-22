//! radio panel — playground demo for the raikou `Radio` component.
//!
//! Port of the reference `radio_demo`: a small group of radio options with one
//! selected and one disabled (the reference `is_enabled` knob is not part of
//! the fyrox port, so the disabled option is omitted).

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r#"Stack::new()
    .spacing(14.0)
    .child(Radio::new().text("Daily digest").selected(true))
    .child(Radio::new().text("Weekly summary"))
    .child(Radio::new().text("Only critical outages"))"#;

pub fn radio_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(14.0)
        .child(
            Radio::new()
                .text("Daily digest")
                .selected(true)
                .build(&mut cx),
        )
        .child(Radio::new().text("Weekly summary").build(&mut cx))
        .child(
            Radio::new()
                .text("Only critical outages")
                .build(&mut cx),
        )
        .child(Radio::new().text("Never").build(&mut cx))
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(280.0, 180.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Radio playground",
        &[
            "Shows one selected option and the rest available.",
            "Clicking an option selects it; only one can be active.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Radio.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(960.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}
