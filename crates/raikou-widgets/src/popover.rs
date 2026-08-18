//! The Popover component — a floating panel anchored to an owner widget.

use fyrox::core::pool::Handle;
use fyrox::gui::popup::{Placement, PopupBuilder, PopupMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};

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
            stays_open: true,
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

    /// Controls whether the popup stays open on outside clicks.
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
