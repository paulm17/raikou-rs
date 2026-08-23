//! slider panel — playground demo for the raikou `Slider`.
//!
//! Two live sliders (whole-number and fractional ranges) with readouts
//! driven by `on_change`.

use fyrox::core::pool::Handle;
use fyrox::gui::text::TextMessage;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r#"Slider::new()
    .min(0.0)
    .max(100.0)
    .value(40.0)
    .on_change(|ui, value| { /* ... */ })"#;

pub fn slider_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let basic_readout = Label::new("Value: 40").font_size(13.0).build(&mut cx);
    let basic_readout: Handle<UiNode> = basic_readout.into();

    let percent_readout = Label::new("Value: 50%").font_size(13.0).build(&mut cx);
    let percent_readout: Handle<UiNode> = percent_readout.into();

    let basic = Slider::new()
        .min(0.0)
        .max(100.0)
        .value(40.0)
        .on_change(move |ui, v| {
            ui.send(basic_readout, TextMessage::Text(format!("Value: {v:.0}")));
        })
        .build(&mut cx);
    cx.ui().send(basic.handle, WidgetMessage::Width(360.0));

    let fractional = Slider::new()
        .min(0.0)
        .max(1.0)
        .step(0.05)
        .value(0.5)
        .on_change(move |ui, v| {
            ui.send(
                percent_readout,
                TextMessage::Text(format!("Value: {:.0}%", v * 100.0)),
            );
        })
        .build(&mut cx);
    cx.ui().send(fractional.handle, WidgetMessage::Width(360.0));

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(18.0)
        .child(basic.handle)
        .child(basic_readout)
        .child(fractional.handle)
        .child(percent_readout)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(420.0, 160.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Slider playground",
        &[
            "Drag the thumb or click the track; the readouts track on_change live.",
            "Thin 4px Fluent-style track with a round thumb; the stock arrows are hidden.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Slider.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(960.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(720.0));
    shell_handle
}
