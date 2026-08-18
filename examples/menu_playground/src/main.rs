//! menu_playground — playground demo for the raikou `MenuBar`.
//!
//! Port of the reference `menu_demo`: a three-menu `MenuBar` plus a standalone
//! menu opened immediately. The reference standalone `Menu` widget does not
//! exist in the fyrox port, so a `ContextMenu` is used for the standalone part.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"MenuBar::new()
    .menu("File", vec![
        MenuItem::new("New file"),
        MenuItem::new("Open recent").submenu(vec![
            MenuItem::new("report.md"),
            MenuItem::new("notes.txt"),
        ]),
        MenuItem::new("Save all"),
    ])
    .menu("Edit", vec![
        MenuItem::new("Undo"),
        MenuItem::new("Redo"),
        MenuItem::new("Format document"),
    ])"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let menu_bar = MenuBar::new()
        .menu(
            "File",
            vec![
                MenuItem::new("New file"),
                MenuItem::new("Open recent")
                    .submenu(vec![MenuItem::new("report.md"), MenuItem::new("notes.txt")]),
                MenuItem::new("Save all"),
            ],
        )
        .menu(
            "Edit",
            vec![
                MenuItem::new("Undo"),
                MenuItem::new("Redo"),
                MenuItem::new("Format document"),
            ],
        )
        .menu("View", vec![MenuItem::new("Command palette"), MenuItem::new("Problems")])
        .build(&mut cx);

    let standalone = ContextMenu::new()
        .item(MenuItem::new("New file"))
        .item(MenuItem::new("Save all"))
        .item(MenuItem::new("Delete").disabled())
        .build(&mut cx);
    let standalone_handle: Handle<UiNode> = standalone.into();
    show_context_menu(cx.ui(), standalone_handle);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(24.0)
        .child(menu_bar)
        .child(standalone_handle)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(400.0, 180.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Menu playground",
        &[
            "The menu bar exposes the classic drop-down affordance per top-level label.",
            "The standalone menu below starts open, matching the reference demo.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Menu.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — menu playground".to_string(),
            width: 980,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}