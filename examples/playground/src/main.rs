//! Unified raikou playground: one binary with a scrollable sidebar listing
//! every component; the right side shows the selected component's full
//! playground shell (preview + controls + generated code).

mod panels;

use std::cell::RefCell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface, VerticalAlignment};
use raikou::prelude::*;
use raikou::{to_fyrox_color, Color, Thickness};
use raikou_demo::Options;
use raikou_style::Theme;
use raikou_widgets::{BuildCx, ComponentRegistry};

/// A sidebar entry: label + panel constructor (`None` for Home).
type PanelFn = fn(&mut UserInterface, &Theme, &mut ComponentRegistry) -> Handle<UiNode>;

const PANELS: &[(&str, PanelFn)] = &[
    ("Accordion", panels::accordion::accordion_panel),
    ("Button", panels::button::button_panel),
    ("Checkbox", panels::checkbox::checkbox_panel),
    ("Combobox", panels::combobox::combobox_panel),
    ("Context menu", panels::context_menu::context_menu_panel),
    ("Image", panels::image::image_panel),
    ("Label", panels::label::label_panel),
    (
        "Loading indicator",
        panels::loading_indicator::loading_indicator_panel,
    ),
    ("Menu", panels::menu::menu_panel),
    ("Popover", panels::popover::popover_panel),
    ("Progress bar", panels::progress_bar::progress_bar_panel),
    ("Radio", panels::radio::radio_panel),
    ("Scroll area", panels::scroll_area::scroll_area_panel),
    ("Select", panels::select::select_panel),
    ("Slider", panels::slider::slider_panel),
    ("Step input", panels::step_input::step_input_panel),
    ("Switch", panels::switch::switch_panel),
    ("Table", panels::table::table_panel),
    ("Tabs", panels::tabs::tabs_panel),
    ("Text area", panels::text_area::text_area_panel),
    ("Text input", panels::text_input::text_input_panel),
    ("Tree", panels::tree::tree_panel),
];

/// Shared selection state flipped by sidebar clicks.
#[derive(Default)]
struct Selection {
    /// Host widget per entry (index 0 = Home).
    hosts: Vec<Handle<UiNode>>,
    /// Sidebar button per entry.
    buttons: Vec<Handle<UiNode>>,
}

/// Finds the first Text node under `root` (the label of a nav button).
fn find_text_child(
    ui: &UserInterface,
    root: fyrox::core::pool::Handle<UiNode>,
) -> Option<fyrox::core::pool::Handle<UiNode>> {
    use fyrox::graph::SceneGraph;
    let mut stack = vec![root];
    while let Some(h) = stack.pop() {
        if h.is_none() {
            continue;
        }
        if ui.try_get_of_type::<fyrox::gui::text::Text>(h).is_ok() {
            return Some(h);
        }
        for c in ui.node(h).children() {
            stack.push(*c);
        }
    }
    None
}

fn select(state: &RefCell<Selection>, ui: &mut UserInterface, theme: &Theme, index: usize) {
    use fyrox::graph::SceneGraph;
    use fyrox::gui::decorator::{Decorator, DecoratorMessage};
    let state = state.borrow();
    // Active pill = accent background + white label; inactive = transparent.
    let accent = theme
        .color("accent.solid")
        .unwrap_or(Color::new(0.0, 0.47, 0.84, 1.0));
    let accent_hover = theme
        .color("accent.hover")
        .unwrap_or(Color::new(0.26, 0.61, 0.89, 1.0));
    let idle_text = theme
        .color("text.primary")
        .unwrap_or(Color::new(0.06, 0.06, 0.07, 1.0));
    let hover_idle = theme.color("fluent.control.hover");
    let pressed_idle = theme.color("fluent.control.pressed");
    let prop = |c: fyrox::core::color::Color| Brush::Solid(c).into();
    for (i, (&host, &button)) in state.hosts.iter().zip(state.buttons.iter()).enumerate() {
        let is_active = i == index;
        ui.send(host, WidgetMessage::Visibility(is_active));

        // The decorator paints the button; WidgetMessage::Background on the
        // button itself is hidden behind it (see switch.rs).
        let decorator = ui
            .node(button)
            .children()
            .first()
            .copied()
            .filter(|h| ui.try_get_of_type::<Decorator>(*h).is_ok());
        if let Some(decorator) = decorator {
            let (normal, hover, pressed) = if is_active {
                (
                    to_fyrox_color(accent),
                    to_fyrox_color(accent_hover),
                    to_fyrox_color(theme.color("accent.pressed").unwrap_or(accent)),
                )
            } else {
                (
                    fyrox::core::color::Color::TRANSPARENT,
                    to_fyrox_color(hover_idle.unwrap_or(Color::new(0.96, 0.96, 0.96, 1.0))),
                    to_fyrox_color(pressed_idle.unwrap_or(Color::new(0.92, 0.92, 0.92, 1.0))),
                )
            };
            ui.send(decorator, DecoratorMessage::NormalBrush(prop(normal)));
            ui.send(decorator, DecoratorMessage::HoverBrush(prop(hover)));
            ui.send(decorator, DecoratorMessage::PressedBrush(prop(pressed)));
        }

        if let Some(text) = find_text_child(ui, button) {
            ui.send(
                text,
                WidgetMessage::Foreground(if is_active {
                    prop(fyrox::core::color::Color::WHITE)
                } else {
                    prop(to_fyrox_color(idle_text))
                }),
            );
        }
    }
}

fn build_app(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let fallback_primary = Color::new(0.06, 0.06, 0.07, 1.0);
    let fallback_muted = Color::new(0.45, 0.45, 0.47, 1.0);

    // --- pass 1: home content + sidebar (raikou components) ---------------
    let selection = Rc::new(RefCell::new(Selection::default()));

    let (home_stack, sidebar): (Handle<UiNode>, Handle<UiNode>) = {
        let mut cx = BuildCx::new(ui, theme, registry);

        let title = Label::new("raikou playground")
            .font_size(28.0)
            .color(theme.color("text.primary").unwrap_or(fallback_primary))
            .margin(Thickness::uniform(8.0))
            .build(&mut cx);
        let subtitle = Label::new(
            "Fluent-styled components for Fyrox. Pick a component from the \
             list on the left to open its live playground.",
        )
        .font_size(14.0)
        .color(theme.color("text.muted").unwrap_or(fallback_muted))
        .margin(Thickness::uniform(8.0))
        .build(&mut cx);
        let hint = Label::new(
            "Every playground shows a live preview with working controls, \
             plus the exact Rust snippet that reproduces it.",
        )
        .font_size(14.0)
        .color(theme.color("text.muted").unwrap_or(fallback_muted))
        .margin(Thickness::uniform(8.0))
        .build(&mut cx);

        let home_stack: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("playground_home")
                    .with_vertical_alignment(VerticalAlignment::Top)
                    .with_child(title.into())
                    .with_child(subtitle.into())
                    .with_child(hint.into()),
            )
            .build(&mut ctx)
            .to_base()
        };

        let mut buttons: Vec<Handle<UiNode>> = Vec::new();
        let b = Button::new()
            .text("Home")
            .variant(ButtonVariant::Ghost)
            .width(Length::Fixed(184.0))
            .margin(Thickness::uniform(2.0))
            .on_click({
                let sel = Rc::clone(&selection);
                let theme = theme.clone();
                move |ui, _| select(&sel, ui, &theme, 0)
            })
            .build(&mut cx);
        buttons.push(b.handle);

        for (i, (label, _)) in PANELS.iter().enumerate() {
            let sel = Rc::clone(&selection);
            let theme = theme.clone();
            let idx = i + 1;
            let b = Button::new()
                .text((*label).to_string())
                .variant(ButtonVariant::Ghost)
                .width(Length::Fixed(184.0))
                .margin(Thickness::uniform(2.0))
                .on_click(move |ui, _| select(&sel, ui, &theme, idx))
                .build(&mut cx);
            buttons.push(b.handle);
        }

        let sidebar_list: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("playground_sidebar_list")
                    .with_children(buttons.iter().copied()),
            )
            .build(&mut ctx)
            .to_base()
        };

        let sidebar = ScrollArea::new()
            .content(sidebar_list)
            .vertical_scroll_allowed(true)
            .width(Length::Fixed(210.0))
            .height(Length::Fixed(780.0))
            .build(&mut cx);

        {
            let mut s = selection.borrow_mut();
            s.buttons = buttons;
        }

        (home_stack, sidebar.into())
    };

    // --- pass 2: mount every component panel ------------------------------
    let mut hosts: Vec<Handle<UiNode>> = vec![home_stack];
    for (_, panel) in PANELS {
        hosts.push(panel(ui, theme, registry));
    }
    selection.borrow_mut().hosts = hosts.clone();

    // --- pass 3: overlay + root --------------------------------------------
    let root: Handle<UiNode> = {
        let mut cx = BuildCx::new(ui, theme, registry);
        use fyrox::gui::border::BorderBuilder;
        use fyrox::gui::grid::{GridBuilder, GridDimension};

        // fyrox never paints a background for the root canvas or plain
        // widgets, so every unpainted region shows the window's raw clear
        // color (black in a real window). A full-window Border beneath the
        // app content fixes that; a Grid stretches it to fill.
        let page = theme
            .color("surface.canvas")
            .unwrap_or_else(|| raikou::Color::new(0.97, 0.97, 0.98, 1.0));

        let overlay: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            GridBuilder::new(
                WidgetBuilder::new()
                    .with_name("playground_overlay")
                    .with_children(hosts.iter().copied()),
            )
            .add_row(GridDimension::auto())
            .add_column(GridDimension::auto())
            .build(&mut ctx)
            .to_base()
        };

        let content: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            // Horizontal stack of [sidebar | panel host]. Children without
            // explicit on_row/on_column all land in Grid cell (0,0), so a
            // plain StackPanel is used here instead of a second Grid.
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("playground_root")
                    .with_child(sidebar)
                    .with_child(overlay),
            )
            .with_orientation(fyrox::gui::Orientation::Horizontal)
            .build(&mut ctx)
            .to_base()
        };

        let backdrop: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            BorderBuilder::new(
                WidgetBuilder::new()
                    .with_name("playground_backdrop")
                    .with_hit_test_visibility(false)
                    .with_background(Brush::Solid(to_fyrox_color(page)).into()),
            )
            .with_stroke_thickness(fyrox::gui::Thickness::uniform(0.0).into())
            .build(&mut ctx)
            .to_base()
        };

        let mut ctx = cx.ctx();
        GridBuilder::new(
            WidgetBuilder::new()
                .with_name("playground_window")
                .with_child(backdrop)
                .with_child(content),
        )
        .add_row(GridDimension::stretch())
        .add_column(GridDimension::stretch())
        .build(&mut ctx)
        .to_base()
    };

    // Initial selection: Button (entry 1). Applied once messages pump.
    // RAIKOU_PANEL=<name> selects a sidebar entry by name (used by shot.sh).
    {
        let sel = Rc::clone(&selection);
        let theme = theme.clone();
        let index = std::env::var("RAIKOU_PANEL")
            .ok()
            .and_then(|name| {
                PANELS
                    .iter()
                    .position(|(label, _)| label.eq_ignore_ascii_case(&name))
            })
            .map(|panel| panel + 1) // entry 0 is Home
            .unwrap_or(2); // Button
        select(&sel, ui, &theme, index);
    }

    root
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou playground".to_string(),
            width: 1240,
            height: 820,
        },
        Box::new(build_app),
    );
}
