//! tree panel — playground demo for the raikou `Tree` component.
//!
//! Port of the reference `tree_demo`: a small file-tree with two roots, nodes
//! starting expanded so the hierarchy and disclosure affordances are visible.

use fyrox::core::pool::Handle;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou_playground::*;

const CODE: &str = r#"Tree::new()
    .node(
        TreeNode::new("Roadmap")
            .expanded()
            .child(TreeNode::new("Widgets").expanded())
            .child(TreeNode::new("Docs")),
    )
    .node(TreeNode::new("Releases").expanded())"#;

pub fn tree_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let tree = Tree::new()
        .node(
            TreeNode::new("Roadmap")
                .expanded()
                .child(
                    TreeNode::new("Widgets")
                        .expanded()
                        .child(TreeNode::new("Accordion"))
                        .child(TreeNode::new("Table"))
                        .child(TreeNode::new("Tree")),
                )
                .child(TreeNode::new("Docs")),
        )
        .node(
            TreeNode::new("Releases")
                .expanded()
                .child(TreeNode::new("v0.1.0"))
                .child(TreeNode::new("v0.2.0-preview")),
        )
        .build(&mut cx);

    let preview = PlaygroundPreview::new(tree)
        .content_max_size(320.0, 240.0)
        .build(&mut cx);

    let notes = playground_notes(
        &mut cx,
        "Tree playground",
        &[
            "Nodes start expanded so the hierarchy and indentation are visible.",
            "Click rows to toggle expansion and watch the layout update.",
        ],
    )
    .build(&mut cx);

    let code = PlaygroundCodeBlock::new(|| CODE.to_string()).build(&mut cx);
    let code_panel = PlaygroundCodePanel::new("Tree.rs", code).build(&mut cx);

    let shell = PlaygroundShell::new(preview, notes, code_panel)
        .sidebar_width(280.0)
        .code_height(220.0)
        .build(&mut cx);
    let shell_handle: Handle<UiNode> = shell.into();
    cx.ui().send(shell_handle, WidgetMessage::Width(980.0));
    cx.ui().send(shell_handle, WidgetMessage::Height(760.0));

    // RAIKOU_TREE_SELECT=1: simulate a left-click on the first tree item so
    // the native selection brush is visible in audit captures.
    if std::env::var("RAIKOU_TREE_SELECT").as_deref() == Ok("1") {
        use fyrox::graph::SceneGraph;
        use fyrox::gui::message::MouseButton;

        let mut stack = vec![ui.root()];
        while let Some(h) = stack.pop() {
            if h.is_none() {
                continue;
            }
            if ui.try_get_of_type::<fyrox::gui::tree::Tree>(h).is_ok() {
                ui.send(
                    h,
                    WidgetMessage::MouseDown {
                        pos: Default::default(),
                        button: MouseButton::Left,
                    },
                );
                break;
            }
            stack.extend(ui.node(h).children().iter().copied());
        }
    }

    shell_handle
}
