//! The Checkbox component.
//!
//! Maps onto fyrox's `CheckBoxBuilder` and reports state changes through a
//! per-component `on_change` handler. Optionally runs in Avalonia's
//! three-state mode where pointer toggles cycle unchecked → indeterminate →
//! checked (fyrox natively flips a two-state bool and cannot represent the
//! cycle, so the handler corrects the native state after each click).

use std::cell::Cell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::check_box::{CheckBoxBuilder, CheckBoxMessage};
use fyrox::gui::message::{MessageDirection, MouseButton, UiMessage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{is_in_subtree, Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, bool);
type TriStateChangeCallback = dyn Fn(&mut UserInterface, Option<bool>);

/// Event handlers of a Checkbox component.
#[derive(Clone)]
pub struct CheckboxHandlers {
    /// Invoked with the checked state whenever the box is toggled
    /// (`false` when indeterminate). Unused when [`Self::on_change_state`]
    /// is set.
    pub on_change: Option<Rc<ChangeCallback>>,
    /// Invoked with the tri-state value whenever the box is toggled.
    pub on_change_state: Option<Rc<TriStateChangeCallback>>,
    /// Handle of the checkbox widget (corrections are sent to it).
    pub(crate) handle: Handle<UiNode>,
    /// Whether pointer clicks cycle through the indeterminate state like
    /// Avalonia's `IsThreeState` check box.
    pub(crate) three_state: bool,
    /// Last known state, kept to drive the Avalonia toggle cycle.
    pub(crate) last_state: Rc<Cell<Option<bool>>>,
    /// When set, the next `Check` echo with this value is our own correction
    /// coming back — accept it silently instead of correcting again.
    pub(crate) suppress: Rc<Cell<Option<Option<bool>>>>,
    /// Set while a pointer press inside the checkbox subtree is "in flight";
    /// only click-initiated echoes participate in the cycle, programmatic
    /// sets land verbatim.
    pub(crate) armed: Rc<Cell<bool>>,
}

/// The next state after a pointer click in Avalonia's three-state cycle:
/// unchecked → indeterminate → checked → unchecked.
fn avalonia_cycle(current: Option<bool>) -> Option<bool> {
    match current {
        None => Some(true),
        Some(false) => None,
        Some(true) => Some(false),
    }
}

impl CheckboxHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        // Arm the cycle when a pointer release lands anywhere in the
        // checkbox subtree (fyrox toggles on MouseUp, and releases target
        // whatever child sits under the cursor).
        if let Some(WidgetMessage::MouseUp { button, .. }) = message.data::<WidgetMessage>() {
            if message.direction() == MessageDirection::ToWidget
                && *button == MouseButton::Left
                && is_in_subtree(ui, message.destination(), self.handle)
            {
                self.armed.set(true);
            }
            return;
        }

        let Some(CheckBoxMessage::Check(state)) = message.data::<CheckBoxMessage>() else {
            return;
        };
        // Only echoes for THIS box drive the bookkeeping: the registry-global
        // clone also sees sibling checkboxes' echoes.
        if !is_in_subtree(ui, message.destination(), self.handle) {
            return;
        }
        // Only FromWidget echoes drive the bookkeeping: they arrive both for
        // clicks and for programmatic sets, while ToWidget commands include
        // fyrox's own internal flip which must not touch our cycle state.
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        // A correction we sent came back: settle on it.
        if let Some(expected) = self.suppress.get() {
            if *state == expected {
                self.suppress.set(None);
                self.last_state.set(*state);
                self.fire(ui, *state);
            }
            return;
        }
        // Click echo: steer the native two-state flip onto Avalonia's
        // unchecked → indeterminate → checked cycle. Echoes without a
        // preceding pointer press are programmatic sets and land verbatim.
        if self.three_state && self.armed.replace(false) {
            let want = avalonia_cycle(self.last_state.get());
            if *state != want {
                self.suppress.set(Some(want));
                ui.send(self.handle, CheckBoxMessage::Check(want));
                return;
            }
        }
        self.last_state.set(*state);
        self.fire(ui, *state);
    }

    fn fire(&self, ui: &mut UserInterface, state: Option<bool>) {
        if let Some(callback) = &self.on_change_state {
            callback(ui, state);
            return;
        }
        if let Some(callback) = &self.on_change {
            callback(ui, state.unwrap_or(false));
        }
    }
}

/// Builder for a [`Checkbox`] component.
///
/// ```rust,ignore
/// let checkbox = Checkbox::new()
///     .checked(true)
///     .on_change(|ui, checked| println!("checked: {checked}"))
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct Checkbox {
    label: String,
    checked: bool,
    initial_state: Option<Option<bool>>,
    three_state: bool,
    on_change: Option<Rc<ChangeCallback>>,
    on_change_state: Option<Rc<TriStateChangeCallback>>,
    margin: Thickness,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Checkbox {
    /// Creates a new checkbox builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            checked: false,
            initial_state: None,
            three_state: false,
            on_change: None,
            on_change_state: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the checkbox label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Sets the initial checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self.initial_state = None;
        self
    }

    /// Sets the initial tri-state value (used for indeterminate starts).
    pub fn state(mut self, state: Option<bool>) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Enables Avalonia-style three-state cycling on pointer clicks
    /// (unchecked → indeterminate → checked). Keyboard toggles keep fyrox's
    /// native two-state flip. Pair with [`Self::on_change_state`] to observe
    /// the indeterminate value; `on_change` still fires with `false`.
    pub fn three_state(mut self, three_state: bool) -> Self {
        self.three_state = three_state;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the checkbox is toggled.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, bool) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Sets the callback invoked with the tri-state value when the checkbox
    /// is toggled. Takes precedence over `on_change` when set.
    pub fn on_change_state<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, Option<bool>) + 'static,
    {
        self.on_change_state = Some(Rc::new(callback));
        self
    }

    /// Builds the checkbox, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let initial = self.initial_state.unwrap_or(Some(self.checked));

        let label_handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            fyrox::gui::text::TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(to_fyrox_thickness(Thickness::new(0.0, 0.0, 0.0, 0.0)))
                    .with_vertical_alignment(fyrox::gui::VerticalAlignment::Center),
            )
            .with_text(&self.label)
            .with_font(font)
            .build(&mut ctx)
            .to_base()
        };

        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_checkbox")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            CheckBoxBuilder::new(widget_builder)
                .checked(initial)
                .with_content(label_handle)
                .build(&mut ctx)
                .to_base()
        };

        // The native check box lays itself out on a stretched grid row, which
        // makes it claim all available height inside constrained containers.
        // Switch the grid to auto sizing so the widget hugs its content.
        {
            use fyrox::graph::SceneGraph;
            use fyrox::gui::grid::{Column, Grid, Row};
            let ui = cx.ui();
            if let Some(grid_handle) = ui.node(handle).children().first().copied() {
                if let Ok(grid) = ui.try_get_mut_of_type::<Grid>(grid_handle) {
                    *grid.rows.borrow_mut() = vec![Row::auto()];
                    *grid.columns.borrow_mut() = vec![Column::auto(), Column::auto()];
                }
            }
        }

        // The exact-path listener owns the toggle bookkeeping; the global one
        // only arms the cycle on pointer presses aimed at deep children. Both
        // share the same state cells so either can observe the other's work.
        let last_state = Rc::new(Cell::new(initial));
        let suppress: Rc<Cell<Option<Option<bool>>>> = Rc::new(Cell::new(None));
        let armed = Rc::new(Cell::new(false));
        let make_handlers = |on_change: Option<Rc<ChangeCallback>>,
                                 on_change_state: Option<Rc<TriStateChangeCallback>>| {
            CheckboxHandlers {
                on_change,
                on_change_state,
                handle,
                three_state: self.three_state,
                last_state: last_state.clone(),
                suppress: suppress.clone(),
                armed: armed.clone(),
            }
        };

        let component = Component {
            handle,
            kind: ComponentKind::Checkbox(make_handlers(
                self.on_change,
                self.on_change_state.clone(),
            )),
        };
        cx.register(&component);
        if self.three_state {
            cx.register_global(&Component {
                handle,
                kind: ComponentKind::Checkbox(make_handlers(None, self.on_change_state)),
            });
        }
        component
    }
}

/// A handle to a built checkbox, returned for convenience.
pub type CheckboxHandle = Handle<UiNode>;
