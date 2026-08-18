//! tabs_playground — playground demo for the raikou `Tabs` component.
//!
//! Port of the reference `tabs_demo`: three tabs with different content. The
//! reference `content_height` knob is not part of the fyrox port.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"Tabs::new()
    .tab("Overview", Label::new("Healthy"))
    .tab("Deployments", Label::new("3 queued"))
    .tab("Alerts", Label::new("2 unresolved"))"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let overview: Handle<UiNode> = Stack::new()
        .spacing(10.0)
        .child(
            Label::new("Healthy")
                .font_size(18.0)
                .build(&mut cx),
        )
        .child(
            Label::new("API latency and background jobs are within normal range.")
                .font_size(13.0)
                .color(Color::new(0.37, 0.42, 0.48, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let deployments: Handle<UiNode> = Stack::new()
        .spacing(10.0)
        .child(
            Label::new("3 queued")
                .font_size(18.0)
                .build(&mut cx),
        )
        .child(
            Button::new()
                .text("Open pipeline")
                .width(Length::Fixed(150.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let alerts: Handle<UiNode> = Stack::new()
        .spacing(10.0)
        .child(
            Label::new("2 unresolved")
                .font_size(18.0)
                .build(&mut cx),
        )
        .child(
            Button::new()
                .text("View incidents")
                .width(Length::Fixed(150.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let tabs = Tabs::new()
        .tab("Overview", overview)
        .tab("Deployments", deployments)
        .tab("Alerts", alerts)
        .build(&mut cx);

    let preview = PlaygroundPreview::new(tabs)
        .content_max_size(400.0, 220.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Tabs playground",
        &[
            "Each tab shows meaningful content for its pane.",
            "Click each tab to validate active-state painting and content switching.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Tabs.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — tabs playground".to_string(),
            width: 980,
            height: 760,
        },
        Box::new(build_demo_panel),
    );
}