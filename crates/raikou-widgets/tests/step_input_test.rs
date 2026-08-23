//! Functional tests for the StepInput component.

mod common;

use common::{Counter, Harness};
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::numeric::NumericUpDownMessage;
use raikou_widgets::StepInput;

#[test]
fn step_input_reports_value_changes() {
    let mut h = Harness::new();
    let seen = std::rc::Rc::new(std::cell::Cell::new(f64::NAN));
    let s = seen.clone();
    let input = h.build(move |cx| {
        StepInput::new()
            .value(1.0)
            .min(0.0)
            .max(10.0)
            .step(0.5)
            .on_change(move |_, v| s.set(v))
            .build(cx)
    });

    // FromWidget value reports (what the widget emits on user interaction)
    // must reach the callback.
    h.ui.send_message(
        UiMessage::with_data(NumericUpDownMessage::<f64>::Value(2.5))
            .with_destination(input.handle)
            .with_direction(MessageDirection::FromWidget),
    );
    h.pump();
    assert_eq!(seen.get(), 2.5, "FromWidget Value must fire on_change");
}

#[test]
fn step_input_ignores_to_widget_commands() {
    let mut h = Harness::new();
    let changes = Counter::new();
    let c = changes.clone();
    let input = h.build(move |cx| StepInput::new().on_change(move |_, _| c.bump()).build(cx));

    // A ToWidget command is applied by the widget, which echoes one
    // FromWidget report — exactly one callback, not two.
    h.ui.send(input.handle, NumericUpDownMessage::<f64>::Value(3.0));
    h.pump();
    assert_eq!(changes.get(), 1, "programmatic set must fire exactly once");

    // Setting the same value again is a no-op: no echo, no callback.
    h.ui.send(input.handle, NumericUpDownMessage::<f64>::Value(3.0));
    h.pump();
    assert_eq!(changes.get(), 1, "no-op set must not fire on_change");
}

#[test]
fn step_input_spinner_repeats_on_hold() {
    use fyrox::core::algebra::Vector2;
    use fyrox::graph::SceneGraph;
    use fyrox::gui::button::Button;
    use fyrox::gui::numeric::NumericUpDown;
    use fyrox::gui::widget::WidgetMessage;

    let mut h = Harness::new();
    let input = h.build(|cx| StepInput::new().value(10.0).step(1.0).build(cx));

    // Locate the inner numeric and its increase button.
    let mut stack = vec![input.handle];
    let mut inner = None;
    while let Some(node) = stack.pop() {
        if let Ok(n) = h.ui.try_get_of_type::<NumericUpDown<f64>>(node) {
            inner = Some((*n.increase).to_base());
            break;
        }
        for c in h.ui.node(node).children() {
            stack.push(*c);
        }
    }
    let increase: fyrox::core::pool::Handle<fyrox::gui::UiNode> =
        inner.expect("inner numeric updown");

    // Press and hold: the repeat timer fires a click per update tick once
    // the 0.1 s interval elapses.
    h.ui.send(
        increase,
        WidgetMessage::MouseDown {
            button: fyrox::gui::message::MouseButton::Left,
            pos: Vector2::new(4.0, 4.0),
        },
    );
    h.pump();
    // Pump after every tick like a real app frame: clicks posted from the
    // repeat timer reach the numeric updater before the next tick.
    for _ in 0..3 {
        h.ui.update(
            Vector2::new(800.0, 600.0),
            0.15,
            &fyrox::gui::UiUpdateSwitches::default(),
        );
        h.pump();
    }

    let value = *h
        .ui
        .try_get_of_type::<NumericUpDown<f64>>({
            let mut found = None;
            let mut st = vec![input.handle];
            while let Some(n) = st.pop() {
                if h.ui.try_get_of_type::<NumericUpDown<f64>>(n).is_ok() {
                    found = Some(n);
                    break;
                }
                for c in h.ui.node(n).children() {
                    st.push(*c);
                }
            }
            found.expect("inner numeric updown")
        })
        .unwrap()
        .value;
    assert_eq!(value, 13.0, "three hold ticks must step three times");
    assert!(
        h.ui.try_get_of_type::<Button>(increase).is_ok(),
        "spinner must be a real button"
    );
}
