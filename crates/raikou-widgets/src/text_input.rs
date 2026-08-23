//! The TextInput component (single-line text field).
//!
//! Maps onto fyrox's `TextBoxBuilder` and reports text changes through an
//! `on_change` handler. Uses `TextCommitMode::Immediate` so the callback fires
//! on every edit. Adds Avalonia behaviors fyrox lacks: Ctrl+Z / Ctrl+Y
//! undo-redo history, double-click word selection, and a Fluent focus ring
//! that accents the field chrome while any node of the field holds focus.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::graph::SceneGraph;
use fyrox::gui::border::Border;
use fyrox::gui::brush::Brush;
use fyrox::gui::message::{KeyCode, MessageDirection, MouseButton, UiMessage};
use fyrox::gui::text::TextMessage;
use fyrox::gui::text_box::{EmptyTextPlaceholder, TextBox, TextBoxBuilder, TextCommitMode};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::{Color, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{is_in_subtree, Component, ComponentKind};
use crate::convert::to_fyrox_color;

type ChangeCallback = dyn Fn(&mut UserInterface, &str);

/// Undo/redo bookkeeping shared between the handler clones. Snapshots are
/// plain strings (caret/selection are not restored); echoes of our own
/// undo/redo commands are recognized through `suppress_echo` so they do not
/// push new history entries.
#[derive(Default)]
pub(crate) struct TextHistory {
    undo: Vec<String>,
    redo: Vec<String>,
    last_text: String,
    suppress_echo: Option<String>,
}

const HISTORY_LIMIT: usize = 100;

/// Double-click detection window and slop, mirroring common platform
/// defaults (Avalonia uses the system values; fyrox has none).
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
const DOUBLE_CLICK_SLOP: f32 = 4.0;

/// Event handlers of a TextInput component.
#[derive(Clone)]
pub struct TextInputHandlers {
    /// Invoked with the current text whenever it changes.
    pub on_change: Option<Rc<ChangeCallback>>,
    /// The inner text box that receives programmatic commands.
    pub command_target: Handle<UiNode>,
    /// Shared undo/redo state.
    pub(crate) history: Rc<RefCell<TextHistory>>,
}

impl TextInputHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        // Undo/redo keystrokes aimed at the field. fyrox's text box has no
        // undo stack, so these would otherwise be dead keys.
        if message.direction() == MessageDirection::ToWidget
            && message.destination() == self.command_target
        {
            if let Some(WidgetMessage::KeyDown(key)) = message.data::<WidgetMessage>() {
                let modifiers = ui.keyboard_modifiers();
                if modifiers.control {
                    let redo =
                        *key == KeyCode::KeyY || (*key == KeyCode::KeyZ && modifiers.shift);
                    let undo = *key == KeyCode::KeyZ && !modifiers.shift;
                    if (undo || redo) && self.apply_history(ui, redo) {
                        return;
                    }
                }
            }
        }

        if let Some(text) = message.data::<TextMessage>() {
            // Forward ToWidget commands aimed at the outer chrome to the
            // inner text box (skips the forwarded copy itself).
            if message.direction() == MessageDirection::ToWidget
                && message.destination() != self.command_target
            {
                ui.send(self.command_target, text.clone());
                return;
            }
            if message.direction() != MessageDirection::FromWidget {
                return;
            }
            if let Some(TextMessage::Text(text)) = message.data::<TextMessage>() {
                self.observe_edit(text);
                if let Some(callback) = &self.on_change {
                    callback(ui, text);
                }
            }
        }
    }

    /// Updates the history with an observed edit; undoes/redos echo back the
    /// exact snapshot they produced and skip the push.
    fn observe_edit(&self, text: &str) {
        let mut history = self.history.borrow_mut();
        if history.suppress_echo.as_deref() == Some(text) {
            history.suppress_echo = None;
            history.last_text = text.to_owned();
            return;
        }
        if history.last_text != text {
            let previous = history.last_text.clone();
            history.undo.push(previous);
            if history.undo.len() > HISTORY_LIMIT {
                history.undo.remove(0);
            }
            history.redo.clear();
            history.last_text = text.to_owned();
        }
    }

    /// Pops the undo or redo stack and sends the restored text to the field.
    /// Returns `false` when there is nothing to restore.
    fn apply_history(&self, ui: &mut UserInterface, redo: bool) -> bool {
        let next = {
            let mut history = self.history.borrow_mut();
            let next = if redo {
                history.redo.pop()
            } else {
                history.undo.pop()
            };
            let Some(next) = next else {
                return false;
            };
            let previous = history.last_text.clone();
            if redo {
                history.undo.push(previous);
            } else {
                history.redo.push(previous);
            }
            history.suppress_echo = Some(next.clone());
            next
        };
        ui.send(self.command_target, TextMessage::Text(next));
        true
    }
}

/// Global watcher that accents a field's chrome border while keyboard focus
/// sits anywhere inside its subtree (fyrox routes focus messages to the
/// deepest widget under the cursor, not to the component root).
#[derive(Clone)]
pub struct FocusRingHandlers {
    /// Root of the watched subtree (the field chrome).
    pub(crate) target_subtree: Handle<UiNode>,
    /// The chrome border whose stroke is swapped.
    pub(crate) border: Handle<UiNode>,
    pub(crate) normal: Brush,
    pub(crate) accent: Brush,
    pub(crate) active: Rc<Cell<bool>>,
}

impl FocusRingHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(WidgetMessage::Focus) = message.data::<WidgetMessage>() else {
            return;
        };
        if is_in_subtree(ui, message.destination(), self.target_subtree) {
            self.set_border(ui, self.accent.clone());
            self.active.set(true);
        } else if self.active.replace(false) {
            self.set_border(ui, self.normal.clone());
        }
    }

    fn set_border(&self, ui: &mut UserInterface, brush: Brush) {
        if let Ok(border) = ui.try_get_mut_of_type::<Border>(self.border) {
            border
                .widget
                .foreground
                .set_value_and_mark_modified(brush.into());
        }
    }
}

/// Global watcher implementing double-click word selection for a text box.
/// Lives outside the exact-path registry because presses are aimed at
/// whichever deep child sits under the cursor.
#[derive(Clone)]
pub struct WordSelectHandlers {
    /// The inner text box.
    pub(crate) target: Handle<UiNode>,
    pub(crate) last_click: Rc<RefCell<Option<(Instant, Vector2<f32>)>>>,
}

impl WordSelectHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::ToWidget {
            return;
        }
        let Some(WidgetMessage::MouseDown { button, pos }) = message.data::<WidgetMessage>() else {
            return;
        };
        if *button != MouseButton::Left {
            return;
        }
        if !is_in_subtree(ui, message.destination(), self.target) {
            return;
        }
        let now = Instant::now();
        let is_double = self
            .last_click
            .borrow()
            .map(|(at, at_position)| {
                now.duration_since(at) < DOUBLE_CLICK_WINDOW
                    && (*pos - at_position).norm() < DOUBLE_CLICK_SLOP
            })
            .unwrap_or(false);
        if is_double {
            *self.last_click.borrow_mut() = None;
            select_word_at_caret(ui, self.target);
        } else {
            *self.last_click.borrow_mut() = Some((now, *pos));
        }
    }
}

/// Replicates fyrox's private `TextBox::select_word`: expands the selection
/// from the caret over the run of characters sharing the anchor character's
/// whitespace classification.
fn select_word_at_caret(ui: &mut UserInterface, target: Handle<UiNode>) {
    use fyrox::gui::text_box::SelectionRange;

    let Ok(text_box) = ui.try_get_of_type::<TextBox>(target) else {
        return;
    };
    let caret = *text_box.caret_position;
    let Some(index) = text_box.position_to_char_index_clamped(caret) else {
        return;
    };
    let text: Vec<char> = text_box.text().chars().collect();
    let _ = text_box; // release the immutable borrow before the mutable one

    let Some(&anchor) = text.get(index) else {
        return;
    };    let search_whitespace = !anchor.is_whitespace();

    let mut left_index = index;
    while left_index > 0 {
        let is_whitespace = text[left_index].is_whitespace();
        if search_whitespace && is_whitespace || !search_whitespace && !is_whitespace {
            left_index += 1;
            break;
        }
        left_index = left_index.saturating_sub(1);
    }

    let mut right_index = index;
    while right_index < text.len() {
        let is_whitespace = text[right_index].is_whitespace();
        if search_whitespace && is_whitespace || !search_whitespace && !is_whitespace {
            break;
        }
        right_index += 1;
    }

    let Ok(text_box) = ui.try_get_mut_of_type::<TextBox>(target) else {
        return;
    };
    if let (Some(left), Some(right)) = (
        text_box.char_index_to_position(left_index),
        text_box.char_index_to_position(right_index),
    ) {
        text_box
            .selection_range
            .set_value_and_mark_modified(Some(SelectionRange {
                begin: left,
                end: right,
            }));
    }
}

/// Builder for a [`TextInput`] component.
#[derive(Clone)]
pub struct TextInput {
    text: String,
    placeholder: String,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a new text input builder.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the initial text value.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Sets the placeholder text shown when the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the text changes.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, &str) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the text input, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let theme = cx.theme().clone();

        // Inner text box: Avalonia TextControl padding (10, 6, 6, 5).
        let inner = {
            let widget_builder = WidgetBuilder::new().with_name("raikou_text_input_inner");

            let placeholder = self.placeholder.clone();
            let mut ctx = cx.ctx();
            let mut builder = TextBoxBuilder::new(widget_builder)
                .with_text(&self.text)
                .with_text_commit_mode(TextCommitMode::Immediate)
                .with_padding(fyrox::gui::Thickness {
                    left: 10.0,
                    top: 6.0,
                    right: 6.0,
                    bottom: 5.0,
                });
            if !placeholder.is_empty() {
                // Fluent placeholder gray from the theme. The native
                // `EmptyTextPlaceholder::Text` path styles itself with fyrox's
                // BRUSH_LIGHTER style property, which the bridge maps to a
                // near-invisible hover tint — hence an explicit widget here.
                let muted = theme.color("text.muted").unwrap_or(raikou_core::Color::new(
                    0.63, 0.62, 0.61, 1.0,
                ));
                let node = fyrox::gui::text::TextBuilder::new(
                    WidgetBuilder::new()
                        .with_visibility(self.text.is_empty())
                        .with_foreground(
                            fyrox::gui::brush::Brush::Solid(to_fyrox_color(muted)).into(),
                        ),
                )
                .with_text(&placeholder)
                .with_vertical_text_alignment(fyrox::gui::VerticalAlignment::Center)
                .build(&mut ctx)
                .to_base();
                builder =
                    builder.with_empty_text_placeholder(EmptyTextPlaceholder::Widget(node));
            }
            builder.build(&mut ctx).to_base()
        };

        // Outer Fluent chrome (rounded border + min height).
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

        // Focus ring brushes: the chrome's resting stroke plus the theme
        // accent used while the field holds focus (Avalonia TextControl
        // FocusBorderBrush analogue).
        let normal_stroke = theme
            .color("border.default")
            .unwrap_or(Color::new(0.0, 0.0, 0.0, 0.4));
        let accent = theme
            .color("accent.solid")
            .unwrap_or(Color::new(0.13, 0.39, 0.94, 1.0));

        let handlers = TextInputHandlers {
            on_change: self.on_change.clone(),
            command_target: inner,
            history: Rc::new(RefCell::new(TextHistory {
                last_text: self.text.clone(),
                ..Default::default()
            })),
        };

        let component = Component {
            handle,
            kind: ComponentKind::TextInput(handlers.clone()),
        };
        cx.register(&component);
        // The inner text box emits the FromWidget messages; register it too so
        // exact-destination dispatch finds the handlers.
        cx.register(&Component {
            handle: inner,
            kind: ComponentKind::TextInput(handlers),
        });
        // Global watchers: focus routing and presses land on deep children,
        // never on the chrome handle itself.
        cx.register_global(&Component {
            handle,
            kind: ComponentKind::FocusRing(FocusRingHandlers {
                target_subtree: handle,
                border: handle,
                normal: Brush::Solid(to_fyrox_color(normal_stroke)),
                accent: Brush::Solid(to_fyrox_color(accent)),
                active: Rc::new(Cell::new(false)),
            }),
        });
        cx.register_global(&Component {
            handle: inner,
            kind: ComponentKind::WordSelect(WordSelectHandlers {
                target: inner,
                last_click: Rc::new(RefCell::new(None)),
            }),
        });
        component
    }
}

/// A handle to a built text input, returned for convenience.
pub type TextInputHandle = Handle<UiNode>;
