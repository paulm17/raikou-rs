//! The Accordion component.
//!
//! Maps onto a vertical stack of fyrox `Expander` widgets. Expansion toggles
//! are reported through a per-component `on_toggle` handler. When
//! `allow_multiple` is false, opening one item collapses the others.

use std::rc::Rc;

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::check_box::{CheckBox, CheckBoxBuilder, CheckBoxMessage};
use fyrox::gui::expander::{Expander, ExpanderBuilder, ExpanderMessage};
use fyrox::gui::message::MouseButton;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::text::TextBuilder;
use fyrox::gui::vector_image::{Primitive, VectorImageBuilder};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{Orientation, UiNode, UserInterface, VerticalAlignment};
use fyrox::graph::SceneGraph;

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};
use crate::tween::{TweenJob, TweenMessage};

type ToggleCallback = dyn Fn(&mut UserInterface, usize, bool);

/// Fluent Expander headers are ~44px tall.
const FLUENT_HEADER_HEIGHT: f32 = 44.0;

/// Down-pointing chevron primitives inside a 10x10 box (expanded mark).
pub(crate) fn down_prims() -> Vec<Primitive> {
    vec![
        Primitive::Line {
            begin: Vector2::new(2.0, 3.5),
            end: Vector2::new(5.0, 6.5),
            thickness: 1.5,
        },
        Primitive::Line {
            begin: Vector2::new(5.0, 6.5),
            end: Vector2::new(8.0, 3.5),
            thickness: 1.5,
        },
    ]
}

/// Right-pointing chevron primitives inside a 10x10 box (collapsed mark).
pub(crate) fn right_prims() -> Vec<Primitive> {
    vec![
        Primitive::Line {
            begin: Vector2::new(3.5, 2.0),
            end: Vector2::new(6.5, 5.0),
            thickness: 1.5,
        },
        Primitive::Line {
            begin: Vector2::new(6.5, 5.0),
            end: Vector2::new(3.5, 8.0),
            thickness: 1.5,
        },
    ]
}

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
    let down = down_prims();
    let right = right_prims();
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
/// background (no stock button chrome). Returns the checkbox plus both mark
/// nodes so callers can animate the chevron spin.
pub(crate) fn fluent_expander_checkbox(
    ctx: &mut fyrox::gui::BuildContext,
    theme: &raikou_style::Theme,
    expanded: bool,
) -> (Handle<CheckBox>, Handle<UiNode>, Handle<UiNode>) {
    let (check_mark, uncheck_mark) = chevron_mark_nodes(ctx, theme);
    let marks = (check_mark, uncheck_mark);
    let checkbox = CheckBoxBuilder::new(
        WidgetBuilder::new().with_vertical_alignment(VerticalAlignment::Center),
    )
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
    .build(ctx);
    (checkbox, marks.0, marks.1)
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
    /// Shared tweener node that animates the chevron spin.
    pub(crate) tweener: Handle<UiNode>,
    /// The expanded-state chevron mark (down-pointing base).
    pub(crate) check_mark: Handle<UiNode>,
    /// The collapsed-state chevron mark (right-pointing base).
    pub(crate) uncheck_mark: Handle<UiNode>,
    /// Pending programmatic sibling collapses (shared). fyrox's expander
    /// reports state changes as ToWidget `Expand` messages, which is
    /// indistinguishable from a user click at the destination — this counter
    /// lets exclusive-open collapses be swallowed silently.
    pub(crate) suppress_collapses: Rc<std::cell::Cell<usize>>,
}

impl AccordionItemHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        let Some(ExpanderMessage::Expand(expanded)) = message.data::<ExpanderMessage>() else {
            return;
        };
        match message.direction() {
            // Direct FromWidget commands always report (legacy API path).
            MessageDirection::FromWidget => {}
            // fyrox emits ToWidget `Expand` both for real clicks (via the
            // embedded checkbox) and for our own sibling collapses; swallow
            // the latter.
            MessageDirection::ToWidget => {
                let pending = self.suppress_collapses.get();
                if pending > 0 {
                    self.suppress_collapses.set(pending - 1);
                    return;
                }
            }
        }
        // Spin the chevron into its new pose; each mark's primitives are
        // authored at rest, so start 90 degrees off and settle to zero.
        let job = if *expanded {
            TweenJob::ChevronSpin {
                image: self.check_mark,
                base: down_prims(),
                from_deg: -90.0,
                to_deg: 0.0,
            }
        } else {
            TweenJob::ChevronSpin {
                image: self.uncheck_mark,
                base: right_prims(),
                from_deg: 90.0,
                to_deg: 0.0,
            }
        };
        ui.send(self.tweener, TweenMessage(job));
        if !self.allow_multiple && *expanded {
            self.suppress_collapses.set(self.siblings.len());
            for sibling in &self.siblings {
                ui.send(*sibling, ExpanderMessage::Expand(false));
            }
        }
        if let Some(callback) = &self.on_toggle {
            callback(ui, self.index, *expanded);
        }
    }
}

/// Global watcher making the whole expander header a click target: any left
/// mouse-up inside an item (except on its embedded checkbox or its expanded
/// content) flips the expansion, matching Fluent Expander behavior.
#[derive(Clone)]
pub struct AccordionHeaderHit {
    /// The expander whose header is the hit target.
    pub(crate) expander: Handle<UiNode>,
    /// The item's expandable content, excluded from the hit target.
    pub(crate) content: Option<Handle<UiNode>>,
}

impl AccordionHeaderHit {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(WidgetMessage::MouseUp {
            button: MouseButton::Left,
            ..
        }) = message.data::<WidgetMessage>()
        else {
            return;
        };
        if !crate::component::is_in_subtree(ui, message.destination(), self.expander) {
            return;
        }
        // The embedded checkbox toggles natively; content clicks are text
        // selection territory, not header hits.
        let Some((checkbox, expanded)) = ui
            .try_get_of_type::<Expander>(self.expander)
            .ok()
            .map(|expander| (*expander.expander, *expander.is_expanded))
        else {
            return;
        };
        let checkbox_base = checkbox.to_base();
        if crate::component::is_in_subtree(ui, message.destination(), checkbox_base)
            || self.content.is_some_and(|content| {
                crate::component::is_in_subtree(ui, message.destination(), content)
            })
        {
            return;
        }
        ui.send(checkbox, CheckBoxMessage::Check(Some(!expanded)));
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
    /// Accent color drawn as a short vertical bar on the header's leading
    /// edge (Fluent-style section marker). `None` draws nothing.
    pub accent: Option<raikou_core::Color>,
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
            accent: None,
        });
        self
    }

    /// Appends an item with expandable content.
    pub fn item_with_content(mut self, label: impl Into<String>, content: Handle<UiNode>) -> Self {
        self.items.push(AccordionItem {
            label: label.into(),
            expanded: false,
            content: Some(content),
            accent: None,
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
            accent: None,
        });
        self
    }

    /// Appends a fully specified item (accent included).
    pub fn push_item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
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
        let mut item_marks: Vec<(Handle<UiNode>, Handle<UiNode>)> = Vec::new();
        let mut children: Vec<Handle<UiNode>> = Vec::new();

        // Hairline separators between items (Fluent Expander rhythm). Both
        // brushes are set explicitly: fyrox falls back to global defaults for
        // unset background/foreground, which would paint a visible slab.
        let divider_color = theme
            .color("border.subtle")
            .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.10));

        for (item_index, item) in self.items.iter().enumerate() {
            let item_is_last = item_index + 1 == self.items.len();
            let header: Handle<UiNode> = {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                // Fluent Expander headers are ~44px tall; the label sits on a
                // fixed-height row so the chevron centers against it.
                let text = TextBuilder::new(
                    WidgetBuilder::new()
                        .with_vertical_alignment(VerticalAlignment::Center)
                        .with_margin(fyrox::gui::Thickness {
                            left: 12.0,
                            top: 0.0,
                            right: 8.0,
                            bottom: 0.0,
                        }),
                )
                .with_text(&item.label)
                .with_font(font)
                .build(&mut ctx)
                .to_base();

                let mut row_children: Vec<Handle<UiNode>> = Vec::new();
                if let Some(accent) = item.accent {
                    let bar = fyrox::gui::border::BorderBuilder::new(
                        WidgetBuilder::new()
                            .with_width(3.0)
                            .with_height(16.0)
                            .with_vertical_alignment(VerticalAlignment::Center)
                            .with_background(Brush::Solid(to_fyrox_color(accent)).into())
                            .with_foreground(
                                Brush::Solid(fyrox::core::color::Color::TRANSPARENT).into(),
                            ),
                    )
                    .with_stroke_thickness(fyrox::gui::Thickness::zero().into())
                    .with_pad_by_corner_radius(false)
                    .build(&mut ctx);
                    row_children.push(bar.to_base());
                }
                row_children.push(text);

                let header_row: Handle<UiNode> = StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_height(FLUENT_HEADER_HEIGHT)
                        .with_children(row_children),
                )
                .with_orientation(Orientation::Horizontal)
                .build(&mut ctx)
                .to_base();
                header_row
            };

            let expander = {
                let mut ctx = cx.ctx();
                let (checkbox, check_mark, uncheck_mark) =
                    fluent_expander_checkbox(&mut ctx, &theme, item.expanded);
                // NOTE: always pass the desired state — fyrox's expander
                // builder defaults to expanded(true), so skipping this call
                // leaves collapsed items with visible content.
                let mut builder = ExpanderBuilder::new(WidgetBuilder::new())
                    .with_header(header)
                    .with_checkbox(checkbox)
                    .with_expanded(item.expanded);
                if let Some(content) = item.content {
                    builder = builder.with_content(content);
                }
                let node = builder.build(&mut ctx).to_base();
                item_marks.push((check_mark, uncheck_mark));
                node
            };
            expander_handles.push(expander);
            children.push(expander);

            if !item_is_last {
                let mut ctx = cx.ctx();
                let divider = fyrox::gui::border::BorderBuilder::new(
                    WidgetBuilder::new()
                        .with_height(1.0)
                        .with_background(Brush::Solid(to_fyrox_color(divider_color)).into())
                        .with_foreground(
                            Brush::Solid(fyrox::core::color::Color::TRANSPARENT).into(),
                        ),
                )
                .with_stroke_thickness(fyrox::gui::Thickness::zero().into())
                .with_pad_by_corner_radius(false)
                .build(&mut ctx);
                children.push(divider.to_base());
            }
        }

        let panel = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_accordion")
                    .with_margin(to_fyrox_thickness(self.margin))
                    .with_children(children),
            )
            .with_orientation(Orientation::Vertical)
            .build(&mut ctx)
            .to_base()
        };

        // Shared tweener node that animates every chevron spin.
        let tweener = crate::tween::spawn_tweener(&mut cx.ctx(), panel);

        // Register an item handler for every expander so per-index toggles can
        // be dispatched and exclusive-open enforcement applied. A global
        // watcher per item also makes the whole header a hit target.
        let on_toggle = self.on_toggle;
        let suppress_collapses = Rc::new(std::cell::Cell::new(0usize));
        for (index, handle) in expander_handles.iter().enumerate() {
            let siblings: Vec<Handle<UiNode>> = expander_handles
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, h)| *h)
                .collect();
            let (check_mark, uncheck_mark) = item_marks[index];
            let content = self.items[index].content;
            let handlers = AccordionItemHandlers {
                index,
                allow_multiple: self.allow_multiple,
                siblings,
                on_toggle: on_toggle.clone(),
                tweener,
                check_mark,
                uncheck_mark,
                suppress_collapses: suppress_collapses.clone(),
            };
            cx.register(&Component {
                handle: *handle,
                kind: ComponentKind::AccordionItem(handlers),
            });
            // Whole-header hit target: clicks anywhere on the expander except
            // the embedded checkbox (which toggles natively) flip the state.
            cx.register_global(&Component {
                handle: *handle,
                kind: ComponentKind::AccordionHeaderHit(crate::accordion::AccordionHeaderHit {
                    expander: *handle,
                    content,
                }),
            });
        }

        Component {
            handle: panel,
            kind: ComponentKind::Static,
        }
    }
}

/// A handle to a built accordion container.
pub type AccordionHandle = Handle<UiNode>;
