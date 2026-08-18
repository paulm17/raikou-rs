//! progress_bar_playground — playground demo for the raikou `ProgressBar`.
//!
//! Port of the reference `progress_bar_demo`: three bars at different values
//! and fills. The reference `corner_radius` knob is not part of the fyrox port.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"ProgressBar::new()
    .value(0.68)
    .width(Length::Fixed(360.0))
    .height(16.0)
    .fill_color(Color::new(0.18, 0.60, 0.95, 1.0))"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(18.0)
        .child(
            ProgressBar::new()
                .value(0.22)
                .width(Length::Fixed(360.0))
                .height(14.0)
                .fill_color(Color::new(0.95, 0.48, 0.18, 1.0))
                .build(&mut cx),
        )
        .child(
            ProgressBar::new()
                .value(0.68)
                .width(Length::Fixed(360.0))
                .height(16.0)
                .fill_color(Color::new(0.18, 0.60, 0.95, 1.0))
                .build(&mut cx),
        )
        .child(
            ProgressBar::new()
                .value(1.0)
                .width(Length::Fixed(360.0))
                .height(14.0)
                .fill_color(Color::new(0.17, 0.66, 0.44, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(400.0, 140.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "ProgressBar playground",
        &[
            "Three stages at once: in-flight, near-complete and complete.",
            "The fill color and bar thickness are configurable per bar.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("ProgressBar.rs", code).build(&mut cx);

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
            title: "raikou — progress bar playground".to_string(),
            width: 960,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}