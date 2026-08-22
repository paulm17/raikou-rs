//! containers_demo — exercises the Phase 3 container components: Accordion,
//! Tabs, and ScrollArea.

use fyrox::core::pool::Handle;
use fyrox::gui::text::{TextBuilder, TextMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::{Color, Length, Thickness};
use raikou_demo::Options;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let status: Handle<UiNode> = TextBuilder::new(WidgetBuilder::new().with_name("raikou_status"))
        .with_text("no interaction yet")
        .build(&mut cx.ctx())
        .to_base();

    // --- Accordion ---
    let accordion_content: Handle<UiNode> = TextBuilder::new(WidgetBuilder::new())
        .with_text("Accordion body content")
        .build(&mut cx.ctx())
        .to_base();

    let accordion = Accordion::new()
        .item_with_content("General", accordion_content)
        .item("Billing")
        .item("Security")
        .on_toggle(move |ui, index, expanded| {
            ui.send(
                status,
                TextMessage::Text(format!("accordion[{index}] -> {expanded}")),
            );
        })
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .build(&mut cx);
    let accordion_handle: Handle<UiNode> = accordion.into();

    // --- Tabs ---
    let tab1: Handle<UiNode> = TextBuilder::new(WidgetBuilder::new())
        .with_text("Tab one contents")
        .build(&mut cx.ctx())
        .to_base();
    let tab2: Handle<UiNode> = TextBuilder::new(WidgetBuilder::new())
        .with_text("Tab two contents")
        .build(&mut cx.ctx())
        .to_base();

    let tabs = Tabs::new()
        .tab("First", tab1)
        .tab("Second", tab2)
        .on_change(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("active tab -> {index}")));
        })
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .build(&mut cx);
    let tabs_handle: Handle<UiNode> = tabs.into();

    // --- ScrollArea ---
    let mut tall_children: Vec<Handle<UiNode>> = Vec::new();
    for i in 0..20 {
        let label = Label::new(format!("row {i}"))
            .color(Color::new(0.36, 0.42, 0.50, 1.0))
            .build(&mut cx);
        tall_children.push(label.into());
    }
    let tall_panel: Handle<UiNode> = Stack::new()
        .spacing(6.0)
        .children(tall_children)
        .build(&mut cx)
        .into();

    let scroll = ScrollArea::new()
        .content(tall_panel)
        .v_scroll_speed(30.0)
        .width(Length::Fixed(360.0))
        .height(Length::Fixed(220.0))
        .on_scroll(move |ui, v, h| {
            ui.send(
                status,
                TextMessage::Text(format!("scrolled -> v {v:.1}, h {h:.1}")),
            );
        })
        .build(&mut cx);
    let scroll_handle: Handle<UiNode> = scroll.into();

    let heading = Label::new("Containers")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(accordion_handle)
        .child(tabs_handle)
        .child(scroll_handle)
        .child(status)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — containers demo".to_string(),
            width: 900,
            height: 640,
        },
        Box::new(build_demo_panel),
    );
}
