//! Functional tests for TextInput and TextArea.

mod common;

use common::{Counter, Harness};
use fyrox::gui::text::TextMessage;
use raikou_widgets::{TextArea, TextInput};

#[test]
fn text_input_reports_text_changes() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let s = seen.clone();
    let input = h.build(move |cx| {
        TextInput::new()
            .placeholder("Type here")
            .on_change(move |_, v| {
                s.replace(v.to_string());
            })
            .build(cx)
    });

    h.ui.send(input.handle, TextMessage::Text("hello".into()));
    h.pump();
    assert_eq!(
        seen.borrow().as_str(),
        "hello",
        "TextMessage::Text must forward to on_change"
    );

    h.ui.send(input.handle, TextMessage::Text("world".into()));
    h.pump();
    assert_eq!(seen.borrow().as_str(), "world");
}

#[test]
fn text_area_reports_text_changes() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let s = seen.clone();
    let area = h.build(move |cx| {
        TextArea::new()
            .rows(4)
            .on_change(move |_, v| {
                s.replace(v.to_string());
            })
            .build(cx)
    });

    h.ui.send(area.handle, TextMessage::Text("line1\nline2".into()));
    h.pump();
    assert_eq!(seen.borrow().as_str(), "line1\nline2");
}
