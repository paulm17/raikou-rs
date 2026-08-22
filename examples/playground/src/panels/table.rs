//! table panel — playground demo for the raikou `Table` component.
//!
//! Port of the reference `table_demo`: a build-status table with explicit
//! column widths and four rows.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r##"Table::new()
    .column(TableColumn::new("Build", 140.0))
    .column(TableColumn::new("Status", 120.0))
    .column(TableColumn::new("Owner", 180.0))
    .row(vec!["#1428", "Running", "CI"])
    .row(vec!["#1427", "Passed", "Release bot"])"##;

pub fn table_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let table = Table::new()
        .column(TableColumn::new("Build", 140.0))
        .column(TableColumn::new("Status", 120.0))
        .column(TableColumn::new("Owner", 180.0))
        .row(vec!["#1428", "Running", "CI"])
        .row(vec!["#1427", "Passed", "Release bot"])
        .row(vec!["#1426", "Blocked", "Platform"])
        .row(vec!["#1425", "Passed", "Infra"])
        .build(&mut cx);

    let preview = PlaygroundPreview::new(table)
        .content_max_size(440.0, 220.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Table playground",
        &[
            "Explicit column widths keep the grid readable.",
            "Hovering rows validates the baseline interactive treatment.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Table.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
