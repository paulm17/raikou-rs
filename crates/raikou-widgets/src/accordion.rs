//! The Accordion component.
//!
//! Maps onto a vertical stack of fyrox `Expander` widgets. Expansion toggles
//! are reported through a per-component `on_toggle` handler. When
//! `allow_multiple` is false, opening one item collapses the others.

use std::rc::Rc;

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::check_box::{CheckBox, CheckBoxBuilder};
use fyrox::gui::expander::{ExpanderBuilder, ExpanderMessage};
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::text::TextBuilder;
use fyrox::gui::vector_image::{Primitive, VectorImageBuilder};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{Orientation, UiNode, UserInterface, VerticalAlignment};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

type ToggleCallback = dyn Fn(&mut UserInterface, usize, bool);

/// Builds the Fluent chevron marks: pointing down when expanded, right when
/// collapsed. Drawn as vector lines colored `text.secondary` (no font or
/// texture dependency). Shared by the accordion and tree restyling.
///
/// Marks come back centered within their host box.
pub(crate) fn chevron_mark_nodes(
    ctx: &mut fyrox::gui::BuildContext,
    theme: &raikou_style::Theme,
) -> (Handle<UiNode>, Handle<UiNode>) {
    use fyrox::gui::HorizontalAlignment;

    let fallback = raikou_core::Color::new(0.35, 0.35, 0.39, 1.0);
    let color = to_fyrox_color(theme.color("text.secondary").unwrap_or(fallback));
    let t = 1.5;
    let down = vec![
        Primitive::Line {
            begin: Vector2::new(2.0, 3.5),
            end: Vector2::new(5.0, 6.5),
            thickness: t,
        },
        Primitive::Line {
            begin: Vector2::new(5.0, 6.5),
            end: Vector2::new(8.0, 3.5),
            thickness: t,
        },
    ];
    let right = vec![
        Primitive::Line {
            begin: Vector2::new(3.5, 2.0),
            end: Vector2::new(6.5, 5.0),
            thickness: t,
        },
        Primitive::Line {
            begin: Vector2::new(6.5, 5.0),
            end: Vector2::new(3.5, 8.0),
            thickness: t,
        },
    ];
    let mark = |prims: Vec<Primitive>, ctx: &mut fyrox::gui::BuildContext| -> Handle<UiNode> {
        VectorImageBuilder::new(
            WidgetBuilder::new()
                .with_width(10.0)
                .with_height(10.0)
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Center)
                .with_foreground(Brush::Solid(color).into()),
        )
        .with_primitives(prims)
        .build(ctx)
        .to_base()
    };
    (mark(down, ctx), mark(right, ctx))
}

/// Builds the Fluent expander checkbox: bare chevron marks on a transparent
/// background (no stock button chrome).
pub(crate) fn fluent_expander_checkbox(
    ctx: &mut fyrox::gui::BuildContext,
    theme: &raikou_style::Theme,
    expanded: bool,
) -> Handle<CheckBox> {
    let (check_mark, uncheck_mark) = chevron_mark_nodes(ctx, theme);
    CheckBoxBuilder::new(WidgetBuilder::new().with_vertical_alignment(VerticalAlignment::Center))
        .with_check_mark(check_mark)
        .with_uncheck_mark(uncheck_mark)
        .with_background(
            fyrox::gui::border::BorderBuilder::new(
                WidgetBuilder::new()
                    .with_min_size(Vector2::new(10.0, 10.0))
                    .with_background(Brush::Solid(fyrox::core::color::Color::TRANSPARENT).into())
                    .with_foreground(Brush::Solid(fyrox::core::color::Color::TRANSPARENT).into()),
            )
            .with_stroke_thickness(fyrox::gui::Thickness::zero().into())
            .with_pad_by_corner_radius(false)
            .build(ctx),
        )
        .checked(Some(expanded))
        .build(ctx)
}

/// Handlers for one expander item within an [`Accordion`].
#[derive(Clone)]
pub struct AccordionItemHandlers {
    /// Index of this item within the accordion.
    pub index: usize,
    /// Whether multiple items may be open at once.
    pub allow_multiple: bool,
    /// Handles of all sibling expanders (to collapse when exclusive).
    pub siblings: Vec<Handle<UiNode>>,
    /// Invoked with the item index and new expanded state on toggle.
    pub on_toggle: Option<Rc<ToggleCallback>>,
}

impl AccordionItemHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(ExpanderMessage::Expand(expanded)) = message.data::<ExpanderMessage>() {
            if !self.allow_multiple && *expanded {
                for sibling in &self.siblings {
                    ui.send(*sibling, ExpanderMessage::Expand(false));
                }
            }
            if let Some(callback) = &self.on_toggle {
                callback(ui, self.index, *expanded);
            }
        }
    }
}

/// An item of an [`Accordion`]: a label plus optional expandable content.
#[derive(Clone)]
pub struct AccordionItem {
    /// Header label text.
    pub label: String,
    /// Whether the item starts expanded.
    pub expanded: bool,
    /// Content shown when expanded, if any.
    pub content: Option<Handle<UiNode>>,
}

/// Builder for an [`Accordion`] component.
#[derive(Clone)]
pub struct Accordion {
    items: Vec<AccordionItem>,
    allow_multiple: bool,
    on_toggle: Option<Rc<ToggleCallback>>,
    margin: Thickness,
}

impl Default for Accordion {
    fn default() -> Self {
        Self::new()
    }
}

impl Accordion {
    /// Creates a new accordion builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            allow_multiple: false,
            on_toggle: None,
            margin: Thickness::ZERO,
        }
    }

    /// Appends a header-only item (no expandable content).
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(AccordionItem {
            label: label.into(),
            expanded: false,
            content: None,
        });
        self
    }

    /// Appends an item with expandable content.
    pub fn item_with_content(mut self, label: impl Into<String>, content: Handle<UiNode>) -> Self {
        self.items.push(AccordionItem {
            label: label.into(),
            expanded: false,
            content: Some(content),
        });
        self
    }

    /// Appends an item with expandable content that starts expanded.
    pub fn item_with_content_expanded(
        mut self,
        label: impl Into<String>,
        content: Handle<UiNode>,
    ) -> Self {
        self.items.push(AccordionItem {
            label: label.into(),
            expanded: true,
            content: Some(content),
        });
        self
    }

    /// Sets whether multiple items may be open at once (default false).
    pub fn allow_multiple(mut self, allow_multiple: bool) -> Self {
        self.allow_multiple = allow_multiple;
        self
    }

    /// Sets the callback invoked with `(index, expanded)` on every toggle.
    pub fn on_toggle<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize, bool) + 'static,
    {
        self.on_toggle = Some(Rc::new(callback));
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the accordion, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();
        let mut expander_handles: Vec<Handle<UiNode>> = Vec::new();

        for item in &self.items {
            let header: Handle<UiNode> = {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                TextBuilder::new(WidgetBuilder::new())
                    .with_text(&item.label)
                    .with_font(font)
                    .build(&mut ctx)
                    .to_base()
            };

            let expander = {
                let mut ctx = cx.ctx();
                let checkbox = fluent_expander_checkbox(&mut ctx, &theme, item.expanded);
                let mut builder = ExpanderBuilder::new(WidgetBuilder::new())
                    .with_header(header)
                    .with_checkbox(checkbox);
                if item.expanded {
                    builder = builder.with_expanded(true);
                }
                if let Some(content) = item.content {
                    builder = builder.with_content(content);
                }
                builder.build(&mut ctx).to_base()
            };
            expander_handles.push(expander);
        }

        let panel = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_accordion")
                    .with_margin(to_fyrox_thickness(self.margin))
                    .with_children(expander_handles.clone()),
            )
            .with_orientation(Orientation::Vertical)
            .build(&mut ctx)
            .to_base()
        };

        // Register an item handler for every expander so per-index toggles can
        // be dispatched and exclusive-open enforcement applied.
        let on_toggle = self.on_toggle;
        for (index, handle) in expander_handles.iter().enumerate() {
            let siblings: Vec<Handle<UiNode>> = expander_handles
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, h)| *h)
                .collect();
            let handlers = AccordionItemHandlers {
                index,
                allow_multiple: self.allow_multiple,
                siblings,
                on_toggle: on_toggle.clone(),
            };
            let component = Component {
                handle: *handle,
                kind: ComponentKind::AccordionItem(handlers),
            };
            cx.register(&component);
        }

        Component {
            handle: panel,
            kind: ComponentKind::Static,
        }
    }
}

/// A handle to a built accordion container.
pub type AccordionHandle = Handle<UiNode>;
