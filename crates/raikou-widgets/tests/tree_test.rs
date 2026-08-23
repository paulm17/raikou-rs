//! Functional tests for the Tree component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tree::TreeRootMessage;
use raikou_widgets::{Tree, TreeCommand, TreeNode};

#[test]
fn tree_reports_selection_count() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(usize::MAX));
    let s = seen.clone();
    let tree = h.build(move |cx| {
        Tree::new()
            .node(
                TreeNode::new("Root")
                    .child(TreeNode::new("Child A"))
                    .child(TreeNode::new("Child B")),
            )
            .on_select(move |_, n| s.set(n))
            .build(cx)
    });

    // FromWidget Select reports carry the selected item handles; raikou
    // forwards their count.
    h.ui.send_message(
        UiMessage::with_data(TreeRootMessage::Select(vec![Default::default(); 2]))
            .with_destination(tree.handle)
            .with_direction(MessageDirection::FromWidget),
    );
    h.pump();
    assert_eq!(seen.get(), 2, "selection of 2 items must report count 2");
}

#[test]
fn tree_build_creates_nodes() {
    let mut h = Harness::new();
    let tree = h.build(|cx| {
        Tree::new()
            .node(TreeNode::new("Only").child(TreeNode::new("Nested")))
            .item_height(24.0)
            .build(cx)
    });

    assert!(
        h.ui.try_get_node(tree.handle).is_ok(),
        "tree root must exist in the graph"
    );
}

/// Finds every fyrox `Tree` node under `root` and returns
/// `(label, handle)` pairs in document order.
fn labeled_trees(
    h: &Harness,
    root: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
) -> Vec<(String, fyrox::core::pool::Handle<fyrox::gui::tree::Tree>)> {
    use fyrox::gui::text::Text;

    let mut stack = vec![root];
    let mut visited = vec![root];
    let mut found = Vec::new();
    while let Some(node) = stack.pop() {
        if node.is_none() {
            continue;
        }
        if let Ok(tree) = h.ui.try_get_of_type::<fyrox::gui::tree::Tree>(node) {
            let label = h
                .ui
                .try_get_of_type::<Text>(tree.content)
                .map(|t| t.text())
                .unwrap_or_default();
            found.push((label, node.transmute()));
        }
        for child in h.ui.node(node).children().to_vec() {
            if !child.is_none() && !visited.contains(&child) {
                visited.push(child);
                stack.push(child);
            }
        }
    }
    found
}

#[test]
fn tree_on_selection_reports_indices() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let s = seen.clone();
    let tree = h.build(move |cx| {
        Tree::new()
            .node(
                TreeNode::new("Root")
                    .child(TreeNode::new("Child A"))
                    .child(TreeNode::new("Child B")),
            )
            .on_selection(move |_, indices| s.borrow_mut().push(indices))
            .build(cx)
    });

    // Depth-first model order (children before parent): A=0, B=1, Root=2.
    let trees = labeled_trees(&h, tree.handle);
    let b = trees
        .iter()
        .find(|(label, _)| label == "Child B")
        .map(|(_, handle)| *handle)
        .expect("Child B tree must exist");

    h.ui.send_message(
        UiMessage::with_data(TreeRootMessage::Select(vec![b]))
            .with_destination(tree.handle)
            .with_direction(MessageDirection::FromWidget),
    );
    h.pump();
    assert_eq!(*seen.borrow(), vec![vec![1]], "Child B maps to index 1");
}

#[test]
fn tree_programmatic_select_command_applies() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let s = seen.clone();
    let tree = h.build(move |cx| {
        Tree::new()
            .node(
                TreeNode::new("Root").child(TreeNode::new("Child A")),
            )
            .on_selection(move |_, indices| s.borrow_mut().push(indices))
            .build(cx)
    });

    h.ui.send_message(
        UiMessage::with_data(TreeCommand::Select(vec![0]))
            .with_destination(tree.handle)
            .with_direction(MessageDirection::ToWidget),
    );
    h.pump();

    // The native mirror of the applied selection reports back through the
    // lossless payload callback...
    assert_eq!(
        *seen.borrow(),
        vec![vec![0]],
        "programmatic select must report the same index"
    );

    // ...and the root's selection state must hold the mapped handle.
    let root = h
        .ui
        .try_get_of_type::<fyrox::gui::tree::TreeRoot>(tree.handle)
        .expect("tree root exists");
    assert_eq!(root.selected.len(), 1, "one node selected");
}

#[test]
fn tree_initial_selected_flag_applies() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let s = seen.clone();
    let tree = h.build(move |cx| {
        Tree::new()
            .node(
                TreeNode::new("Root")
                    .expanded()
                    .child(TreeNode::new("Child A").selected()),
            )
            .on_selection(move |_, indices| s.borrow_mut().push(indices))
            .build(cx)
    });
    h.update_and_pump();

    let root = h
        .ui
        .try_get_of_type::<fyrox::gui::tree::TreeRoot>(tree.handle)
        .expect("tree root exists");
    assert_eq!(
        root.selected.len(),
        1,
        "TreeNode.selected must revive as an initial selection"
    );
    assert_eq!(*seen.borrow(), vec![vec![0]]);
}
