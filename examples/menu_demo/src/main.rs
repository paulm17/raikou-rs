//! menu_demo — exercises the Phase 4 MenuBar component: top-level menus,
//! sub-menus, disabled items and index-based item click reporting.

use fyrox::core::pool::Handle;
use fyrox::gui::text::{TextBuilder, TextMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::{Color, Thickness};
use raikou_demo::Options;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let status: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new().with_name("raikou_status"),
    )
    .with_text("no interaction yet")
    .build(&mut cx.ctx())
    .to_base();

    let file_menu = vec![
        MenuItem::new("New").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("File > New (index {index})")));
        }),
        MenuItem::new("Open").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("File > Open (index {index})")));
        }),
        MenuItem::new("Recent")
            .submenu(vec![
                MenuItem::new("report.md").on_click(move |ui, index| {
                    ui.send(status, TextMessage::Text(format!("Recent > report.md (index {index})")));
                }),
                MenuItem::new("notes.txt").on_click(move |ui, index| {
                    ui.send(status, TextMessage::Text(format!("Recent > notes.txt (index {index})")));
                }),
            ])
            .on_click(move |ui, index| {
                ui.send(status, TextMessage::Text(format!("File > Recent (index {index})")));
            }),
        MenuItem::new("Save").disabled(),
        MenuItem::new("Quit").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("File > Quit (index {index})")));
        }),
    ];

    let edit_menu = vec![
        MenuItem::new("Undo").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Edit > Undo (index {index})")));
        }),
        MenuItem::new("Redo").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Edit > Redo (index {index})")));
        }),
        MenuItem::new("Find").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Edit > Find (index {index})")));
        }),
    ];

    let menu_bar = MenuBar::new()
        .menu("File", file_menu)
        .menu("Edit", edit_menu)
        .on_item_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("item clicked -> index {index}")));
        })
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .build(&mut cx);
    let menu_bar_handle: Handle<UiNode> = menu_bar.into();

    let heading = Label::new("MenuBar")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let hint = Label::new("Open File / Edit to explore the menus.")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let hint_handle: Handle<UiNode> = hint.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(menu_bar_handle)
        .child(hint_handle)
        .child(status)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — menu demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}