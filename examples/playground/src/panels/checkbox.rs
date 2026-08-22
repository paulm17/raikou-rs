//! checkbox panel — playground demo for the raikou `Checkbox` component.
//!
//! Port of the reference `checkbox_demo`: a preview checkbox whose label and
//! checked/disabled flags are live, while the size, radius, color and
//! indeterminate options regenerate the code panel.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::check_box::CheckBoxMessage;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

/// Color presets selectable in the controls sidebar.
const COLORS: &[(&str, Color)] = &[
    ("Slate", Color::new(0.52, 0.58, 0.66, 1.0)),
    ("Coral", Color::new(0.98, 0.34, 0.32, 1.0)),
    ("Rose", Color::new(0.88, 0.27, 0.53, 1.0)),
    ("Violet", Color::new(0.67, 0.31, 0.87, 1.0)),
    ("Indigo", Color::new(0.42, 0.32, 0.88, 1.0)),
    ("Blue", Color::new(0.22, 0.55, 0.93, 1.0)),
    ("Cyan", Color::new(0.10, 0.69, 0.83, 1.0)),
    ("Emerald", Color::new(0.11, 0.72, 0.45, 1.0)),
    ("Lime", Color::new(0.47, 0.78, 0.20, 1.0)),
    ("Amber", Color::new(0.95, 0.68, 0.11, 1.0)),
    ("Orange", Color::new(0.97, 0.48, 0.10, 1.0)),
];

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    label: String,
    color: usize,
    size: f32,
    radius: f32,
    checked: bool,
    disabled: bool,
    indeterminate: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            label: "I agree to sell my privacy".to_string(),
            color: 5,
            size: 20.0,
            radius: 4.0,
            checked: true,
            disabled: false,
            indeterminate: false,
        }
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let mut code = format!(
        "let checkbox = Checkbox::new()\n    .text({:?})\n    .checked({})\n    .size({:.0})\n    .corner_radius({:.0})\n    .color(Color::new({}, {}, {}, 1.0));\n",
        state.label,
        state.checked,
        state.size,
        state.radius,
        COLORS[state.color].1.red,
        COLORS[state.color].1.green,
        COLORS[state.color].1.blue,
    );
    if state.disabled {
        code.push_str("// ui.send(checkbox, WidgetMessage::Enabled(false));\n");
    }
    if state.indeterminate {
        code.push_str("// indeterminate: not a Checkbox prop; drawn by the preview.\n");
    }
    code.push_str(&format!("// accent: {}\n", COLORS[state.color].0));
    code
}

pub fn checkbox_panel(
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

    let code_state = Rc::clone(&state);
    let code_fn: Rc<dyn Fn() -> String> =
        Rc::new(move || build_code(&code_state.borrow().clone()));

    let code_fn_for_block = Rc::clone(&code_fn);
    let code_handle = PlaygroundCodeBlock::new(move || code_fn_for_block()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Checkbox.rs", code_handle)
        .height(280.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    let defaults = PlaygroundState::default();
    let preview_checkbox = Checkbox::new()
        .text(&defaults.label)
        .checked(true)
        .on_change(|_ui, _checked| {})
        .build(&mut cx);
    let checkbox_handle: Handle<UiNode> = preview_checkbox.into();

    let preview = PlaygroundPreview::new(checkbox_handle)
        .content_max_size(420.0, 160.0)
        .build(&mut cx);

    // --- controls sidebar --------------------------------------------------
    let state_label = Rc::clone(&state);
    let code_fn_label = Rc::clone(&code_fn);
    let label_input = TextInput::new()
        .text(&defaults.label)
        .on_change(move |ui, text| {
            state_label.borrow_mut().label = text.to_string();
            update_code(ui, code_handle, &*code_fn_label);
        })
        .build(&mut cx);

    let color_names: Vec<String> = COLORS.iter().map(|(name, _)| name.to_string()).collect();
    let state_color = Rc::clone(&state);
    let code_fn_color = Rc::clone(&code_fn);
    let color_select = Select::new()
        .items(color_names)
        .selected(defaults.color)
        .on_change(move |ui, index| {
            state_color.borrow_mut().color = index;
            update_code(ui, code_handle, &*code_fn_color);
        })
        .build(&mut cx);

    let state_size = Rc::clone(&state);
    let code_fn_size = Rc::clone(&code_fn);
    let size_slider = control_slider()
        .min(14.0)
        .max(32.0)
        .step(1.0)
        .value(defaults.size)
        .on_change(move |ui, v| {
            state_size.borrow_mut().size = v;
            update_code(ui, code_handle, &*code_fn_size);
        })
        .build(&mut cx);

    let state_radius = Rc::clone(&state);
    let code_fn_radius = Rc::clone(&code_fn);
    let radius_slider = control_slider()
        .min(0.0)
        .max(16.0)
        .step(1.0)
        .value(defaults.radius)
        .on_change(move |ui, v| {
            state_radius.borrow_mut().radius = v;
            update_code(ui, code_handle, &*code_fn_radius);
        })
        .build(&mut cx);

    let state_checked = Rc::clone(&state);
    let code_fn_checked = Rc::clone(&code_fn);
    let checked_switch = Switch::new()
        .text("Checked")
        .toggled(true)
        .on_change(move |ui, on| {
            state_checked.borrow_mut().checked = on;
            ui.send(checkbox_handle, CheckBoxMessage::Check(Some(on)));
            update_code(ui, code_handle, &*code_fn_checked);
        })
        .build(&mut cx);

    let state_disabled = Rc::clone(&state);
    let code_fn_disabled = Rc::clone(&code_fn);
    let disabled_switch = Switch::new()
        .text("Disabled")
        .on_change(move |ui, on| {
            state_disabled.borrow_mut().disabled = on;
            ui.send(checkbox_handle, WidgetMessage::Enabled(!on));
            update_code(ui, code_handle, &*code_fn_disabled);
        })
        .build(&mut cx);

    let state_indeterminate = Rc::clone(&state);
    let code_fn_indeterminate = Rc::clone(&code_fn);
    let indeterminate_switch = Switch::new()
        .text("Indeterminate")
        .on_change(move |ui, on| {
            state_indeterminate.borrow_mut().indeterminate = on;
            update_code(ui, code_handle, &*code_fn_indeterminate);
        })
        .build(&mut cx);

    let controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Checkbox playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(Label::new("Label").font_size(12.0).color(muted).build(&mut cx))
        .child(label_input)
        .child(Label::new("Color").font_size(12.0).color(muted).build(&mut cx))
        .child(color_select)
        .child(Label::new("Size").font_size(12.0).color(muted).build(&mut cx))
        .child(size_slider)
        .child(Label::new("Radius").font_size(12.0).color(muted).build(&mut cx))
        .child(radius_slider)
        .child(checked_switch)
        .child(disabled_switch)
        .child(indeterminate_switch)
        .build(&mut cx);

    // --- shell -------------------------------------------------------------
    let shell = PlaygroundShell::new(preview, controls, code_panel)
        .sidebar_width(280.0)
        .code_height(280.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(820.0));
    shell_handle
}
