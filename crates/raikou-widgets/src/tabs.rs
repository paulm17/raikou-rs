//! The Tabs component.
//!
//! Maps onto fyrox's `TabControl`. Each tab is assigned a fresh UUID; a change
//! of the active tab is reported through a per-component `on_change` handler
//! with the tab index.
//!
//! Fluent chrome: headers are padded stacks with a 2px accent underline that
//! tracks the active tab; the selected header uses Avalonia's pale accent
//! tint (#CCE4F7-family) with primary text instead of a saturated fill.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tab_control::{TabControlBuilder, TabControlMessage, TabDefinition};
use fyrox::gui::text::TextBuilder;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use uuid::Uuid;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Handlers of a Tabs component.
#[derive(Clone)]
pub struct TabsHandlers {
    /// UUIDs of the tabs in build order, used to map active-tab events to indexes.
    pub uuids: Vec<Uuid>,
    /// Accent underline nodes, one per tab; visibility tracks the active tab.
    pub underlines: Vec<Handle<UiNode>>,
    /// Invoked with the newly active tab index on change.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl TabsHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(TabControlMessage::ActiveTab(Some(uuid))) = message.data::<TabControlMessage>()
        {
            if let Some(index) = self.uuids.iter().position(|u| u == uuid) {
                // Keep the underline tracking the live selection even when the
                // tab was changed programmatically or by pointer.
                for (i, underline) in self.underlines.iter().enumerate() {
                    ui.send(*underline, WidgetMessage::Visibility(i == index));
                }
                if let Some(callback) = &self.on_change {
                    callback(ui, index);
                }
            }
        }
    }
}

/// Builder for a [`Tabs`] component.
#[derive(Clone)]
pub struct Tabs {
    headers: Vec<String>,
    contents: Vec<Handle<UiNode>>,
    initial_tab: usize,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    /// Creates a new tabs builder.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            contents: Vec::new(),
            initial_tab: 0,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Appends a tab with the given header label and content.
    pub fn tab(mut self, label: impl Into<String>, content: Handle<UiNode>) -> Self {
        self.headers.push(label.into());
        self.contents.push(content);
        self
    }

    /// Sets the index of the initially active tab (default 0).
    pub fn initial_tab(mut self, index: usize) -> Self {
        self.initial_tab = index;
        self
    }

    /// Sets the callback invoked with the active tab index on change.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the tabs, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        use raikou_style::ThemeProvider;

        let theme = cx.theme().clone();
        let dark = theme.variant().is_dark();
        let fallback_accent = raikou_core::Color::new(0.0, 0.47, 0.84, 1.0);
        let accent = theme.color("accent.solid").unwrap_or(fallback_accent);
        // Fluent selected-header fill: pale accent tint in light themes,
        // an elevated neutral surface in dark ones.
        let selected_fill = if dark {
            theme
                .color("surface.elevated")
                .unwrap_or(raikou_core::Color::new(0.18, 0.18, 0.18, 1.0))
        } else {
            theme
                .color("fluent.accent.tint")
                .unwrap_or(raikou_core::Color::new(0.80, 0.89, 0.97, 1.0))
        };

        let mut uuids: Vec<Uuid> = Vec::new();
        let mut underlines: Vec<Handle<UiNode>> = Vec::new();
        let mut tab_builder = TabControlBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_tabs")
                .with_margin(to_fyrox_thickness(self.margin)),
        )
        .with_initial_tab(self.initial_tab);

        for (index, (label, content)) in self.headers.iter().zip(self.contents.iter()).enumerate()
        {
            // Fluent header: padded label over a 2px accent underline that is
            // visible only while the tab is active.
            let (header, underline) = {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                let label_node: Handle<UiNode> = TextBuilder::new(WidgetBuilder::new().with_margin(
                    fyrox::gui::Thickness {
                        left: 12.0,
                        top: 9.0,
                        right: 12.0,
                        bottom: 7.0,
                    },
                ))
                .with_text(label)
                .with_font(font)
                .build(&mut ctx)
                .to_base();

                let underline: Handle<UiNode> = {
                    use fyrox::gui::border::BorderBuilder;
                    BorderBuilder::new(
                        WidgetBuilder::new()
                            .with_name("raikou_tab_underline")
                            .with_height(2.0)
                            .with_visibility(index == self.initial_tab)
                            .with_background(
                                fyrox::gui::brush::Brush::Solid(to_fyrox_color(accent)).into(),
                            ),
                    )
                    .with_stroke_thickness(fyrox::gui::Thickness::uniform(0.0).into())
                    .build(&mut ctx)
                    .to_base()
                };

                let stack: Handle<UiNode> = {
                    use fyrox::gui::stack_panel::StackPanelBuilder;
                    StackPanelBuilder::new(
                        WidgetBuilder::new()
                            .with_name("raikou_tab_header")
                            .with_children([label_node, underline]),
                    )
                    .build(&mut ctx)
                    .to_base()
                };
                (stack, underline)
            };
            underlines.push(underline);

            let uuid = Uuid::new_v4();
            uuids.push(uuid);
            tab_builder = tab_builder.with_tab(TabDefinition {
                uuid,
                header,
                content: *content,
                can_be_closed: false,
                user_data: None,
            });
        }

        let handle = {
            let mut ctx = cx.ctx();
            tab_builder.build(&mut ctx).to_base()
        };

        // Fluent chrome: strip the stock TabControl backdrop, give headers
        // transparent/hover/selected brushes (pale tint when selected), pin
        // label text to the theme's primary color, and pad nothing post-hoc
        // (header stacks already carry comfortable padding).
        {
            use fyrox::graph::SceneGraph;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::DecoratorMessage;

            let token_brush =
                |name: &str| Brush::Solid(to_fyrox_color(theme.color(name).unwrap())).into();
            let transparent = Brush::Solid(fyrox::core::color::Color::TRANSPARENT);
            let transparent_prop: fyrox::gui::style::StyledProperty<Brush> =
                transparent.clone().into();
            let selected_prop: fyrox::gui::style::StyledProperty<Brush> =
                Brush::Solid(to_fyrox_color(selected_fill)).into();
            let text_prop: fyrox::gui::style::StyledProperty<Brush> =
                token_brush("text.primary");
            let ui = cx.ui();

            let root_node = ui.node(handle);
            let border_h = *root_node.children().first().expect("tab control border");
            let grid_h = *ui
                .node(border_h)
                .children()
                .first()
                .expect("tab control grid");
            let headers_h = *ui
                .node(grid_h)
                .children()
                .first()
                .expect("headers container");
            ui.send(
                border_h,
                WidgetMessage::Background(transparent_prop.clone()),
            );
            ui.send(
                border_h,
                WidgetMessage::Foreground(transparent_prop.clone()),
            );

            for header_button in ui.node(headers_h).children().to_vec() {
                // Decorators sit between the button and its content grid;
                // decorator messages must target them directly.
                let Some(decorator) = ui.node(header_button).children().first().copied() else {
                    continue;
                };
                ui.send(
                    decorator,
                    DecoratorMessage::NormalBrush(transparent_prop.clone()),
                );
                ui.send(
                    decorator,
                    DecoratorMessage::HoverBrush(token_brush("fluent.list.low")),
                );
                ui.send(
                    decorator,
                    DecoratorMessage::PressedBrush(token_brush("fluent.list.medium")),
                );
                // The native TabControl drives this brush per active tab.
                ui.send(decorator, DecoratorMessage::SelectedBrush(selected_prop.clone()));

                // Pin the label color so selected headers read as dark-on-tint
                // (light) / light-on-gray (dark).
                if let Some(label) = find_text_child(ui, header_button) {
                    ui.send(label, WidgetMessage::Foreground(text_prop.clone()));
                }
            }
        }

        let component = Component {
            handle,
            kind: ComponentKind::Tabs(TabsHandlers {
                uuids,
                underlines,
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

/// Finds the first Text node under `root` (the header label).
fn find_text_child(
    ui: &UserInterface,
    root: Handle<UiNode>,
) -> Option<Handle<UiNode>> {
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

/// A handle to a built tabs container.
pub type TabsHandle = Handle<UiNode>;
