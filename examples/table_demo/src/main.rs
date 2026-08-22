//! table_demo — exercises the Phase 4 Table component: columns, header row
//! and fixed-height data rows (static, no selection).

use fyrox::core::pool::Handle;
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

    let table = Table::new()
        .column(TableColumn::new("Name", 160.0))
        .column(TableColumn::new("Role", 140.0))
        .column(TableColumn::new("Status", 100.0))
        .row(vec!["Ada Lovelace", "Analyst", "active"])
        .row(vec!["Grace Hopper", "Compiler", "active"])
        .row(vec!["Alan Turing", "Theorist", "away"])
        .row(vec!["Linus Torvalds", "Maintainer", "active"])
        .row_height(32.0)
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .build(&mut cx);
    let table_handle: Handle<UiNode> = table.into();

    let heading = Label::new("Table")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let hint = Label::new("A static grid — headers in the first row, data below.")
        .color(
            theme
                .color("text.muted")
                .unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)),
        )
        .build(&mut cx);
    let hint_handle: Handle<UiNode> = hint.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(table_handle)
        .child(hint_handle)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — table demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}
