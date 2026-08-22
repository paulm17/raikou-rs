//! accordion panel — playground demo for the raikou `Accordion` component.
//!
//! Port of the reference `accordion_demo`: a three-section accordion preview
//! whose labels, titles, accents and expansion state are driven from the
//! sidebar. Only the width and allow-multiple flags are live; the rest
//! regenerate the code panel.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

/// Accent color presets for the accordion sections.
const ACCENTS: &[(&str, Color)] = &[
    ("Sky", Color::new(0.21, 0.58, 0.93, 1.0)),
    ("Emerald", Color::new(0.18, 0.70, 0.49, 1.0)),
    ("Amber", Color::new(0.93, 0.63, 0.18, 1.0)),
    ("Coral", Color::new(0.91, 0.39, 0.34, 1.0)),
    ("Slate", Color::new(0.39, 0.45, 0.55, 1.0)),
];

/// State for a single accordion section.
#[derive(Clone, Debug, PartialEq)]
struct AccordionItemState {
    label: String,
    title: String,
    body: String,
    accent: usize,
    expanded: bool,
}

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    width: f32,
    allow_multiple: bool,
    items: Vec<AccordionItemState>,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            width: 360.0,
            allow_multiple: false,
            items: vec![
                AccordionItemState {
                    label: "Shipping".to_string(),
                    title: "Fast delivery".to_string(),
                    body: "Orders placed before 2pm leave the warehouse the same day.".to_string(),
                    accent: 0,
                    expanded: true,
                },
                AccordionItemState {
                    label: "Returns".to_string(),
                    title: "30 day returns".to_string(),
                    body: "Unused items can be returned without a restocking fee.".to_string(),
                    accent: 1,
                    expanded: false,
                },
                AccordionItemState {
                    label: "Support".to_string(),
                    title: "Human support".to_string(),
                    body: "Chat and email support are available Monday to Friday.".to_string(),
                    accent: 2,
                    expanded: false,
                },
            ],
        }
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let mut code = format!(
        "let accordion = Accordion::new()\n    .width(Length::Fixed({:.0}))",
        state.width,
    );
    if state.allow_multiple {
        code.push_str("\n    .allow_multiple(true)");
    }
    code.push_str(";\n");
    for item in &state.items {
        code.push_str(&format!(
            "// section {:?} — accent: {} (expanded: {})\n",
            item.label, ACCENTS[item.accent].0, item.expanded
        ));
    }
    code
}

pub fn accordion_panel(
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
    let code_panel = PlaygroundCodePanel::new("Accordion.rs", code_handle)
        .height(280.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    // Build one content handle per section; the accordion is built once with
    // the initial state.
    let mut sections: Vec<Handle<UiNode>> = Vec::new();
    for item in &PlaygroundState::default().items {
        let content: Handle<UiNode> = Stack::new()
            .spacing(8.0)
            .child(
                Label::new(&item.title)
                    .font_size(14.0)
                    .color(primary)
                    .build(&mut cx),
            )
            .child(
                Label::new(&item.body)
                    .font_size(12.0)
                    .color(muted)
                    .build(&mut cx),
            )
            .build(&mut cx)
            .into();
        sections.push(content);
    }

    let defaults = PlaygroundState::default();
    let mut accordion = Accordion::new();
    accordion = accordion.allow_multiple(defaults.allow_multiple);
    for (i, item) in defaults.items.iter().enumerate() {
        let section = sections[i];
        accordion = if item.expanded {
            accordion.item_with_content_expanded(&item.label, section)
        } else {
            accordion.item_with_content(&item.label, section)
        };
    }
    let state_toggle = Rc::clone(&state);
    accordion = accordion.on_toggle(move |_ui, index, expanded| {
        let mut s = state_toggle.borrow_mut();
        if !s.allow_multiple && expanded {
            for (i, it) in s.items.iter_mut().enumerate() {
                it.expanded = i == index;
            }
        } else if let Some(it) = s.items.get_mut(index) {
            it.expanded = expanded;
        }
    });
    let accordion = accordion.build(&mut cx);
    let accordion_handle: Handle<UiNode> = accordion.into();

    let preview = PlaygroundPreview::new(accordion_handle)
        .content_max_size(400.0, 320.0)
        .build(&mut cx);

    // --- controls sidebar --------------------------------------------------
    let state_width = Rc::clone(&state);
    let code_fn_width = Rc::clone(&code_fn);
    let width_slider = control_slider()
        .min(240.0)
        .max(400.0)
        .step(8.0)
        .value(defaults.width)
        .on_change(move |ui, v| {
            state_width.borrow_mut().width = v;
            update_code(ui, code_handle, &*code_fn_width);
        })
        .build(&mut cx);

    let state_multiple = Rc::clone(&state);
    let code_fn_multiple = Rc::clone(&code_fn);
    let multiple_switch = Switch::new()
        .text("Allow multiple")
        .on_change(move |ui, on| {
            state_multiple.borrow_mut().allow_multiple = on;
            update_code(ui, code_handle, &*code_fn_multiple);
        })
        .build(&mut cx);

    let mut controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Accordion playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(
            Label::new("Width")
                .font_size(12.0)
                .color(muted)
                .build(&mut cx),
        )
        .child(width_slider)
        .child(multiple_switch);

    let mut per_section = Vec::new();
    for i in 0..defaults.items.len() {
        let state_label = Rc::clone(&state);
        let code_fn_label = Rc::clone(&code_fn);
        let label_input = TextInput::new()
            .text(&defaults.items[i].label)
            .on_change(move |ui, text| {
                if let Some(item) = state_label.borrow_mut().items.get_mut(i) {
                    item.label = text.to_string();
                }
                update_code(ui, code_handle, &*code_fn_label);
            })
            .build(&mut cx);

        let state_title = Rc::clone(&state);
        let code_fn_title = Rc::clone(&code_fn);
        let title_input = TextInput::new()
            .text(&defaults.items[i].title)
            .on_change(move |ui, text| {
                if let Some(item) = state_title.borrow_mut().items.get_mut(i) {
                    item.title = text.to_string();
                }
                update_code(ui, code_handle, &*code_fn_title);
            })
            .build(&mut cx);

        let accent_names: Vec<String> = ACCENTS.iter().map(|(n, _)| n.to_string()).collect();
        let state_accent = Rc::clone(&state);
        let code_fn_accent = Rc::clone(&code_fn);
        let accent_select = Select::new()
            .items(accent_names)
            .selected(defaults.items[i].accent)
            .on_change(move |ui, index| {
                if let Some(item) = state_accent.borrow_mut().items.get_mut(i) {
                    item.accent = index;
                }
                update_code(ui, code_handle, &*code_fn_accent);
            })
            .build(&mut cx);

        controls = controls
            .child(
                Label::new(format!("Section {}", i + 1))
                    .font_size(13.0)
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
                Label::new("Title")
                    .font_size(12.0)
                    .color(muted)
                    .build(&mut cx),
            )
            .child(title_input)
            .child(
                Label::new("Accent")
                    .font_size(12.0)
                    .color(muted)
                    .build(&mut cx),
            )
            .child(accent_select);
        per_section.push(());
    }
    let _ = per_section;

    let controls = controls.build(&mut cx);

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
