//! tree_demo — exercises the Phase 4 Tree component: nested nodes,
//! expanded-by-default nodes and selection reporting.

use fyrox::core::pool::Handle;
use fyrox::gui::text::{TextBuilder, TextMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};
use raikou::prelude::*;
use raikou::{Color, Thickness};
use raikou_demo::Options;

fn build_demo_panel(
    ui: &mut UserInterface,
    theme: &Theme,
    registry: &mut ComponentRegistry,
) -> Handle<UiNode> {
    let mut cx = BuildCx::new(ui, theme, registry);

    let status: Handle<UiNode> = TextBuilder::new(
        WidgetBuilder::new().with_name("raikou_status"),
    )
    .with_text("no interaction yet")
    .build(&mut cx.ctx())
    .to_base();

    let tree = Tree::new()
        .node(
            TreeNode::new("Project")
                .expanded()
                .child(
                    TreeNode::new("src")
                        .expanded()
                        .child(TreeNode::new("main.rs"))
                        .child(TreeNode::new("lib.rs")),
                )
                .child(TreeNode::new("Cargo.toml")),
        )
        .node(TreeNode::new("Docs").child(TreeNode::new("architecture.md")))
        .item_height(28.0)
        .margin(Thickness::new(0.0, 0.0, 0.0, 16.0))
        .on_select(move |ui, count| {
            ui.send(status, TextMessage::Text(format!("selection -> {count} node(s)")));
        })
        .build(&mut cx);
    let tree_handle: Handle<UiNode> = tree.into();

    let heading = Label::new("Tree")
        .font_size(18.0)
        .color(Color::new(0.09, 0.09, 0.10, 1.0))
        .build(&mut cx);
    let heading_handle: Handle<UiNode> = heading.into();

    let hint = Label::new("Expand the nodes and click a leaf to select it.")
        .color(theme.color("text.muted").unwrap_or(Color::new(0.4, 0.4, 0.4, 1.0)))
        .build(&mut cx);
    let hint_handle: Handle<UiNode> = hint.into();

    Stack::new()
        .spacing(12.0)
        .child(heading_handle)
        .child(tree_handle)
        .child(hint_handle)
        .child(status)
        .build(&mut cx)
        .into()
}

fn main() {
    raikou_demo::run(
        Options {
            title: "raikou — tree demo".to_string(),
            width: 900,
            height: 600,
        },
        Box::new(build_demo_panel),
    );
}