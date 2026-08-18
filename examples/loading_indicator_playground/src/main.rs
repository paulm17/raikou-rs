//! loading_indicator_playground — playground demo for `LoadingIndicator`.
//!
//! Port of the reference `loading_indicator_demo`: two rows of indicators
//! covering eight of the nine modes (Arc, Ring, ThreeDots, Wave, Pulse,
//! ArcsRing, DoubleBounce, FlipPlane), all animating from startup.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_demo::Options;
use raikou_playground::*;

const CODE: &str = r#"Group::new()
    .spacing(24.0)
    .child(
        LoadingIndicator::new()
            .mode(LoadingIndicatorMode::Arc)
            .size(30.0)
            .color(Color::new(0.19, 0.55, 0.95, 1.0)),
    )
    .child(
        LoadingIndicator::new()
            .mode(LoadingIndicatorMode::Ring)
            .size(30.0)
            .color(Color::new(0.12, 0.71, 0.48, 1.0)),
    )"#;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let row1 = Group::new()
        .spacing(24.0)
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::Arc)
                .size(30.0)
                .color(Color::new(0.19, 0.55, 0.95, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::Ring)
                .size(30.0)
                .color(Color::new(0.12, 0.71, 0.48, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::ThreeDots)
                .size(30.0)
                .color(Color::new(0.91, 0.55, 0.17, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::Wave)
                .size(30.0)
                .color(Color::new(0.74, 0.29, 0.89, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx);

    let row2 = Group::new()
        .spacing(24.0)
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::Pulse)
                .size(30.0)
                .color(Color::new(0.93, 0.35, 0.32, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::ArcsRing)
                .size(30.0)
                .color(Color::new(0.22, 0.61, 0.77, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::DoubleBounce)
                .size(30.0)
                .color(Color::new(0.24, 0.68, 0.43, 1.0))
                .build(&mut cx),
        )
        .child(
            LoadingIndicator::new()
                .mode(LoadingIndicatorMode::FlipPlane)
                .size(30.0)
                .color(Color::new(0.35, 0.43, 0.95, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx);

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(26.0)
        .child(row1)
        .child(row2)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(420.0, 180.0)
        .stage_color(Color::new(0.95, 0.97, 1.0, 1.0))
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "LoadingIndicator playground",
        &[
            "Every indicator stays active so the preview animates immediately.",
            "Multiple modes are shown together since visibility is the main concern.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("LoadingIndicator.rs", code).build(&mut cx);

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
            title: "raikou — loading indicator playground".to_string(),
            width: 980,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}