//! Select component: a read-only dropdown backed by fyrox's `DropdownList`.

use std::rc::Rc;

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::gui::brush::Brush;
use fyrox::gui::dropdown_list::{DropdownList, DropdownListBuilder, DropdownListMessage};
use fyrox::gui::message::{KeyCode, MessageDirection, UiMessage};
use fyrox::gui::list_view::{ListView, ListViewMessage};
use fyrox::gui::popup::Popup;
use fyrox::gui::vector_image::{Primitive, VectorImage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_color;

type ChangeCallback = dyn Fn(&mut UserInterface, usize);

/// Event handlers of a Select component.
#[derive(Clone)]
pub struct SelectHandlers {
    on_change: Option<Rc<ChangeCallback>>,
    /// The inner dropdown list that receives programmatic commands.
    command_target: Handle<UiNode>,
    /// Muted text shown when nothing is selected (if any).
    placeholder: Option<Handle<UiNode>>,
}

impl SelectHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(selection) = message.data::<DropdownListMessage>() {
            // Forward ToWidget commands aimed at the outer chrome to the
            // inner dropdown list (skips the forwarded copy itself).
            if message.direction() == MessageDirection::ToWidget
                && message.destination() != self.command_target
            {
                ui.send(self.command_target, selection.clone());
                return;
            }
            if message.direction() != MessageDirection::FromWidget {
                return;
            }
            // Flip the placeholder with the selection state.
            if let (Some(placeholder), Some(DropdownListMessage::Selection(selected))) =
                (&self.placeholder, message.data::<DropdownListMessage>())
            {
                ui.send(*placeholder, WidgetMessage::Visibility(selected.is_none()));
            }
            if let Some(on_change) = &self.on_change {
                if let Some(DropdownListMessage::Selection(Some(index))) =
                    message.data::<DropdownListMessage>()
                {
                    on_change(ui, *index);
                }
            }
        }
    }
}

/// Builder for a [`Select`] component (read-only dropdown).
#[derive(Clone)]
pub struct Select {
    items: Vec<String>,
    selected: Option<usize>,
    placeholder: String,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}

impl Select {
    /// Creates a new select builder.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            placeholder: String::new(),
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the selectable items.
    pub fn items(mut self, items: Vec<impl Into<String>>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the initially selected item index (clamped to the item count).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = (index < self.items.len()).then_some(index);
        self
    }

    /// Sets the placeholder text shown when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when a selection is made (passes the index).
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the select, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();

        let inner = {
            let mut ctx = cx.ctx();

            let mut item_nodes = Vec::new();
            for item in &self.items {
                let font = ctx.default_font();
                let text = fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
                    .with_text(item)
                    .with_font(font)
                    .build(&mut ctx);
                item_nodes.push(text.to_base());
            }

            let mut builder =
                DropdownListBuilder::new(WidgetBuilder::new().with_name("raikou_select_inner"))
                    .with_items(item_nodes);

            if let Some(selected) = self.selected {
                builder = builder.with_selected(selected);
            }

            builder.build(&mut ctx).to_base()
        };

        // Fluent restyle: thin stroked foreground chevron instead of the
        // stock filled accent triangle (Avalonia ComboBox glyph).
        fluent_dropdown_arrow(&mut cx.ctx(), &theme, inner);

        // Nothing selected: plant a muted placeholder text into the inner
        // dropdown's content grid; handlers flip its visibility with the
        // selection state. An empty placeholder string means none.
        let placeholder = if self.selected.is_none() && !self.placeholder.is_empty() {
            let main_grid = {
                use fyrox::graph::SceneGraph;
                cx.ui()
                    .try_get_of_type::<DropdownList>(inner)
                    .ok()
                    .filter(|dd| dd.current.is_none())
                    .map(|dd| *dd.main_grid)
            };
            main_grid.map(|grid| {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                let fallback_muted = raikou_core::Color::new(0.45, 0.45, 0.45, 1.0);
                let muted = Brush::Solid(to_fyrox_color(
                    theme.color("text.muted").unwrap_or(fallback_muted),
                ));
                let text: Handle<UiNode> = fyrox::gui::text::TextBuilder::new(
                    WidgetBuilder::new()
                        .on_row(0)
                        .on_column(0)
                        .with_vertical_alignment(fyrox::gui::VerticalAlignment::Center)
                        .with_foreground(muted.into()),
                )
                .with_text(&self.placeholder)
                .with_font(font)
                // Glyph-level centering: keeps the placeholder vertically
                // centered even when the field is stretched tall (the widget
                // alone may be arranged at the top of its grid cell).
                .with_vertical_text_alignment(fyrox::gui::VerticalAlignment::Center)
                .build(&mut ctx)
                .to_base();
                ctx.link(text, grid);
                text
            })
        } else {
            None
        };

        let handle = {
            let mut ctx = cx.ctx();
            crate::field::field_chrome(
                &mut ctx,
                &theme,
                inner,
                crate::field::FIELD_MIN_HEIGHT,
                self.margin,
            )
        };

        let component = Component {
            handle,
            kind: ComponentKind::Select(SelectHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
                placeholder,
            }),
        };
        cx.register(&component);
        // The inner dropdown list emits the FromWidget messages; register it
        // too so exact-destination dispatch finds the handlers.
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::Select(SelectHandlers {
                on_change: self.on_change.clone(),
                command_target: inner,
                placeholder,
            }),
        });
        // Arrow-key cycling of the open flyout needs a global watcher: with
        // the popup open, focus lives inside it and presses never land on a
        // registered handle.
        cx.register_global(&Component {
            handle: inner,
            kind: ComponentKind::SelectNav(SelectNavHandlers { target: inner }),
        });
        component
    }
}

/// Swaps the stock filled triangle of a fyrox `DropdownList` for Fluent's
/// thin stroked chevron glyph in the theme's secondary text color (the
/// Avalonia ComboBox drop-down marker). Shared with the combobox component.
pub(crate) fn fluent_dropdown_arrow(
    ctx: &mut fyrox::gui::BuildContext,
    theme: &raikou_style::Theme,
    dropdown: Handle<UiNode>,
) {
    let main_grid = ctx[dropdown].cast::<DropdownList>().map(|dd| *dd.main_grid);
    let Some(main_grid) = main_grid else {
        return;
    };

    let fallback = raikou_core::Color::new(0.35, 0.35, 0.39, 1.0);
    let color = to_fyrox_color(theme.color("text.secondary").unwrap_or(fallback));
    let thickness = 1.5;
    let chevron = vec![
        Primitive::Line {
            begin: Vector2::new(1.5, 1.75),
            end: Vector2::new(5.0, 5.25),
            thickness,
        },
        Primitive::Line {
            begin: Vector2::new(5.0, 5.25),
            end: Vector2::new(8.5, 1.75),
            thickness,
        },
    ];

    let children = match ctx.try_get_node(main_grid.to_base()) {
        Ok(node) => node.children().to_vec(),
        Err(_) => return,
    };
    for child in children {
        if child.is_none() {
            continue;
        }
        if let Some(arrow) = ctx[child].cast_mut::<VectorImage>() {
            arrow
                .primitives
                .set_value_and_mark_modified(chevron.clone());
            arrow.widget.set_width(10.0);
            arrow.widget.set_height(7.0);
            arrow
                .widget
                .foreground
                .set_value_and_mark_modified(Brush::Solid(color).into());
        }
    }
}

/// A handle to a built select.
pub type SelectHandle = Handle<UiNode>;

/// Global key watcher that cycles an open dropdown list with the arrow keys.
///
/// When the flyout is open, fyrox moves keyboard focus into the popup's list,
/// so arrow presses are aimed at nodes the exact-path registry never sees.
/// The watcher only reacts while its own dropdown's flyout is open and the
/// press landed inside that flyout; `close_on_selection` stays off in raikou
/// builds, so cycling commits the highlighted item without dismissing it
/// (Avalonia highlights first and commits on Enter — a documented deviation).
#[derive(Clone)]
pub struct SelectNavHandlers {
    /// The inner `DropdownList` this watcher serves.
    pub(crate) target: Handle<UiNode>,
}

impl SelectNavHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        use fyrox::graph::SceneGraph;

        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(&WidgetMessage::KeyDown(key)) = message.data::<WidgetMessage>() else {
            return;
        };
        if !matches!(key, KeyCode::ArrowDown | KeyCode::ArrowUp) {
            return;
        }
        let (popup, list_view, count, current) = {
            let Ok(dd) = ui.try_get_of_type::<DropdownList>(self.target) else {
                return;
            };
            let popup: Handle<UiNode> = (*dd.popup).to_base();
            let list_view: Handle<UiNode> = (*dd.list_view).to_base();
            let Ok(list) = ui.try_get_of_type::<ListView>(list_view) else {
                return;
            };
            (
                popup,
                list_view,
                (*list.items).len(),
                list.selection.first().copied(),
            )
        };
        let Ok(p) = ui.try_get_of_type::<Popup>(popup) else {
            return;
        };
        if !*p.is_open {
            return;
        }
        if !crate::component::is_in_subtree(ui, message.destination(), popup) {
            return;
        }
        if count == 0 {
            return;
        }
        let next = match (key, current) {
            (KeyCode::ArrowDown, Some(i)) => (i + 1).min(count - 1),
            (KeyCode::ArrowDown, None) => 0,
            (KeyCode::ArrowUp, Some(i)) => i.saturating_sub(1),
            (KeyCode::ArrowUp, None) => count - 1,
            _ => return,
        };
        // Commit through the list view only: fyrox mirrors the change back
        // as a FromWidget `DropdownListMessage::Selection`, and the exact-path
        // `SelectHandlers` already reports that (firing here too would count
        // every cycle twice).
        ui.send(list_view, ListViewMessage::Selection(vec![next]));
    }
}
