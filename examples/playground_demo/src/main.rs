//! playground_demo — exercises the raikou-playground building blocks.
//!
//! Port of the reference slider playground: a `PlaygroundShell` splits the
//! window into a live preview stage (left), a scrollable controls sidebar
//! (right) and a code panel (bottom) that reflects the current control state.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::UiNode;
use fyrox::gui::UserInterface;
use raikou::prelude::*;
use raikou::{to_fyrox_color, Color};
use raikou_demo::Options;
use raikou_playground::*;

/// Color presets selectable in the controls sidebar.
const COLORS: &[(&str, Color)] = &[
    ("Crimson", Color::new(0.80, 0.10, 0.20, 1.0)),
    ("Orange", Color::new(0.93, 0.51, 0.13, 1.0)),
    ("Amber", Color::new(0.96, 0.77, 0.19, 1.0)),
    ("Teal", Color::new(0.10, 0.66, 0.55, 1.0)),
    ("Blue", Color::new(0.13, 0.39, 0.94, 1.0)),
    ("Indigo", Color::new(0.36, 0.29, 0.85, 1.0)),
    ("Violet", Color::new(0.64, 0.32, 0.82, 1.0)),
    ("Slate", Color::new(0.30, 0.36, 0.45, 1.0)),
    ("Graphite", Color::new(0.18, 0.20, 0.23, 1.0)),
    ("Coral", Color::new(0.97, 0.46, 0.42, 1.0)),
    ("Mint", Color::new(0.28, 0.82, 0.60, 1.0)),
];

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    value: f32,
    color: usize,
    show_value: bool,
    disabled: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            value: 50.0,
            color: 4,
            show_value: true,
            disabled: false,
        }
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let color = COLORS[state.color].0;
    format!(
        "let mut slider = Slider::new()\n    .min(0.0)\n    .max(100.0)\n    .value({value:.0})\n    .on_change(|ui, value| {{\n        state.value = value;\n    }});\n\n// accent: {color}\n// show_value: {show_value}\n// disabled: {disabled}",
        value = state.value,
        show_value = state.show_value,
        disabled = state.disabled
    )
}

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let primary = theme
        .color("text.primary")
        .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));
    let muted = theme
        .color("text.muted")
        .unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0));

    let state = Rc::new(RefCell::new(PlaygroundState::default()));

    // The code generator is created first so every handler can refresh the
    // code panel through it.
    let code_state = Rc::clone(&state);
    let code_fn: Rc<dyn Fn() -> String> =
        Rc::new(move || build_code(&code_state.borrow().clone()));

    // --- code panel -------------------------------------------------------
    let code_fn_for_block = Rc::clone(&code_fn);
    let code_handle = PlaygroundCodeBlock::new(move || code_fn_for_block()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Slider.rs", code_handle)
        .height(280.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    let value_label: Handle<UiNode> = Label::new("50%")
        .font_size(24.0)
        .color(primary)
        .build(&mut cx)
        .into();

    let bubble: Handle<UiNode> = BoxWidget::new()
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(64.0))
        .color(COLORS[PlaygroundState::default().color].1)
        .corner_radius(32.0)
        .build(&mut cx)
        .into();

    let state_slider = Rc::clone(&state);
    let code_fn_slider = Rc::clone(&code_fn);
    let slider = control_slider()
        .value(50.0)
        .on_change(move |ui, v| {
            state_slider.borrow_mut().value = v;
            set_label_text(ui, value_label, format!("{v:.0}%"));
            update_code(ui, code_handle, &*code_fn_slider);
        })
        .build(&mut cx);
    let slider_handle: Handle<UiNode> = slider.into();

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(16.0)
        .child(bubble)
        .child(value_label)
        .child(slider_handle)
        .build(&mut cx)
        .into();

    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(560.0, 300.0)
        .build(&mut cx);

    // --- controls sidebar --------------------------------------------------
    let color_names: Vec<String> = COLORS.iter().map(|(name, _)| name.to_string()).collect();

    let state_select = Rc::clone(&state);
    let code_fn_select = Rc::clone(&code_fn);
    let select = Select::new()
        .items(color_names)
        .selected(4)
        .placeholder("Pick a color")
        .on_change(move |ui, index| {
            state_select.borrow_mut().color = index;
            let color = COLORS[index].1;
            ui.send(
                bubble,
                WidgetMessage::Background(Brush::Solid(to_fyrox_color(color)).into()),
            );
            update_code(ui, code_handle, &*code_fn_select);
        })
        .build(&mut cx);

    let state_show = Rc::clone(&state);
    let show_switch = Switch::new()
        .text("Show value")
        .toggled(true)
        .on_change(move |ui, on| {
            state_show.borrow_mut().show_value = on;
            ui.send(value_label, WidgetMessage::Visibility(on));
            ui.send(bubble, WidgetMessage::Visibility(on));
        })
        .build(&mut cx);

    let state_disabled = Rc::clone(&state);
    let disabled_switch = Switch::new()
        .text("Disabled")
        .on_change(move |ui, on| {
            state_disabled.borrow_mut().disabled = on;
            ui.send(slider_handle, WidgetMessage::Enabled(!on));
        })
        .build(&mut cx);

    let controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Slider playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(
            Label::new("Value")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(slider_handle)
        .child(
            Label::new("Accent color")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(select)
        .child(show_switch)
        .child(disabled_switch)
        .build(&mut cx);

    // --- shell -------------------------------------------------------------
    // The root canvas measures its children with an unbounded available size,
    // so the shell must be sized explicitly to the window or it reports an
    // infinite desired size and nothing renders.
    let shell = PlaygroundShell::new(preview, controls, code_panel)
        .sidebar_width(280.0)
        .code_height(280.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui()
        .send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui()
        .send(shell_handle, WidgetMessage::Height(820.0));
    shell_handle
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — playground demo".to_string(),
            width: 980,
            height: 820,
        },
        Box::new(build_demo_panel),
    );
}