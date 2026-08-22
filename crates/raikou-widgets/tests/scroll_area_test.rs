//! Functional tests for the ScrollArea component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::scroll_viewer::ScrollViewerMessage;
use raikou_core::Length;
use raikou_widgets::ScrollArea;

fn scroll(
    h: &mut Harness,
    handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>,
    msg: ScrollViewerMessage,
) {
    h.ui.send_message(
        UiMessage::with_data(msg)
            .with_destination(handle)
            .with_direction(MessageDirection::FromWidget),
    );
}

#[test]
fn scroll_area_reports_offsets() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<(f32, f32)>::new()));
    let s = seen.clone();
    let area = h.build(move |cx| {
        let content = {
            let mut ctx = cx.ctx();
            fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
                .with_text("tall content")
                .build(&mut ctx)
                .to_base()
        };
        ScrollArea::new()
            .content(content)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(100.0))
            .on_scroll(move |_, v, hh| s.borrow_mut().push((v, hh)))
            .build(cx)
    });

    scroll(
        &mut h,
        area.handle,
        ScrollViewerMessage::VerticalScroll(42.0),
    );
    h.pump();
    assert_eq!(
        seen.borrow().last(),
        Some(&(42.0, 0.0)),
        "vertical scroll must report (v, h)"
    );

    scroll(
        &mut h,
        area.handle,
        ScrollViewerMessage::HorizontalScroll(7.0),
    );
    h.pump();
    assert_eq!(
        seen.borrow().last(),
        Some(&(42.0, 7.0)),
        "horizontal scroll must keep last vertical offset"
    );
}
