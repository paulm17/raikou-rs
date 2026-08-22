//! text_input panel — playground demo for the raikou `TextInput`.
//!
//! Port of the reference `text_input_demo`: an empty field showing the
//! placeholder and a pre-filled field. The reference `width` knob is not part
//! of the fyrox port.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r#"TextInput::new()
    .text("raikou-widgets")
    .placeholder("Search packages")"#;

pub fn text_input_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(16.0)
        .child(
            TextInput::new()
                .placeholder("Search packages")
                .build(&mut cx),
        )
        .child(
            TextInput::new()
                .text("raikou-widgets")
                .placeholder("Search packages")
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(400.0, 140.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "TextInput playground",
        &[
            "The first field shows the placeholder, the second a concrete value.",
            "Click and type to validate focus and editing.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("TextInput.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(960.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}
