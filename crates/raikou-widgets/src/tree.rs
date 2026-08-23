//! Tree component: a recursive node list backed by fyrox's `Tree`/`TreeRoot`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageData, MessageDirection, UiMessage};
use fyrox::gui::tree::{Tree as FyroxTree, TreeBuilder, TreeRootBuilder, TreeRootMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::Thickness;

use crate::accordion::chevron_mark_nodes;
use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type SelectCallback = dyn Fn(&mut UserInterface, usize);
type SelectionCallback = dyn Fn(&mut UserInterface, Vec<usize>);

/// Programmatic commands accepted by a built tree (send `ToWidget` to the
/// component handle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCommand {
    /// Selects the nodes at the given depth-first indices (empty clears).
    Select(Vec<usize>),
}

impl MessageData for TreeCommand {}

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

    /// Marks the node as selected initially.
    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// Event handlers of a Tree component.
#[derive(Clone)]
pub struct TreeHandlers {
    on_select: Option<Rc<SelectCallback>>,
    on_selection: Option<Rc<SelectionCallback>>,
    /// The built tree root (command translation target).
    root: Handle<UiNode>,
    /// Depth-first node handles (children before parent); index = position in the user's model.
    nodes: Rc<Vec<Handle<FyroxTree>>>,
    /// Node handle → depth-first index for payload mapping.
    index_of: Rc<RefCell<HashMap<Handle<FyroxTree>, usize>>>,
}

impl TreeHandlers {
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if let Some(TreeRootMessage::Select(selected)) = message.data::<TreeRootMessage>() {
            if message.direction() == MessageDirection::FromWidget {
                if let Some(on_select) = &self.on_select {
                    on_select(ui, selected.len());
                }
                // Lossless payload: map fyrox handles back to depth-first
                // indices in the user's model (unknown handles are skipped).
                let indices: Vec<usize> = {
                    let index_of = self.index_of.borrow();
                    selected
                        .iter()
                        .filter_map(|h| index_of.get(h).copied())
                        .collect()
                };
                if let Some(on_selection) = &self.on_selection {
                    on_selection(ui, indices);
                }
            }
            return;
        }
        // Programmatic selection by model index.
        if message.direction() == MessageDirection::ToWidget {
            if let Some(TreeCommand::Select(indices)) = message.data::<TreeCommand>() {
                let nodes = self.nodes.clone();
                let handles: Vec<Handle<FyroxTree>> = indices
                    .iter()
                    .filter_map(|i| nodes.get(*i).copied())
                    .collect();
                ui.send(self.root, TreeRootMessage::Select(handles));
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
    on_selection: Option<Rc<SelectionCallback>>,
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
            // Fluent TreeView rows are ~32px tall; fyrox bakes 24px.
            item_height: 32.0,
            on_select: None,
            on_selection: None,
            margin: Thickness::ZERO,
        }
    }

    /// Adds a root node.
    pub fn node(mut self, node: TreeNode) -> Self {
        self.roots.push(node);
        self
    }

    /// Sets the item row height (clamped to a minimum of 16).
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

    /// Sets the callback invoked when a selection changes (passes the
    /// depth-first indices of every selected node — lossless payload).
    pub fn on_selection<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, Vec<usize>) + 'static,
    {
        self.on_selection = Some(Rc::new(callback));
        self
    }

    /// Builds the tree, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();
        let font = ctx.default_font();

        // Build every node, in collecting depth-first handles (index = position        // in the user's model) plus the requested initial-selection flags.
        let mut nodes: Vec<Handle<FyroxTree>> = Vec::new();
        let mut selected_flags: Vec<bool> = Vec::new();
        let root_nodes: Vec<Handle<FyroxTree>> = self
            .roots
            .iter()
            .map(|node| {
                let mut flags = Vec::new();
                collect_selection_flags(node, &mut flags);
                selected_flags.append(&mut flags);
                build_tree_node(node, font.clone(), &mut ctx, &mut nodes)
            })
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

        // Fluent row chrome: fyrox bakes 24px rows with a saturated dim-blue
        // selection brush and an opaque hover brush; restyle every descendant
        // tree to subtle list tints and apply the requested item height.
        {
            use fyrox::graph::SceneGraph;
            use fyrox::gui::brush::Brush;
            use fyrox::gui::decorator::Decorator;
            use fyrox::gui::grid::{Grid, Row, SizeMode};
            use fyrox::gui::widget::WidgetMessage;
            use fyrox::gui::VerticalAlignment;
            use std::cell::RefCell;

            use crate::convert::to_fyrox_color;

            let theme = cx.theme().clone();
            let hover_brush = Brush::Solid(to_fyrox_color(
                theme
                    .color("fluent.list.low")
                    .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.05)),
            ));
            let selected_brush = Brush::Solid(to_fyrox_color(
                theme
                    .color("fluent.list.medium")
                    .unwrap_or(raikou_core::Color::new(0.0, 0.0, 0.0, 0.10)),
            ));

            // Collect every descendant tree first (immutable pass), then
            // mutate: try_get_of_type borrows the UI for the duration.
            let trees: Vec<Handle<UiNode>> = {
                let ui = cx.ui();
                let mut stack = vec![handle];
                let mut visited = vec![handle];
                let mut trees = Vec::new();
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
                trees
            };

            let ui = cx.ui();
            for tree_handle in trees {
                let Ok(tree) = ui.try_get_of_type::<fyrox::gui::tree::Tree>(tree_handle) else {
                    continue;
                };
                let background = tree.background;
                let content = tree.content;
                let expander: Handle<UiNode> = tree.expander.to_base();

                // Swap the stock hover/selection brushes for Fluent list tints.
                if let Some(decorator) = ui.node_mut(background).cast_mut::<Decorator>() {
                    decorator
                        .hover_brush
                        .set_value_and_mark_modified(hover_brush.clone().into());
                    decorator
                        .selected_brush
                        .set_value_and_mark_modified(selected_brush.clone().into());
                }

                // Row heights live in two grids: the outer row strip and the
                // internals grid nested inside the item background border
                // (which insets its child by its 1px stroke on each side).
                let item_height = self.item_height.max(16.0);
                let outer_grid = ui.node(tree_handle).children()[0];
                let internals_grid = ui.node(background).children()[0];
                for (grid, height) in [
                    (outer_grid, item_height),
                    (internals_grid, item_height - 2.0),
                ] {
                    if let Some(grid) = ui.node_mut(grid).cast_mut::<Grid>() {
                        let mut rows = grid.rows.borrow().clone();
                        if rows.is_empty() || rows[0].size_mode != SizeMode::Strict {
                            continue;
                        }
                        rows[0] = Row::strict(height);
                        grid.rows.set_value_and_mark_modified(RefCell::new(rows));
                    }
                }

                // Center the label and expander within the taller row.
                ui.send(content, WidgetMessage::VerticalAlignment(VerticalAlignment::Center));
                ui.send(expander, WidgetMessage::VerticalAlignment(VerticalAlignment::Center));
            }
        }

        // Initial selection: fyrox hardcodes `is_selected: false` at build,
        // so revive `TreeNode.selected` by posting the selection after build.
        {
            let ui = cx.ui();
            let selected_handles: Vec<Handle<FyroxTree>> = nodes
                .iter()
                .zip(&selected_flags)
                .filter(|(_, sel)| **sel)
                .map(|(h, _)| *h)
                .collect();
            if !selected_handles.is_empty() {
                ui.send(handle, TreeRootMessage::Select(selected_handles));
            }
        }

        let index_of: HashMap<Handle<FyroxTree>, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, h)| (*h, i))
            .collect();
        let component = Component {
            handle,
            kind: ComponentKind::Tree(TreeHandlers {
                on_select: self.on_select,
                on_selection: self.on_selection,
                root: handle,
                nodes: Rc::new(nodes),
                index_of: Rc::new(RefCell::new(index_of)),
            }),
        };
        cx.register(&component);
        component
    }
}

/// Collects the `selected` flags of a node subtree. Traversal mirrors
/// `build_tree_node`'s handle emission order (children first — fyrox trees
/// are built bottom-up, so a parent's handle exists only after its
/// children's).
fn collect_selection_flags(node: &TreeNode, flags: &mut Vec<bool>) {
    for child in &node.children {
        collect_selection_flags(child, flags);
    }
    flags.push(node.selected);
}

/// Recursively builds a `TreeNode` into a fyrox `Tree`, appending each
/// created handle to `nodes` in depth-first.
fn build_tree_node(
    node: &TreeNode,
    font: fyrox::gui::font::FontResource,
    ctx: &mut fyrox::gui::BuildContext,
    nodes: &mut Vec<Handle<FyroxTree>>,
) -> Handle<FyroxTree> {
    let label: Handle<UiNode> = fyrox::gui::text::TextBuilder::new(WidgetBuilder::new())
        .with_text(&node.label)
        .with_font(font.clone())
        .build(ctx)
        .to_base();

    let mut child_trees: Vec<Handle<FyroxTree>> = Vec::new();
    let mut child_nodes: Vec<Handle<FyroxTree>> = Vec::new();
    for child in &node.children {
        child_trees.push(build_tree_node(child, font.clone(), ctx, &mut child_nodes));
    }
    nodes.append(&mut child_nodes);

    let tree = TreeBuilder::new(WidgetBuilder::new())
        .with_content(label)
        .with_items(child_trees)
        .with_expanded(node.expanded)
        .build(ctx);
    nodes.push(tree);
    tree
}

/// A handle to a built tree.
pub type TreeHandle = Handle<UiNode>;
