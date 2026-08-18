//! context_menu_playground — playground demo for the raikou `ContextMenu`.
//!
//! Port of the reference `context_menu_demo`: a labelled target surface with a
//! context menu opened immediately so the popup body is visible on first paint.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"ContextMenu::new()
    .item(MenuItem::new("Open"))
    .item(MenuItem::new("Rename"))
    .item(MenuItem::new("Delete").disabled());"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let menu = ContextMenu::new()
        .item(MenuItem::new("Open"))
        .item(MenuItem::new("Rename"))
        .item(MenuItem::new("Delete").disabled())
        .build(&mut cx);
    let menu_handle: Handle<UiNode> = menu.into();
    show_context_menu(cx.ui(), menu_handle);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(18.0)
        .child(
            Label::new("Context target area")
                .font_size(14.0)
                .color(Color::new(0.35, 0.40, 0.47, 1.0))
                .build(&mut cx),
        )
        .child(
            BoxWidget::new()
                .width(260.0)
                .height(84.0)
                .color(Color::new(0.89, 0.94, 0.99, 1.0))
                .corner_radius(16.0)
                .border_width(1.0)
                .border_color(Color::new(0.68, 0.78, 0.92, 1.0))
                .build(&mut cx),
        )
        .child(menu_handle)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(280.0, 240.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "ContextMenu playground",
        &[
            "The menu starts open; the target surface above helps recognize it as a context action list.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("ContextMenu.rs", code).build(&mut cx);

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
            title: "raikou — context menu playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}