//! The Switch component.
//!
//! A Fluent-style toggle switch: a pill-shaped 40x20 track whose knob slides
//! between the ends, with an optional label rendered beside the track. Maps
//! onto fyrox's `ToggleButton` so state flows through `ToggleButtonMessage`.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::border::Border;
use fyrox::gui::brush::Brush;
use fyrox::gui::decorator::{Decorator, DecoratorMessage};
use fyrox::gui::message::{KeyCode, MessageDirection, UiMessage};
use fyrox::gui::toggle::{ToggleButton, ToggleButtonBuilder, ToggleButtonMessage};
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::Thickness as FyroxThickness;
use fyrox::gui::{HorizontalAlignment, UiNode, UserInterface, VerticalAlignment};
use fyrox::graph::SceneGraph;

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{is_in_subtree, Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};
use crate::tween::{spawn_tweener, TweenJob, TweenMessage};

/// Knob inset from the track edges (both themes).
const KNOB_PAD: f32 = 2.5;
/// Distance between the knob's two rest positions (40 - 14 - 2 * 2.5).
const KNOB_SPAN: f32 = 21.0;

type ChangeCallback = dyn Fn(&mut UserInterface, bool);

/// Event handlers of a Switch component.
#[derive(Clone)]
pub struct SwitchHandlers {
    /// Invoked with the new toggled state whenever the switch flips.
    pub on_change: Option<Rc<ChangeCallback>>,
    /// Track widget that receives forwarded commands.
    pub command_target: Handle<UiNode>,
    /// Sliding knob node kept in sync with the toggled state.
    pub knob: Handle<UiNode>,
    /// The full switch (stack incl. label); scopes key handling.
    pub(crate) subtree: Handle<UiNode>,
    /// Tweener node that animates the knob; `None` snaps instantly.
    pub(crate) animator: Option<Handle<UiNode>>,
    /// When set, this handler clone only reacts to Space key presses
    /// (used by the global watcher so sibling echoes are ignored).
    pub(crate) keys_only: bool,
}

impl SwitchHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        // Space toggles the switch when the event lands inside its subtree
        // (fyrox ToggleButton has no key handling of its own).
        if let Some(WidgetMessage::KeyDown(KeyCode::Space)) =
            message.data::<WidgetMessage>()
        {
            if message.direction() == MessageDirection::ToWidget
                && is_in_subtree(ui, message.destination(), self.subtree)
            {
                let flipped = ui
                    .try_get_of_type::<ToggleButton>(self.command_target)
                    .map(|track| !track.is_toggled)
                    .ok();
                if let Some(state) = flipped {
                    ui.send(self.command_target, ToggleButtonMessage::Toggled(state));
                }
            }
            return;
        }

        if self.keys_only {
            return;
        }

        if let Some(ToggleButtonMessage::Toggled(state)) = message.data::<ToggleButtonMessage>() {
            if message.direction() == MessageDirection::ToWidget {
                if message.destination() != self.command_target {
                    ui.send(self.command_target, ToggleButtonMessage::Toggled(*state));
                }
                return;
            }
            // Slide the knob to the correct side of the track; the tweener
            // interpolates the travel, otherwise snap straight to it.
            match self.animator {
                Some(tweener) => {
                    ui.send(
                        tweener,
                        TweenMessage(TweenJob::KnobSlide {
                            knob: self.knob,
                            pad: KNOB_PAD,
                            span: KNOB_SPAN,
                        }),
                    );
                }
                None => {
                    ui.send(
                        self.knob,
                        WidgetMessage::HorizontalAlignment(if *state {
                            HorizontalAlignment::Right
                        } else {
                            HorizontalAlignment::Left
                        }),
                    );
                }
            }
            if let Some(callback) = &self.on_change {
                callback(ui, *state);
            }
        }
    }
}

/// Builder for a [`Switch`] component.
#[derive(Clone)]
pub struct Switch {
    label: String,
    toggled: bool,
    on_change: Option<Rc<ChangeCallback>>,
    margin: Thickness,
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Switch {
    /// Creates a new switch builder.
    pub fn new() -> Self {
        Self {
            label: String::new(),
            toggled: false,
            on_change: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the switch label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }

    /// Sets the initial toggled state.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when the switch flips.
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, bool) + 'static,
    {
        self.on_change = Some(Rc::new(callback));
        self
    }

    /// Builds the switch, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        use fyrox::graph::SceneGraph;

        let theme = cx.theme().clone();

        // Pill-shaped interactive track.
        let track = {
            let mut ctx = cx.ctx();
            ToggleButtonBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_switch_track")
                    .with_width(40.0)
                    .with_height(20.0)
                    .with_vertical_alignment(VerticalAlignment::Center),
            )
            .with_toggled(self.toggled)
            .build(&mut ctx)
            .to_base()
        };

        {
            let ui = cx.ui();
            // Round the decorator border into a pill and give the off state a
            // subtle outline; the selected (on) brush comes from the global
            // style bridge (accent).
            if let Some(decorator_handle) = ui.node(track).children().first().copied() {
                if let Ok(border) = ui.try_get_mut_of_type::<Border>(decorator_handle) {
                    *border.corner_radius = 10.0f32.into();
                }
                if let Ok(decorator) = ui.try_get_mut_of_type::<Decorator>(decorator_handle) {
                    let off_fill = theme
                        .color("surface.panel")
                        .unwrap_or(raikou_core::Color::new(1.0, 1.0, 1.0, 1.0));
                    let off_stroke = theme
                        .color("border.default")
                        .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.4));
                    *decorator.border.stroke_thickness = FyroxThickness::uniform(1.0).into();
                    // NormalBrush both stores the brush and re-applies it as
                    // the widget background while the decorator is unselected.
                    ui.send(
                        decorator_handle,
                        DecoratorMessage::NormalBrush(
                            Brush::Solid(to_fyrox_color(off_fill)).into(),
                        ),
                    );
                    ui.send(
                        decorator_handle,
                        WidgetMessage::Foreground(Brush::Solid(to_fyrox_color(off_stroke)).into()),
                    );
                }
            }
        }

        // Knob: small white circle hugging one end of the track.
        // Fluent knobs are white in both themes (the track provides contrast),
        // with a subtle outline for definition on light tracks.
        let knob_fill = raikou_core::Color::new(1.0, 1.0, 1.0, 1.0);
        let knob: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            fyrox::gui::border::BorderBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_switch_knob")
                    .with_width(14.0)
                    .with_height(14.0)
                    .with_margin(FyroxThickness::uniform(2.5))
                    .with_horizontal_alignment(if self.toggled {
                        HorizontalAlignment::Right
                    } else {
                        HorizontalAlignment::Left
                    })
                    .with_vertical_alignment(VerticalAlignment::Center),
            )
            .with_corner_radius(7.0f32.into())
            .with_stroke_thickness(FyroxThickness::uniform(1.0f32).into())
            .build(&mut ctx)
            .to_base()
        };
        cx.ui().send(
            knob,
            WidgetMessage::Background(Brush::Solid(to_fyrox_color(knob_fill)).into()),
        );
        cx.ui().send(
            knob,
            WidgetMessage::Foreground(
                Brush::Solid(to_fyrox_color(
                    theme
                        .color("border.subtle")
                        .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.2)),
                ))
                .into(),
            ),
        );
        // Place the knob inside the decorator border so it clips to the pill.
        if let Some(decorator_handle) = cx.ui().node(track).children().first().copied() {
            cx.ctx().link(knob, decorator_handle);
        }

        // Optional label beside the track.
        let label_handle: Option<Handle<UiNode>> = if self.label.is_empty() {
            None
        } else {
            let mut ctx = cx.ctx();
            let font = ctx.default_font();
            let fg = theme
                .color("text.primary")
                .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 1.0));
            let text = fyrox::gui::text::TextBuilder::new(
                WidgetBuilder::new()
                    .with_margin(to_fyrox_thickness(Thickness::new(8.0, 0.0, 0.0, 0.0)))
                    .with_vertical_alignment(VerticalAlignment::Center),
            )
            .with_text(&self.label)
            .with_font(font)
            .build(&mut ctx)
            .to_base();
            cx.ui().send(
                text,
                WidgetMessage::Foreground(Brush::Solid(to_fyrox_color(fg)).into()),
            );
            Some(text)
        };

        let outer: Handle<UiNode> = match label_handle {
            Some(label) => {
                use fyrox::gui::stack_panel::StackPanelBuilder;
                use fyrox::gui::Orientation;
                let mut ctx = cx.ctx();
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_name("raikou_switch")
                        .with_margin(to_fyrox_thickness(self.margin))
                        .with_child(track)
                        .with_child(label),
                )
                .with_orientation(Orientation::Horizontal)
                .build(&mut ctx)
                .to_base()
            }
            None => {
                use fyrox::gui::stack_panel::StackPanelBuilder;
                let mut ctx = cx.ctx();
                StackPanelBuilder::new(
                    WidgetBuilder::new()
                        .with_name("raikou_switch")
                        .with_child(track),
                )
                .build(&mut ctx)
                .to_base()
            }
        };

        let kind = ComponentKind::Switch(SwitchHandlers {
            on_change: None,
            command_target: track,
            knob,
            subtree: outer,
            animator: None,
            keys_only: false,
        });
        // Tweener node that animates the knob slide.
        let tweener = spawn_tweener(&mut cx.ctx(), outer);
        let component_outer = Component {
            handle: outer,
            kind: kind.clone(),
        };
        let component_inner = Component {
            handle: track,
            kind: ComponentKind::Switch(SwitchHandlers {
                on_change: self.on_change.clone(),
                command_target: track,
                knob,
                subtree: outer,
                animator: Some(tweener),
                keys_only: false,
            }),
        };
        cx.register(&component_outer);
        cx.register(&component_inner);
        // Global watcher so Space works when a deep child (decorator, knob)
        // holds focus; the keys-only clone ignores state echoes entirely.
        let mut key_watcher = kind.clone();
        if let ComponentKind::Switch(handlers) = &mut key_watcher {
            handlers.keys_only = true;
        }
        cx.register_global(&Component {
            handle: outer,
            kind: key_watcher,
        });
        component_outer
    }
}

/// A handle to a built switch, returned for convenience.
pub type SwitchHandle = Handle<UiNode>;
