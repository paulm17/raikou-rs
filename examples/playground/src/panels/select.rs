//! select panel — playground demo for the raikou `Select` component.
//!
//! Port of the reference `select_demo`: a preview select with framework items.
//! The selected item, placeholder, color and disabled flag drive the live
//! preview where possible; all regenerate the code panel.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

const FRAMEWORKS: [&str; 4] = ["React", "Angular", "Vue", "Svelte"];

/// Color presets selectable in the controls sidebar.
const COLORS: &[(&str, Color)] = &[
    ("Blue", Color::new(0.22, 0.55, 0.93, 1.0)),
    ("Emerald", Color::new(0.10, 0.67, 0.43, 1.0)),
    ("Amber", Color::new(0.91, 0.62, 0.14, 1.0)),
    ("Rose", Color::new(0.87, 0.30, 0.49, 1.0)),
    ("Slate", Color::new(0.43, 0.49, 0.58, 1.0)),
];

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    color: usize,
    selection: usize,
    placeholder: String,
    disabled: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            color: 0,
            selection: 0,
            placeholder: "Select placeholder".to_string(),
            disabled: false,
        }
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let mut code = format!(
        "let select = Select::new()\n    .items(vec![\"React\", \"Angular\", \"Vue\", \"Svelte\"])\n    .placeholder({:?})",
        state.placeholder,
    );
    if state.selection != 0 {
        code.push_str(&format!("\n    .selected({})", state.selection - 1));
    }
    code.push_str(";\n");
    if state.disabled {
        code.push_str("// ui.send(select, WidgetMessage::Enabled(false));\n");
    }
    code.push_str(&format!("// accent: {}\n", COLORS[state.color].0));
    code
}

pub fn select_panel(
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
    let code_panel = PlaygroundCodePanel::new("Select.rs", code_handle)
        .height(300.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    let defaults = PlaygroundState::default();
    let preview_select = Select::new()
        .items(FRAMEWORKS.to_vec())
        .placeholder(&defaults.placeholder)
        .on_change(|_ui, _index| {})
        .build(&mut cx);
    let select_handle: Handle<UiNode> = preview_select.into();

    let preview = PlaygroundPreview::new(select_handle)
        .content_max_size(320.0, 160.0)
        .build(&mut cx);

    // --- controls sidebar --------------------------------------------------
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

    let state_selection = Rc::clone(&state);
    let code_fn_selection = Rc::clone(&code_fn);
    let selection_select = Select::new()
        .items(vec!["None", "React", "Angular", "Vue", "Svelte"])
        .selected(defaults.selection)
        .on_change(move |ui, index| {
            state_selection.borrow_mut().selection = index;
            if index > 0 {
                ui.send(
                    select_handle,
                    fyrox::gui::dropdown_list::DropdownListMessage::Selection(Some(index - 1)),
                );
            }
            update_code(ui, code_handle, &*code_fn_selection);
        })
        .build(&mut cx);

    let state_placeholder = Rc::clone(&state);
    let code_fn_placeholder = Rc::clone(&code_fn);
    let placeholder_input = TextInput::new()
        .text(&defaults.placeholder)
        .on_change(move |ui, text| {
            state_placeholder.borrow_mut().placeholder = text.to_string();
            update_code(ui, code_handle, &*code_fn_placeholder);
        })
        .build(&mut cx);

    let state_disabled = Rc::clone(&state);
    let code_fn_disabled = Rc::clone(&code_fn);
    let disabled_switch = Switch::new()
        .text("Disabled")
        .on_change(move |ui, on| {
            state_disabled.borrow_mut().disabled = on;
            ui.send(select_handle, WidgetMessage::Enabled(!on));
            update_code(ui, code_handle, &*code_fn_disabled);
        })
        .build(&mut cx);

    let controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Select playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(Label::new("Color").font_size(12.0).color(muted).build(&mut cx))
        .child(color_select)
        .child(Label::new("Selected item").font_size(12.0).color(muted).build(&mut cx))
        .child(selection_select)
        .child(Label::new("Placeholder").font_size(12.0).color(muted).build(&mut cx))
        .child(placeholder_input)
        .child(disabled_switch)
        .build(&mut cx);

    // --- shell -------------------------------------------------------------
    let shell = PlaygroundShell::new(preview, controls, code_panel)
        .sidebar_width(280.0)
        .code_height(300.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(820.0));
    shell_handle
}
