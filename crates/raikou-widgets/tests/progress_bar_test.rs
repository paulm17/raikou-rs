//! Functional tests for the ProgressBar component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use fyrox::gui::progress_bar::ProgressBar as FyroxProgressBar;
use raikou_core::Length;
use raikou_widgets::{set_progress, ProgressBar};

#[test]
fn progress_bar_clamps_and_applies_progress() {
    let mut h = Harness::new();
    let bar = h.build(|cx| {
        ProgressBar::new()
            .value(0.25)
            .width(Length::Fixed(200.0))
            .height(4.0)
            .build(cx)
    });

    set_progress(&h.ui, bar.handle, 0.7);
    h.update_and_pump();
    let widget =
        h.ui.try_get_of_type::<FyroxProgressBar>(bar.handle)
            .unwrap();
    assert_eq!(*widget.progress, 0.7, "set_progress must apply the value");

    // Values outside 0..1 must clamp.
    set_progress(&h.ui, bar.handle, 1.5);
    h.update_and_pump();
    let widget =
        h.ui.try_get_of_type::<FyroxProgressBar>(bar.handle)
            .unwrap();
    assert_eq!(*widget.progress, 1.0, "progress must clamp to 1.0");

    set_progress(&h.ui, bar.handle, -0.5);
    h.update_and_pump();
    let widget =
        h.ui.try_get_of_type::<FyroxProgressBar>(bar.handle)
            .unwrap();
    assert_eq!(*widget.progress, 0.0, "progress must clamp to 0.0");
}
