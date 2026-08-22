//! The Stack component — a vertical column of children.

use fyrox::core::pool::Handle;
use fyrox::gui::stack_panel::StackPanelBuilder;
use fyrox::gui::widget::{WidgetBuilder, WidgetMessage};
use fyrox::gui::UiNode;

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

/// Builder for a [`crate::Stack`] component (vertical column).
#[derive(Clone, Default)]
pub struct Stack {
    children: Vec<Handle<UiNode>>,
    spacing: f32,
    margin: Thickness,
}

impl Stack {
    /// Creates a new stack builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the spacing between children.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Appends a child.
    pub fn child(mut self, child: impl Into<Handle<UiNode>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Appends children.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Handle<UiNode>>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the stack and adds it to the UI.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let n = self.children.len();
        if self.spacing > 0.0 && n > 1 {
            let ui = cx.ui();
            for (i, child) in self.children.iter().enumerate() {
                if i < n - 1 {
                    ui.send(
                        *child,
                        WidgetMessage::Margin(fyrox::gui::Thickness {
                            left: 0.0,
                            top: 0.0,
                            right: 0.0,
                            bottom: self.spacing,
                        }),
                    );
                }
            }
        }

        let handle: Handle<UiNode> = {
            let mut ctx = cx.ctx();
            StackPanelBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_stack")
                    .with_margin(to_fyrox_thickness(self.margin))
                    .with_children(self.children),
            )
            .with_orientation(fyrox::gui::Orientation::Vertical)
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

/// A handle to a built stack.
pub type StackHandle = Handle<UiNode>;
