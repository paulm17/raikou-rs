//! Radio components.
//!
//! fyrox has no dedicated radio button, so a [`Radio`] is built from a
//! `CheckBox` and a [`RadioGroup`] enforces exclusive selection across a set
//! of options.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::check_box::{CheckBoxBuilder, CheckBoxMessage};
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ChangeCallback = dyn Fn(&mut UserInterface, bool);

/// Event handlers of a single Radio option.
#[derive(Clone)]
pub struct RadioHandlers {
    /// Invoked with the selection state of this option.
    pub on_change: Option<Rc<ChangeCallback>>,
}

impl RadioHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(CheckBoxMessage::Check(state)) = message.data::<CheckBoxMessage>() {
            if let Some(callback) = &self.on_change {
                callback(ui, state.unwrap_or(false));
            }
        }
    }
}

/// Builder for a single [`Radio`] option.
#[derive(Clone)]
pub struct Radio {
    label: String,
    selected: bool,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Radio {
    fn default() -> Self {
        Self::new()
    }
}

impl Radio {
    /// Creates a new radio option builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            selected: false,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the radio label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Sets the initial selection state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when this option's selection changes.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, bool) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the radio option, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let label_handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
                .with_text(&self.label)
                .with_font(font)
                .build(&mut ctx)
                .to_base()
        };

        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_radio")
            .with_margin(to_fyrox_thickness(self.margin));

        let handle = {
            let mut ctx = cx.ctx();
            CheckBoxBuilder::new(widget_builder)
                .checked(Some(self.selected))
                .with_content(label_handle)
                .build(&mut ctx)
                .to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::Radio(RadioHandlers {
                on_change: self.on_change,
            }),
        };
        cx.register(&component);
        component
    }
}

type GroupCallback = dyn Fn(&mut UserInterface, usize);

/// Event handlers of a [`RadioGroup`].
#[derive(Clone)]
pub struct RadioGroupHandlers {
    /// Invoked with the index of the newly selected option.
    pub on_change: Option<Rc<GroupCallback>>,
}

impl RadioGroupHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, _ui: &mut UserInterface, _message: &UiMessage) {}
}

/// Handlers for one option within a [`RadioGroup`]: unchecks its siblings and
/// fires the group callback when it becomes selected.
#[derive(Clone)]
pub struct RadioGroupItemHandlers {
    /// Handles of the sibling options (all except this one).
    pub siblings: Vec<Handle<UiNode>>,
    /// Index of this option within the group.
    pub index: usize,
    /// Optional callback invoked with this option's index when selected.
    pub on_change: Option<Rc<GroupCallback>>,
}

impl RadioGroupItemHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(CheckBoxMessage::Check(Some(true))) = message.data::<CheckBoxMessage>() {
            for sibling in &self.siblings {
                ui.send(*sibling, CheckBoxMessage::Check(Some(false)));
            }
            if let Some(callback) = &self.on_change {
                callback(ui, self.index);
            }
        }
    }
}

/// Builder for a [`RadioGroup`]: an exclusive set of radio options.
///
/// ```rust,ignore
/// let group = RadioGroup::new()
///     .options(&["Option A", "Option B", "Option C"])
///     .on_change(|ui, index| println!("selected: {index}"))
///     .build(&mut cx);
/// ```
#[derive(Clone)]
pub struct RadioGroup {
    options: Vec<String>,
    selected: usize,
    spacing: f32,
    on_change: Option<Rc<GroupCallback>>,
    margin: Thickness,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioGroup {
    /// Creates a new radio group builder.
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: 0,
            spacing: 8.0,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the option labels, in order.
    pub fn options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the initially selected option index.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the vertical spacing between options.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked with the index of the selected option.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the radio group and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let selected = self.selected.min(self.options.len().saturating_sub(1));

        let mut handles: Vec<Handle<UiNode>> = Vec::new();

        for (i, label) in self.options.iter().enumerate() {
            let label_handle: Handle<UiNode> = {
                let mut ctx = cx.ctx();
                let font = ctx.default_font();
                fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
                    .with_text(label)
                    .with_font(font)
                    .build(&mut ctx)
                    .to_base()
            };

            let is_checked = i == selected;
            let mut widget_builder = WidgetBuilder::new().with_name("raikou_radio_option");
            if i != self.options.len() - 1 {
                widget_builder = widget_builder.with_margin(to_fyrox_thickness(
                    Thickness::new(0.0, 0.0, 0.0, self.spacing),
                ));
            }

            let handle = {
                let mut ctx = cx.ctx();
                CheckBoxBuilder::new(widget_builder)
                    .checked(Some(is_checked))
                    .with_content(label_handle)
                    .build(&mut ctx)
                    .to_base()
            };

            handles.push(handle);
        }

        let panel_wb = WidgetBuilder::new()
            .with_name("raikou_radio_group")
            .with_margin(to_fyrox_thickness(self.margin))
            .with_children(handles.clone());

        let handle = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(panel_wb).build(&mut ctx).to_base()
        };

        // Register exclusive handlers for every option: when one becomes
        // selected it unchecks its siblings and fires the group callback.
        for (i, option_handle) in handles.iter().enumerate() {
            let siblings: Vec<Handle<UiNode>> = handles
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, h)| *h)
                .collect();
            let component = Component {
                handle: *option_handle,
                kind: ComponentKind::RadioGroupItem(RadioGroupItemHandlers {
                    siblings,
                    index: i,
                    on_change: self.on_change.clone(),
                }),
            };
            cx.register(&component);
        }

        Component {
            handle,
            kind: ComponentKind::RadioGroup(RadioGroupHandlers {
                on_change: self.on_change,
            }),
        }
    }
}

/// A handle to a built radio option.
pub type RadioHandle = Handle<UiNode>;

/// A handle to a built radio group.
pub type RadioGroupHandle = Handle<UiNode>;
