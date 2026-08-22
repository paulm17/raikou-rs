//! label panel — playground demo for the raikou `Label` component.
//!
//! Port of the reference `label_demo`: three labels of descending hierarchy on
//! a stage, with the builder source shown in the code panel.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

const CODE: &str = r#"Stack::new()
    .spacing(12.0)
    .child(
        Label::new("Launch metrics are trending up")
            .font_size(28.0),
    )
    .child(
        Label::new("Secondary copy can stay softer without losing readability.")
            .font_size(16.0)
            .color(Color::new(0.36, 0.42, 0.50, 1.0)),
    )"#;

pub fn label_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Launch metrics are trending up")
                .font_size(28.0)
                .color(Color::new(0.12, 0.15, 0.20, 1.0))
                .build(&mut cx),
        )
        .child(
            Label::new("Secondary copy can stay softer without losing readability.")
                .font_size(16.0)
                .color(Color::new(0.36, 0.42, 0.50, 1.0))
                .build(&mut cx),
        )
        .child(
            Label::new("Status: ready for launch")
                .font_size(13.0)
                .color(Color::new(0.17, 0.58, 0.40, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(520.0, 160.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Label playground",
        &[
            "Stacks a headline, body and status copy to show the hierarchy.",
            "Long lines wrap naturally to the available width.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Label.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(960.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}
