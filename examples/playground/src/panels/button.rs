//! button panel — playground demo for the raikou `Button` component.
//!
//! Port of the reference `button_demo`: a preview button whose label, variant,
//! size, width, radius and boolean flags are all driven from the controls
//! sidebar. Only the enabled flag is applied to the live preview (via a
//! message); the remaining options regenerate the code panel.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

/// Color presets selectable in the controls sidebar.
const COLORS: &[(&str, Color)] = &[
    ("Sky", Color::new(0.20, 0.58, 0.95, 1.0)),
    ("Coral", Color::new(0.95, 0.37, 0.34, 1.0)),
    ("Emerald", Color::new(0.12, 0.72, 0.48, 1.0)),
    ("Violet", Color::new(0.56, 0.34, 0.93, 1.0)),
    ("Slate", Color::new(0.36, 0.42, 0.50, 1.0)),
];

const VARIANTS: [&str; 5] = ["Filled", "Outline", "Ghost", "Subtle", "Link"];

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    label: String,
    variant: usize,
    color: usize,
    size: usize,
    width: f32,
    radius: f32,
    loading: bool,
    enabled: bool,
    is_default: bool,
    is_cancel: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            label: "Button".to_string(),
            variant: 0,
            color: 0,
            size: 2,
            width: 168.0,
            radius: 10.0,
            loading: false,
            enabled: true,
            is_default: false,
            is_cancel: false,
        }
    }
}

/// Maps the size slider index to a `ControlSize`.
fn control_size(index: usize) -> ControlSize {
    match index {
        0 => ControlSize::XSmall,
        1 => ControlSize::Small,
        2 => ControlSize::Medium,
        3 => ControlSize::Large,
        _ => ControlSize::XLarge,
    }
}

fn control_size_name(index: usize) -> &'static str {
    match index {
        0 => "ControlSize::XSmall",
        1 => "ControlSize::Small",
        2 => "ControlSize::Medium",
        3 => "ControlSize::Large",
        _ => "ControlSize::XLarge",
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let variant = VARIANTS[state.variant];
    let mut code = format!(
        "let button = Button::new()\n    .text({:?})\n    .variant(ButtonVariant::{variant})\n    .size({})\n    .width(Length::Fixed({:.0}))\n    .corner_radius({:.0})\n    .margin(Thickness::uniform(4.0))",
        state.label,
        control_size_name(state.size),
        state.width,
        state.radius,
    );
    if state.loading {
        code.push_str("\n    .is_loading(true)");
    }
    if state.is_default {
        code.push_str("\n    .is_default(true)");
    }
    if state.is_cancel {
        code.push_str("\n    .is_cancel(true)");
    }
    code.push_str(";\n");
    if !state.enabled {
        code.push_str("// ui.send(button, WidgetMessage::Enabled(false));\n");
    }
    code.push_str(&format!("// accent: {}\n", COLORS[state.color].0));
    code
}

pub fn button_panel(
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
    let code_fn: Rc<dyn Fn() -> String> = Rc::new(move || build_code(&code_state.borrow().clone()));

    let code_fn_for_block = Rc::clone(&code_fn);
    let code_handle = PlaygroundCodeBlock::new(move || code_fn_for_block()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Button.rs", code_handle)
        .height(260.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    let defaults = PlaygroundState::default();
    // RAIKOU_BUTTON_VARIANT=outline: capture a bare Fluent-style default-ish
    // button (Outline, no playground width/radius overrides) for the audit.
    let audit_outline = std::env::var("RAIKOU_BUTTON_VARIANT").as_deref() == Ok("outline");
    let preview_variant = if audit_outline {
        ButtonVariant::Outline
    } else {
        match defaults.variant {
            0 => ButtonVariant::Filled,
            1 => ButtonVariant::Outline,
            2 => ButtonVariant::Ghost,
            3 => ButtonVariant::Subtle,
            _ => ButtonVariant::Link,
        }
    };
    let mut preview_button = Button::new()
        .text(&defaults.label)
        .variant(preview_variant)
        .size(control_size(defaults.size));
    if !audit_outline {
        preview_button = preview_button
            .width(Length::Fixed(defaults.width))
            .corner_radius(defaults.radius);
    }
    let preview_button = preview_button
        .on_click(|ui, _event| {
            let _ = ui;
        })
        .build(&mut cx);
    let button_handle: Handle<UiNode> = preview_button.into();

    let preview = PlaygroundPreview::new(button_handle)
        .content_max_size(420.0, 200.0)
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

    let state_variant = Rc::clone(&state);
    let code_fn_variant = Rc::clone(&code_fn);
    let variant_select = Select::new()
        .items(VARIANTS.to_vec())
        .selected(defaults.variant)
        .on_change(move |ui, index| {
            state_variant.borrow_mut().variant = index;
            update_code(ui, code_handle, &*code_fn_variant);
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
        .min(0.0)
        .max(4.0)
        .step(1.0)
        .value(defaults.size as f32)
        .on_change(move |ui, v| {
            state_size.borrow_mut().size = v.round() as usize;
            update_code(ui, code_handle, &*code_fn_size);
        })
        .build(&mut cx);

    let state_width = Rc::clone(&state);
    let code_fn_width = Rc::clone(&code_fn);
    let width_slider = control_slider()
        .min(120.0)
        .max(320.0)
        .step(8.0)
        .value(defaults.width)
        .on_change(move |ui, v| {
            state_width.borrow_mut().width = v;
            update_code(ui, code_handle, &*code_fn_width);
        })
        .build(&mut cx);

    let state_radius = Rc::clone(&state);
    let code_fn_radius = Rc::clone(&code_fn);
    let radius_slider = control_slider()
        .min(0.0)
        .max(28.0)
        .step(1.0)
        .value(defaults.radius)
        .on_change(move |ui, v| {
            state_radius.borrow_mut().radius = v;
            update_code(ui, code_handle, &*code_fn_radius);
        })
        .build(&mut cx);

    let state_loading = Rc::clone(&state);
    let code_fn_loading = Rc::clone(&code_fn);
    let loading_switch = Switch::new()
        .text("Loading")
        .on_change(move |ui, on| {
            state_loading.borrow_mut().loading = on;
            update_code(ui, code_handle, &*code_fn_loading);
        })
        .build(&mut cx);

    let state_enabled = Rc::clone(&state);
    let code_fn_enabled = Rc::clone(&code_fn);
    let enabled_switch = Switch::new()
        .text("Enabled")
        .toggled(true)
        .on_change(move |ui, on| {
            state_enabled.borrow_mut().enabled = on;
            ui.send(button_handle, WidgetMessage::Enabled(on));
            update_code(ui, code_handle, &*code_fn_enabled);
        })
        .build(&mut cx);

    let state_default = Rc::clone(&state);
    let code_fn_default = Rc::clone(&code_fn);
    let default_switch = Switch::new()
        .text("Default button")
        .on_change(move |ui, on| {
            state_default.borrow_mut().is_default = on;
            update_code(ui, code_handle, &*code_fn_default);
        })
        .build(&mut cx);

    let state_cancel = Rc::clone(&state);
    let code_fn_cancel = Rc::clone(&code_fn);
    let cancel_switch = Switch::new()
        .text("Cancel button")
        .on_change(move |ui, on| {
            state_cancel.borrow_mut().is_cancel = on;
            update_code(ui, code_handle, &*code_fn_cancel);
        })
        .build(&mut cx);

    let controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Button playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(
            Label::new("Label")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(label_input)
        .child(
            Label::new("Variant")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(variant_select)
        .child(
            Label::new("Color")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(color_select)
        .child(
            Label::new("Size")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(size_slider)
        .child(
            Label::new("Width")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(width_slider)
        .child(
            Label::new("Radius")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(radius_slider)
        .child(loading_switch)
        .child(enabled_switch)
        .child(default_switch)
        .child(cancel_switch)
        .build(&mut cx);

    // --- shell -------------------------------------------------------------
    let shell = PlaygroundShell::new(preview, controls, code_panel)
        .sidebar_width(260.0)
        .code_height(260.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(920.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
