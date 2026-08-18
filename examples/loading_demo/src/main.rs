//! loading_demo — exercises the Phase 4 LoadingIndicator: all nine animation
//! modes, plus buttons in a loading state and with custom content.

use fyrox::core::pool::Handle;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::{Color, Thickness};
use raikou_demo::Options;

fn mode_row(
    cx: &mut BuildCx,
    label: &str,
    mode: LoadingIndicatorMode,
) -> Handle<UiNode> {
    let name = Label::new(label)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(cx);
    let spinner = LoadingIndicator::new()
        .mode(mode)
        .size(28.0)
        .build(cx);
    Group::new()
        .spacing(12.0)
        .child(name)
        .child(spinner)
        .build(cx)
        .into()
}

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let heading = Label::new("LoadingIndicator")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let sub = Label::new("All nine animation modes")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let sub_handle: Handle<UiNode> = sub.into();

    let mut mode_rows = Vec::new();
    for (name, mode) in [
        ("Arc", LoadingIndicatorMode::Arc),
        ("Arcs", LoadingIndicatorMode::Arcs),
        ("ArcsRing", LoadingIndicatorMode::ArcsRing),
        ("DoubleBounce", LoadingIndicatorMode::DoubleBounce),
        ("FlipPlane", LoadingIndicatorMode::FlipPlane),
        ("Pulse", LoadingIndicatorMode::Pulse),
        ("Ring", LoadingIndicatorMode::Ring),
        ("ThreeDots", LoadingIndicatorMode::ThreeDots),
        ("Wave", LoadingIndicatorMode::Wave),
    ] {
        mode_rows.push(mode_row(&mut cx, name, mode));
    }

    let modes: Handle<UiNode> = Stack::new()
        .spacing(10.0)
        .children(mode_rows)
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .build(&mut cx)
        .into();

    // A button in a loading state (label replaced by a Pulse spinner).
    let loading_button = Button::new()
        .text("Save")
        .is_loading(true)
        .build(&mut cx);
    let loading_button_handle: Handle<UiNode> = loading_button.into();

    // A button with a custom child widget (a small Ring spinner) as content.
    let inline_spinner = LoadingIndicator::new()
        .mode(LoadingIndicatorMode::Ring)
        .size(20.0)
        .build(&mut cx);
    let inline_spinner_handle: Handle<UiNode> = inline_spinner.into();
    let content_button = Button::new()
        .variant(ButtonVariant::Outline)
        .content(inline_spinner_handle)
        .margin(Thickness::new(12.0, 0.0, 0.0, 0.0))
        .build(&mut cx);
    let content_button_handle: Handle<UiNode> = content_button.into();

    let buttons: Handle<UiNode> = Group::new()
        .spacing(0.0)
        .child(loading_button_handle)
        .child(content_button_handle)
        .build(&mut cx)
        .into();

    let section = Label::new("Buttons: loading state and custom content")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let section_handle: Handle<UiNode> = section.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(sub_handle)
        .child(modes)
        .child(section_handle)
        .child(buttons)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — loading indicator demo".to_string(),
            width: 900,
            height: 700,
        },
        Box::new(build_demo_panel),
    );
}