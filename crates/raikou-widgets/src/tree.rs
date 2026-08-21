//! Tree component: a recursive node list backed by fyrox's `Tree`/`TreeRoot`.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tree::{TreeBuilder, TreeRootBuilder, TreeRootMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type SelectCallback = dyn Fn(&mut UserInterface, usize);

/// A tree node: a label plus optional children.
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// The node label.
    pub label: String,
    /// Child nodes.
    pub children: Vec<TreeNode>,
    /// Whether the node is expanded.
    pub expanded: bool,
    /// Whether the node is selected.
    pub selected: bool,
}

impl TreeNode {
    /// Creates a new node with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            selected: false,
        }
    }

    /// Adds a child node.
    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }

    /// Marks the node as expanded by default.
    pub fn expanded(mut self) -> Self {
        self.expanded = true;
        self
    }
}

/// Event handlers of a Tree component.
#[derive(Clone)]
pub struct TreeHandlers {
    on_select: Option<Rc<SelectCallback>>,
}

impl TreeHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        if let Some(on_select) = &self.on_select {
            if let Some(TreeRootMessage::Select(selected)) = message.data::<TreeRootMessage>() {
                on_select(ui, selected.len());
            }
        }
    }
}

/// Builder for a [`Tree`] component.
#[derive(Clone)]
pub struct Tree {
    roots: Vec<TreeNode>,
    item_height: f32,
    on_select: Option<Rc<SelectCallback>>,
    margin: Thickness,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    /// Creates a new tree builder.
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            item_height: 28.0,
            on_select: None,
            margin: Thickness::ZERO,
        }
    }

    /// Adds a root node.
    pub fn node(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Sets the item height (clamped to a minimum of 16).
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height.max(16.0);
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the callback invoked when a selection changes (passes the count).
    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, usize) + 'static,
    {
        self.on_select = Some(Rc::new(callback));
        self
    }

    /// Builds the tree, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();
        let font = ctx.default_font();

        let root_nodes: Vec<Handle<fyrox::gui::tree::Tree>> = self
            .roots
            .iter()
            .map(|node| build_tree_node(node, font.clone(), &mut ctx))
            .collect();

        let handle = TreeRootBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_tree")
                .with_margin(to_fyrox_thickness(self.margin)),
        )
        .with_items(root_nodes)
        .build(&mut ctx)
        .to_base();

        let component = Component {
            handle,
            kind: ComponentKind::Tree(TreeHandlers {
                on_select: self.on_select,
            }),
        };
        cx.register(&component);
        component
    }
}

/// Recursively builds a `TreeNode` into a fyrox `Tree`.
fn build_tree_node(
    node: &TreeNode,
    font: fyrox::gui::font::FontResource,
    ctx: &mut fyrox::gui::BuildContext,
) -> Handle<fyrox::gui::tree::Tree> {
    let label: Handle<UiNode> = fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
        .with_text(&node.label)
        .with_font(font.clone())
        .build(ctx)
        .to_base();

    let mut child_trees: Vec<Handle<fyrox::gui::tree::Tree>> = Vec::new();
    for child in &node.children {
        child_trees.push(build_tree_node(child, font.clone(), ctx));
    }

    TreeBuilder::new(WidgetBuilder::new())
        .with_content(label)
        .with_items(child_trees)
        .with_expanded(node.expanded)
        .build(ctx)
}

/// A handle to a built tree.
pub type TreeHandle = Handle<UiNode>;
