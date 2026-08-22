//! widgets_demo — exercises the Phase 1 static & layout primitives:
//! BoxWidget, Label, Stack, Group, ProgressBar, ThemeScope and Popover.

use fyrox::core::pool::Handle;
use fyrox::gui::UiNode;
use fyrox::gui::UserInterface;
use raikou::prelude::*;
use raikou::Color;
use raikou_demo::Options;

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
        .unwrap_or(Color::new(0.5, 0.5, 0.5, 1.0));

    // A few labels at different hierarchy levels.
    let heading = Label::new("Phase 1 — static & layout primitives")
        .font_size(22.0)
        .color(primary)
        .build(&mut cx);
    let sub = Label::new("BoxWidget, Label, Stack, Group, ProgressBar, ThemeScope, Popover")
        .font_size(14.0)
        .color(muted)
        .build(&mut cx);

    // A horizontal group of colored boxes.
    let blue = Color::new(0.13, 0.39, 0.94, 1.0);
    let blue_light = Color::new(0.25, 0.48, 0.98, 1.0);
    let blue_pale = Color::new(0.79, 0.88, 1.0, 1.0);
    let boxes = Group::new()
        .spacing(12.0)
        .child(
            BoxWidget::new()
                .width(Length::Fixed(48.0))
                .height(48.0)
                .color(blue)
                .corner_radius(8.0)
                .build(&mut cx),
        )
        .child(
            BoxWidget::new()
                .width(Length::Fixed(48.0))
                .height(48.0)
                .color(blue_light)
                .corner_radius(8.0)
                .build(&mut cx),
        )
        .child(
            BoxWidget::new()
                .width(Length::Fixed(48.0))
                .height(48.0)
                .color(blue_pale)
                .corner_radius(8.0)
                .build(&mut cx),
        )
        .child(
            BoxWidget::new()
                .width(Length::Fixed(48.0))
                .height(48.0)
                .color(Color::TRANSPARENT)
                .border_color(blue)
                .border_width(1.0)
                .corner_radius(8.0)
                .build(&mut cx),
        )
        .build(&mut cx);

    // A progress bar with an explicit value.
    let progress = ProgressBar::new()
        .value(0.65)
        .width(Length::Fixed(260.0))
        .build(&mut cx);

    // A ThemeScope wraps a child (pass-through in this architecture).
    let scoped_label = Label::new("Inside ThemeScope")
        .font_size(16.0)
        .build(&mut cx);
    let scope = ThemeScope::new(scoped_label)
        .theme(theme.clone())
        .build(&mut cx);

    // A Popover anchored to a Button. Clicking the button opens it.
    let popover_content = BoxWidget::new()
        .width(Length::Fixed(160.0))
        .color(
            theme
                .color("surface.elevated")
                .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0)),
        )
        .border_color(
            theme
                .color("border.default")
                .unwrap_or(Color::new(0.7, 0.7, 0.7, 1.0)),
        )
        .border_width(1.0)
        .corner_radius(6.0)
        .build(&mut cx);
    let inner_label = Label::new("Popover content")
        .color(
            theme
                .color("text.primary")
                .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0)),
        )
        .build(&mut cx);
    let popover_content_handle: Handle<UiNode> = popover_content.into();
    {
        let ui = cx.ui();
        let mut ctx = ui.build_ctx();
        ctx.link(inner_label.into(), popover_content_handle);
    }

    let popover = Popover::new()
        .content(popover_content_handle)
        .build(&mut cx);
    let popover_handle: Handle<UiNode> = popover.into();

    let owner = Button::new()
        .text("Show popover")
        .variant(ButtonVariant::Outline)
        .on_click(move |ui, _event| {
            show_popover(ui, popover_handle);
        })
        .build(&mut cx);
    let owner_handle: Handle<UiNode> = owner.into();

    Stack::new()
        .spacing(16.0)
        .child(heading)
        .child(sub)
        .child(boxes)
        .child(progress)
        .child(scope)
        .child(owner_handle)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — widgets demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}
