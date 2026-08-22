//! Functional tests for the Tree component.

mod common;

use common::{Counter, Harness};
use fyrox::graph::SceneGraph;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::tree::TreeRootMessage;
use raikou_widgets::{Tree, TreeNode};

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
