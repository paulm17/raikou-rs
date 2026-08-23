//! scroll_area panel — playground demo for the raikou `ScrollArea`.
//!
//! Port of the reference `scroll_area_demo`: a grid of buttons overflowing in
//! both axes inside a scrollable area. The reference scroll-bar-visibility and
//! line-step knobs map to `horizontal/vertical_scroll_allowed` and `v_scroll_speed`.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r#"ScrollArea::new()
    .horizontal_scroll_allowed(true)
    .vertical_scroll_allowed(true)
    .content(content)"#;

pub fn scroll_area_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let mut rows = Vec::new();
    for r in 0..6 {
        let mut buttons = Vec::new();
        for c in 0..4 {
            buttons.push(
                Button::new()
                    .text(format!("Item {}-{}", r + 1, c + 1))
                    .width(Length::Fixed(150.0))
                    .build(&mut cx),
            );
        }
        rows.push(Group::new().spacing(12.0).children(buttons).build(&mut cx));
    }

    let content: Handle<UiNode> = Stack::new()
        .spacing(14.0)
        .children(rows)
        .build(&mut cx)
        .into();

    let scroll = ScrollArea::new()
        .content(content)
        .horizontal_scroll_allowed(true)
        .vertical_scroll_allowed(true)
        .v_scroll_speed(52.0)
        .build(&mut cx);

    let preview = PlaygroundPreview::new(scroll)
        .content_max_size(420.0, 240.0)
        .build(&mut cx);

    // RAIKOU_SCROLL_SHOW=1 reveals the overlay thumbs (they are hidden until
    // hover/scroll by default) so captures can show the shown state.
    if std::env::var("RAIKOU_SCROLL_SHOW").as_deref() == Ok("1") {
        use fyrox::graph::SceneGraph;
        use fyrox::gui::scroll_bar::ScrollBar;
        let ui = cx.ui();
        let mut stack = vec![Handle::<UiNode>::from(&preview)];
        while let Some(handle) = stack.pop() {
            if handle.is_none() {
                continue;
            }
            if let Ok(bar) = ui.try_get_of_type::<ScrollBar>(handle) {
                ui.send(*bar.indicator, WidgetMessage::Visibility(true));
            }
            for child in ui.node(handle).children().to_vec() {
                stack.push(child);
            }
        }
    }

    let notes = playground_notes(
        &mut cx,
        "ScrollArea playground",
        &[
            "The content overflows in both axes, exercising both scroll directions.",
            "Scroll with the wheel or drag the scroll bars to pan the grid.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("ScrollArea.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));
    shell_handle
}
