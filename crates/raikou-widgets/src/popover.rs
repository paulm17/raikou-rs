//! The Popover component — a floating panel anchored to an owner widget.

use fyrox::core::pool::Handle;
use fyrox::gui::border::Border;
use fyrox::gui::brush::Brush;
use fyrox::gui::popup::{Placement, Popup, PopupBuilder, PopupMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_color;

/// Builder for a [`crate::Popover`] component.
#[derive(Clone)]
pub struct Popover {
    content: Handle<UiNode>,
    owner: Handle<UiNode>,
    placement: Placement,
    stays_open: bool,
}

impl Default for Popover {
    fn default() -> Self {
        Self::new()
    }
}

impl Popover {
    /// Creates a new popover builder.
    pub fn new() -> Self {
        Self {
            content: Handle::NONE,
            owner: Handle::NONE,
            placement: Placement::RightBottom(Handle::NONE),
            // Fluent popups/flyouts light-dismiss on outside clicks by
            // default (Avalonia PopupMode = LightDismiss); opt back in via
            // `.stays_open(true)`.
            stays_open: false,
        }
    }

    /// Sets the popup content.
    pub fn content(mut self, content: impl Into<Handle<UiNode>>) -> Self {
        self.content = content.into();
        self
    }

    /// Sets the owner widget the popup is anchored to.
    pub fn owner(mut self, owner: impl Into<Handle<UiNode>>) -> Self {
        self.owner = owner.into();
        self
    }

    /// Sets the placement relative to the owner.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Controls whether the popup stays open on outside clicks (off by
    /// default, matching Fluent light-dismiss behavior).
    pub fn stays_open(mut self, stays_open: bool) -> Self {
        self.stays_open = stays_open;
        self
    }

    /// Builds the popover and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            PopupBuilder::new(WidgetBuilder::new().with_name("raikou_popover"))
                .with_content(self.content)
                .with_owner(self.owner)
                .with_placement(self.placement)
                .stays_open(self.stays_open)
                .build(&mut ctx)
                .to_base()
        };

        // Fluent flyout chrome: the stock fyrox popup body is a Border with
        // a hardcoded dark surface (fyrox's global primary brush), which
        // reads as a dark slab in the light theme. Restyle it as an elevated
        // surface with a subtle 1px stroke and the Fluent overlay corner
        // radius (Avalonia FlyoutPresenter: FlyoutPresenterBackground +
        // ControlStroke border @ 1px + OverlayCornerRadius = 8).
        {
            let (body, elevated, stroke): (
                Handle<UiNode>,
                Brush,
                Brush,
            ) = {
                use fyrox::graph::SceneGraph;
                let fallback_white = raikou_core::Color::new(1.0, 1.0, 1.0, 1.0);
                let elevated = Brush::Solid(to_fyrox_color(
                    cx.theme().color("surface.elevated").unwrap_or(fallback_white),
                ));
                let stroke = Brush::Solid(to_fyrox_color(
                    cx.theme()
                        .color("border.subtle")
                        .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.14)),
                ));
                let popup = cx.ui().try_get_of_type::<Popup>(handle).ok();
                let body = popup.map(|p| *p.body).unwrap_or_default();
                (body, elevated, stroke)
            };
            if !body.is_none() {
                let mut ctx = cx.ctx();
                if let Some(border) = ctx[body].cast_mut::<Border>() {
                    border
                        .widget
                        .background
                        .set_value_and_mark_modified(elevated.into());
                    border
                        .widget
                        .foreground
                        .set_value_and_mark_modified(stroke.into());
                    border
                        .corner_radius
                        .set_value_and_mark_modified(8.0f32.into());
                }
            }
        }

        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built popover.
pub type PopoverHandle = Handle<UiNode>;

/// Opens the popover.
pub fn show_popover(ui: &fyrox::gui::UserInterface, popover: Handle<UiNode>) {
    ui.send(popover, PopupMessage::Open);
}

/// Closes the popover.
pub fn hide_popover(ui: &fyrox::gui::UserInterface, popover: Handle<UiNode>) {
    ui.send(popover, PopupMessage::Close);
}
