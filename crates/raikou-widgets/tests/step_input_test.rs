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
