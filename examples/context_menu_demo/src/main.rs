//! context_menu_demo — exercises the Phase 4 ContextMenu component: a
//! floating menu opened with `show_context_menu` and closed on selection.

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

    let context_menu = ContextMenu::new()
        .item(MenuItem::new("Cut").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Cut (index {index})")));
        }))
        .item(MenuItem::new("Copy").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Copy (index {index})")));
        }))
        .item(MenuItem::new("Paste").on_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("Paste (index {index})")));
        }))
        .item(MenuItem::new("Delete").disabled())
        .on_item_click(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("item clicked -> index {index}")));
        })
        .build(&mut cx);
    let context_menu_handle: Handle<UiNode> = context_menu.into();

    let open_button = Button::new()
        .text("Show context menu")
        .variant(ButtonVariant::Outline)
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .on_click(move |ui, _event| {
            show_context_menu(ui, context_menu_handle);
        })
        .build(&mut cx);
    let open_button_handle: Handle<UiNode> = open_button.into();

    let heading = Label::new("ContextMenu")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let hint = Label::new("Click the button to open the menu next to the pointer.")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let hint_handle: Handle<UiNode> = hint.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(open_button_handle)
        .child(hint_handle)
        .child(status)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — context menu demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}