//! combobox_playground — playground demo for the raikou `Combobox` component.
//!
//! Port of the reference `combobox_demo`: two comboboxes on a stage, one
//! showing the placeholder and one pre-selected, with the equivalent builder
//! source shown in the code panel.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"Combobox::new()
    .items(vec!["Alice", "Bruno", "Chloe", "Daria"])
    .placeholder("Assign reviewer")
    .selected(2)"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(16.0)
        .child(
            Combobox::new()
                .items(vec!["Alice", "Bruno", "Chloe", "Daria"])
                .placeholder("Assign reviewer")
                .build(&mut cx),
        )
        .child(
            Combobox::new()
                .items(vec!["Alice", "Bruno", "Chloe", "Daria"])
                .placeholder("Assign reviewer")
                .selected(2)
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(240.0, 140.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Combobox playground",
        &[
            "The first combobox starts with the placeholder text, the second is pre-selected.",
            "Type to exercise the filtering path.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Combobox.rs", code).build(&mut cx);

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
            title: "raikou — combobox playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}