//! Functional tests for Label and the container widgets.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use raikou_widgets::{set_label_text, BoxWidget, Group, Label, Stack};

#[test]
fn label_builds_and_updates_text() {
    let mut h = Harness::new();
    let label = h.build(|cx| Label::new("Hello").font_size(16.0).build(cx));

    set_label_text(&h.ui, label.handle, "World");
    h.pump();

    use fyrox::graph::SceneGraph;
    use fyrox::gui::text::Text;
    let text = h.ui.node(label.handle).cast::<Text>().unwrap();
    assert_eq!(text.text(), "World", "set_label_text must update the text");
}

#[test]
fn stack_links_children_in_order() {
    let mut h = Harness::new();
    let stack = h.build(|cx| {
        let a = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("A")
                .build(&mut ctx)
                .to_base()
        };
        let b = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("B")
                .build(&mut ctx)
                .to_base()
        };
        Stack::new().spacing(8.0).children([a, b]).build(cx)
    });

    use fyrox::graph::SceneGraph;
    let children: Vec<_> = h.ui.node(stack.handle).children().to_vec();
    assert_eq!(children.len(), 2, "stack must contain both children");
}

#[test]
fn group_and_box_build() {
    let mut h = Harness::new();
    let group = h.build(|cx| Group::new().spacing(4.0).build(cx));
    let boxed = h.build(|cx| BoxWidget::new().width(120.0).height(40.0).build(cx));

    assert!(h.ui.try_get_node(group.handle).is_ok());
    assert!(h.ui.try_get_node(boxed.handle).is_ok());
}
