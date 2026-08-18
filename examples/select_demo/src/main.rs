//! select_demo — exercises the Phase 4 Select component (read-only dropdown).

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

    let select = Select::new()
        .items(vec!["Crimson", "Teal", "Gold", "Slate"])
        .selected(1)
        .placeholder("Pick a color...")
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .on_change(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("selected -> index {index}")));
        })
        .build(&mut cx);
    let select_handle: Handle<UiNode> = select.into();

    let heading = Label::new("Select")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let hint = Label::new("Open the dropdown and pick a color.")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let hint_handle: Handle<UiNode> = hint.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(select_handle)
        .child(hint_handle)
        .child(status)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — select demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}