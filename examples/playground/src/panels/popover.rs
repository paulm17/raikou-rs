//! popover panel — playground demo for the raikou `Popover` component.
//!
//! Port of the reference `popover_demo`: a trigger button opens a popover with
//! a title + body. The sidebar exposes Open / Dark-theme switches; the code
//! panel regenerates on every change. The preview is built once with the
//! initial state, so the open switch drives the live popover while the dark
//! toggle only affects the generated source.

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::Color;
use raikou_playground::*;

/// Shared demo state mutated by the control handlers.
#[derive(Clone, Debug, PartialEq)]
struct PlaygroundState {
    open: bool,
    dark_theme: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self {
            open: false,
            dark_theme: false,
        }
    }
}

/// Generates the source shown in the code panel for the current state.
fn build_code(state: &PlaygroundState) -> String {
    let mut code = String::from(
        "let content = Stack::new()\n    .spacing(8.0)\n    .child(Label::new(\"Popover title\"))\n    .child(Label::new(\"Popover body text...\"));\nlet popover = Popover::new()\n    .content(content)\n    .owner(trigger);\n",
    );
    if state.open {
        code.push_str("show_popover(ui, popover);\n");
    }
    if state.dark_theme {
        code.push_str("// dark theme: swap the Popover background & text colors\n");
    }
    code
}

pub fn popover_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let primary = theme
        .color("text.primary")
        .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0));

    let state = Rc::new(RefCell::new(PlaygroundState::default()));

    // The code generator is created first so every handler can refresh the
    // code panel through it.
    let code_state = Rc::clone(&state);
    let code_fn: Rc<dyn Fn() -> String> =
        Rc::new(move || build_code(&code_state.borrow().clone()));

    let code_fn_for_block = Rc::clone(&code_fn);
    let code_handle = PlaygroundCodeBlock::new(move || code_fn_for_block()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Popover.rs", code_handle)
        .height(240.0)
        .build(&mut cx);

    // --- preview content ---------------------------------------------------
    // The popover handle is stored in a cell because the trigger's click
    // handler (built first) needs to toggle it.
    let popover_cell = Rc::new(RefCell::new(Handle::NONE));

    let state_trigger = Rc::clone(&state);
    let popover_for_trigger = Rc::clone(&popover_cell);
    let trigger = Button::new()
        .text("Open/Close popover")
        .on_click(move |ui, _event| {
            let popover = *popover_for_trigger.borrow();
            let open = {
                let mut s = state_trigger.borrow_mut();
                s.open = !s.open;
                s.open
            };
            if open && popover.is_some() {
                show_popover(ui, popover);
            } else {
                hide_popover(ui, popover);
            }
        })
        .build(&mut cx);
    let trigger_handle: Handle<UiNode> = trigger.into();

    let content: Handle<UiNode> = Stack::new()
        .spacing(8.0)
        .child(
            Label::new("Popover title")
                .font_size(15.0)
                .color(Color::new(0.10, 0.12, 0.15, 1.0))
                .build(&mut cx),
        )
        .child(
            Label::new("Popover body text...")
                .font_size(13.0)
                .color(Color::new(0.40, 0.44, 0.50, 1.0))
                .build(&mut cx),
        )
        .build(&mut cx)
        .into();

    let popover = Popover::new()
        .content(content)
        .owner(trigger_handle)
        .build(&mut cx);
    let popover_handle: Handle<UiNode> = popover.into();
    *popover_cell.borrow_mut() = popover_handle;

    let preview_content: Handle<UiNode> = Stack::new()
        .spacing(16.0)
        .child(trigger_handle)
        .child(popover_handle)
        .build(&mut cx)
        .into();
    let preview = PlaygroundPreview::new(preview_content)
        .content_max_size(400.0, 240.0)
        .build(&mut cx);

    // --- controls sidebar --------------------------------------------------
    let state_open = Rc::clone(&state);
    let code_fn_open = Rc::clone(&code_fn);
    let open_switch = Switch::new()
        .text("Open")
        .toggled(false)
        .on_change(move |ui, on| {
            state_open.borrow_mut().open = on;
            if on {
                show_popover(ui, popover_handle);
            } else {
                hide_popover(ui, popover_handle);
            }
            update_code(ui, code_handle, &*code_fn_open);
        })
        .build(&mut cx);

    let state_dark = Rc::clone(&state);
    let code_fn_dark = Rc::clone(&code_fn);
    let dark_switch = Switch::new()
        .text("Dark theme")
        .on_change(move |ui, on| {
            state_dark.borrow_mut().dark_theme = on;
            update_code(ui, code_handle, &*code_fn_dark);
        })
        .build(&mut cx);

    let controls = Stack::new()
        .spacing(12.0)
        .child(
            Label::new("Popover playground")
                .font_size(18.0)
                .color(primary)
                .build(&mut cx),
        )
        .child(
            Label::new("Open")
                .font_size(12.0)
                .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
                .build(&mut cx),
        )
        .child(open_switch)
        .child(dark_switch)
        .build(&mut cx);

    // --- shell -------------------------------------------------------------
    let shell = PlaygroundShell::new(preview, controls, code_panel)
        .sidebar_width(260.0)
        .code_height(240.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
