//! Tree component: a recursive node list backed by fyrox's `Tree`/`TreeRoot`.

use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tree::{TreeBuilder, TreeRootBuilder, TreeRootMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::accordion::chevron_mark_nodes;
use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type SelectCallback = dyn Fn(&mut UserInterface, usize);

/// One descendant tree's expander: the checkbox handle, its checked state,
/// the two stock mark handles and the background border that hosts them.
type ExpanderShape = (
    fyrox::core::pool::Handle<fyrox::gui::check_box::CheckBox>,
    bool,
    Handle<UiNode>,
    Handle<UiNode>,
    Handle<UiNode>,
);

/// An expander mid-swap: checkbox + state, old marks, replacement marks.
type ExpanderSwap = (
    fyrox::core::pool::Handle<fyrox::gui::check_box::CheckBox>,
    bool,
    Handle<UiNode>,
    Handle<UiNode>,
    Handle<UiNode>,
    Handle<UiNode>,
);

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

        // Fluent chevron expanders: fyrox's stock tree expanders use bright
        // triangle arrows; swap every descendant tree's expander marks for
        // vector chevrons. Note: trees added after this build are not
        // restyled (acceptable for static trees).
        {
            use fyrox::graph::SceneGraph;
            use fyrox::gui::check_box::CheckBox;
            use fyrox::gui::widget::WidgetMessage;

            // Pass 1: collect every descendant tree's expander shape.
            let expanders: Vec<ExpanderShape> = {
                let ui = cx.ui();
                let mut stack = vec![handle];
                let mut visited = vec![handle];
                let mut trees: Vec<Handle<UiNode>> = Vec::new();
                while let Some(h) = stack.pop() {
                    if h.is_none() {
                        continue;
                    }
                    if ui.try_get_of_type::<fyrox::gui::tree::Tree>(h).is_ok() {
                        trees.push(h);
                    }
                    for child in ui.node(h).children().to_vec() {
                        if !child.is_none() && !visited.contains(&child) {
                            visited.push(child);
                            stack.push(child);
                        }
                    }
                }

                let mut expanders = Vec::new();
                for tree_handle in trees {
                    let tree = match ui.try_get_of_type::<fyrox::gui::tree::Tree>(tree_handle) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let expander = tree.expander;
                    let cb_node = ui.node(expander.to_base());
                    if cb_node.children().len() != 1 {
                        continue;
                    }
                    let grid = cb_node.children()[0];
                    if ui.node(grid).children().is_empty() {
                        continue;
                    }
                    let background = ui.node(grid).children()[0];
                    let cb = match ui.try_get_of_type::<CheckBox>(expander.to_base()) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    expanders.push((
                        expander,
                        cb.checked.unwrap_or(false),
                        *cb.check_mark,
                        *cb.uncheck_mark,
                        background,
                    ));
                }
                expanders
            };

            // Pass 2: build replacement marks and nest them in each host.
            let built: Vec<ExpanderSwap> = {
                let theme = cx.theme().clone();
                let mut ctx = cx.ctx();
                let mut built = Vec::new();
                for (expander, checked, old_check, old_uncheck, background) in expanders {
                    let (new_check, new_uncheck) = chevron_mark_nodes(&mut ctx, &theme);
                    ctx.link(new_check, background);
                    ctx.link(new_uncheck, background);
                    built.push((
                        expander,
                        checked,
                        old_check,
                        old_uncheck,
                        new_check,
                        new_uncheck,
                    ));
                }
                built
            };

            // Pass 3: retire old marks and wire in the right chevron per state.
            {
                let ui = cx.ui();
                for (expander, checked, old_check, old_uncheck, new_check, new_uncheck) in built {
                    ui.send(old_check, WidgetMessage::Visibility(false));
                    ui.send(old_uncheck, WidgetMessage::Visibility(false));
                    ui.send(new_check, WidgetMessage::Visibility(checked));
                    ui.send(new_uncheck, WidgetMessage::Visibility(!checked));

                    if let Ok(cb) = ui.try_get_mut_of_type::<CheckBox>(expander.to_base()) {
                        cb.check_mark.set_value_and_mark_modified(new_check);
                        cb.uncheck_mark.set_value_and_mark_modified(new_uncheck);
                    }
                }
            }
        }

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
