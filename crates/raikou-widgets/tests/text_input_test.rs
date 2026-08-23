//! Functional tests for TextInput and TextArea.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
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

fn type_text(h: &mut Harness, inner: fyrox::core::pool::Handle<fyrox::gui::UiNode>, s: &str) {
    use fyrox::gui::text_box::TextBox;
    let before = h
        .ui
        .try_get_of_type::<TextBox>(inner)
        .unwrap()
        .text();
    h.ui.send(inner, TextMessage::Text(format!("{}{}", before, s)));
    h.pump();
}

#[test]
fn text_input_undo_redo_restores_text() {
    use fyrox::graph::SceneGraph;
    use fyrox::gui::message::{KeyCode, KeyboardModifiers, OsEvent};
    use fyrox::gui::text_box::TextBox;
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let input = h.build(|cx| TextInput::new().build(cx));
    let inner = h.ui.node(input.handle).children()[0];

    // Two edits: "" -> "hello" -> "hello world".
    type_text(&mut h, inner, "hello");
    type_text(&mut h, inner, " world");
    let tb = h.ui.try_get_of_type::<TextBox>(inner).unwrap();
    assert_eq!(tb.text(), "hello world");

    // Hold Ctrl and press Z twice: back to "hello", then to "".
    h.ui.process_os_event(&OsEvent::KeyboardModifiers(KeyboardModifiers {
        alt: false,
        shift: false,
        control: true,
        system: false,
    }));
    h.ui.send(inner, WidgetMessage::KeyDown(KeyCode::KeyZ));
    h.pump();
    let tb = h.ui.try_get_of_type::<TextBox>(inner).unwrap();
    assert_eq!(tb.text(), "hello", "first undo must restore 'hello'");

    h.ui.send(inner, WidgetMessage::KeyDown(KeyCode::KeyZ));
    h.pump();
    let tb = h.ui.try_get_of_type::<TextBox>(inner).unwrap();
    assert_eq!(tb.text(), "", "second undo must restore empty text");

    // Ctrl+Y redoes one step.
    h.ui.send(inner, WidgetMessage::KeyDown(KeyCode::KeyY));
    h.pump();
    let tb = h.ui.try_get_of_type::<TextBox>(inner).unwrap();
    assert_eq!(tb.text(), "hello", "redo must reapply 'hello'");
}

#[test]
fn text_input_double_click_selects_word() {
    use fyrox::graph::SceneGraph;
    use fyrox::gui::message::MouseButton;
    use fyrox::gui::text_box::{SelectionRange, TextBox};
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let input = h.build(|cx| TextInput::new().text("hello world").build(cx));
    let inner = h.ui.node(input.handle).children()[0];

    let click = WidgetMessage::MouseDown {
        pos: fyrox::core::algebra::Vector2::new(30.0, 15.0),
        button: MouseButton::Left,
    };
    h.ui.send(inner, click.clone());
    h.update_and_pump();
    h.ui.send(inner, click);
    h.update_and_pump();

    let range = *h
        .ui
        .try_get_of_type::<TextBox>(inner)
        .unwrap()
        .selection_range;
    assert!(
        matches!(range, Some(SelectionRange { begin, end }) if end != begin),
        "double click must select a word, got {range:?}"
    );
}

#[test]
fn text_input_focus_ring_accents_chrome() {
    use fyrox::graph::SceneGraph;
    use fyrox::gui::border::Border;
    use fyrox::gui::brush::Brush;
    use fyrox::gui::widget::WidgetMessage;

    fn solid_u8(brush: &Brush) -> Option<[u8; 3]> {
        match brush {
            Brush::Solid(c) => Some([c.r, c.g, c.b]),
            _ => None,
        }
    }

    let mut h = Harness::new();
    let input = h.build(|cx| TextInput::new().placeholder("Focus me").build(cx));
    let inner = h.ui.node(input.handle).children()[0];

    let accent = h.theme.color("accent.solid").unwrap();
    let stroke = h.theme.color("border.default").unwrap();
    let to_u8 = |c: raikou_core::Color| -> [u8; 3] {
        [
            (c.red * 255.0).round() as u8,
            (c.green * 255.0).round() as u8,
            (c.blue * 255.0).round() as u8,
        ]
    };

    // Focus arriving via a descendant of the field must accent the chrome
    // (real focus lands on the deepest picked node).
    h.ui.send(inner, WidgetMessage::Focus);
    h.pump();
    let brush = h.ui.try_get_of_type::<Border>(input.handle).unwrap();
    let got = solid_u8(&**brush.widget.foreground);
    assert_eq!(got, Some(to_u8(accent)), "focused chrome must use accent");

    // Focus moving outside the field must revert the chrome.
    let root = h.ui.root();
    h.ui.send(root, WidgetMessage::Focus);
    h.pump();
    let brush = h.ui.try_get_of_type::<Border>(input.handle).unwrap();
    let got = solid_u8(&**brush.widget.foreground);
    assert_eq!(got, Some(to_u8(stroke)), "unfocused chrome must revert");
}
