//! Functional tests for the ScrollArea component.

mod common;

use common::Harness;
use fyrox::core::pool::Handle;
use fyrox::graph::SceneGraph;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::scroll_bar::ScrollBar;
use fyrox::gui::scroll_viewer::ScrollViewerMessage;
use fyrox::gui::widget::WidgetMessage;
use fyrox::gui::{Orientation, UiNode};
use raikou_core::Length;
use raikou_widgets::ScrollArea;

fn scroll(
    h: &mut Harness,
    handle: Handle<UiNode>,
    msg: ScrollViewerMessage,
) {
    h.ui.send_message(
        UiMessage::with_data(msg)
            .with_destination(handle)
            .with_direction(MessageDirection::FromWidget),
    );
}

fn send_widget(h: &mut Harness, handle: Handle<UiNode>, msg: WidgetMessage) {
    h.ui.send_message(
        UiMessage::with_data(msg)
            .with_destination(handle)
            .with_direction(MessageDirection::ToWidget),
    );
}

/// Finds the indicator (thumb) of the vertical bar of a scroll area.
fn vertical_indicator(h: &Harness, area: Handle<UiNode>) -> Handle<UiNode> {
    let mut stack = vec![area];
    while let Some(handle) = stack.pop() {
        if let Ok(bar) = h.ui.try_get_of_type::<ScrollBar>(handle) {
            if *bar.orientation == Orientation::Vertical {
                return *bar.indicator;
            }
        }
        for child in h.ui.node(handle).children().to_vec() {
            stack.push(child);
        }
    }
    panic!("no vertical ScrollBar under the scroll area");
}

fn thumb_visible(h: &Harness, indicator: Handle<UiNode>) -> bool {
    *h.ui.node(indicator).visibility
}

fn build_overflowing_area(h: &mut Harness) -> Handle<UiNode> {
    let content = {
        let mut ctx = h.ui.build_ctx();
        fyrox::gui::text::TextBuilder::new(fyrox::gui::widget::WidgetBuilder::new())
            .with_text("tall content")
            .build(&mut ctx)
            .to_base()
    };
    let component = h.build(move |cx| {
        ScrollArea::new()
            .content(content)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(100.0))
            .build(cx)
    });
    component.handle
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

#[test]
fn overlay_thumb_hidden_until_hover_or_scroll() {
    let mut h = Harness::new();
    let area = build_overflowing_area(&mut h);
    let indicator = vertical_indicator(&h, area);
    let content_child = {
        // Any deep descendant stands in for the widget under the cursor.
        let mut handle = area;
        while let Some(next) = h.ui.node(handle).children().first().copied() {
            handle = next;
        }
        handle
    };

    h.pump();
    assert!(
        !thumb_visible(&h, indicator),
        "overlay thumbs start hidden (Fluent overlay model)"
    );

    // Pointer enters the area over a content node: thumbs appear.
    send_widget(&mut h, content_child, WidgetMessage::MouseEnter);
    h.pump();
    assert!(
        thumb_visible(&h, indicator),
        "hover anywhere over the area must reveal the thumb"
    );

    // Pointer leaves: thumbs hide again.
    send_widget(&mut h, content_child, WidgetMessage::MouseLeave);
    h.pump();
    assert!(
        !thumb_visible(&h, indicator),
        "leaving the area must hide the thumb"
    );

    // Wheel scrolling without pointer movement also reveals them.
    scroll(&mut h, area, ScrollViewerMessage::VerticalScroll(10.0));
    h.pump();
    assert!(
        thumb_visible(&h, indicator),
        "scrolling must reveal the thumb"
    );
}
