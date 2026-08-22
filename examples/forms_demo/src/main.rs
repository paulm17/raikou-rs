//! forms_demo — a standalone window exercising the raikou Phase 2 input
//! components: Checkbox, Switch, RadioGroup, Slider, TextInput, TextArea and
//! StepInput.

#![allow(deprecated)]

use fyrox::core::pool::Handle;
use fyrox::gui::text::TextMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;

/// Builds a demo panel of every Phase 2 input component, with a status label
/// that reflects the latest interaction.
fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let status: Handle<UiNode> = Label::new("no interaction yet")
        .color(
            theme
                .color("text.muted")
                .unwrap_or(raikou::Color::new(0.4, 0.4, 0.4, 1.0)),
        )
        .build(&mut cx)
        .into();

    let checkbox = Checkbox::new()
        .text("Subscribe to updates")
        .on_change(move |ui, checked| {
            ui.send(status, TextMessage::Text(format!("checkbox: {checked}")));
        })
        .build(&mut cx);

    let switch = Switch::new()
        .text("Dark mode")
        .on_change(move |ui, toggled| {
            ui.send(status, TextMessage::Text(format!("switch: {toggled}")));
        })
        .build(&mut cx);

    let radio_group = RadioGroup::new()
        .options(["Small", "Medium", "Large"])
        .selected(1)
        .on_change(move |ui, index| {
            ui.send(status, TextMessage::Text(format!("radio: {index}")));
        })
        .build(&mut cx);

    let slider = Slider::new()
        .min(0.0)
        .max(100.0)
        .value(42.0)
        .step(1.0)
        .on_change(move |ui, value| {
            ui.send(status, TextMessage::Text(format!("slider: {value:.0}")));
        })
        .build(&mut cx);

    let text_input = TextInput::new()
        .placeholder("Type your name")
        .on_change(move |ui, text| {
            ui.send(status, TextMessage::Text(format!("input: {text}")));
        })
        .build(&mut cx);

    let text_area = TextArea::new()
        .text("line one\nline two")
        .rows(4)
        .on_change(move |ui, text| {
            ui.send(status, TextMessage::Text(format!("area: {text}")));
        })
        .build(&mut cx);

    let step_input = StepInput::new()
        .min(0.0)
        .max(10.0)
        .value(3.0)
        .step(1.0)
        .on_change(move |ui, value| {
            ui.send(status, TextMessage::Text(format!("step: {value}")));
        })
        .build(&mut cx);

    let heading = Label::new("Input controls")
        .font_size(18.0)
        .color(
            theme
                .color("text.primary")
                .unwrap_or(raikou::Color::new(1.0, 1.0, 1.0, 1.0)),
        )
        .build(&mut cx);

    Stack::new()
        .spacing(12.0)
        .child(heading)
        .child(status)
        .child(checkbox)
        .child(switch)
        .child(radio_group)
        .child(slider)
        .child(text_input)
        .child(text_area)
        .child(step_input)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        raikou_demo::Options {
            title: "raikou — forms demo".to_string(),
            width: 900,
            height: 720,
        },
        Box::new(build_demo_panel),
    );
}
