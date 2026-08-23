//! Functional tests for the LoadingIndicator component.

mod common;

use common::Harness;
use fyrox::graph::SceneGraph;
use raikou_core::Length;
use raikou_widgets::loading_indicator::LoadingIndicatorControl;
use raikou_widgets::{LoadingIndicator, LoadingIndicatorMode};

fn control_of(h: &Harness, handle: fyrox::core::pool::Handle<fyrox::gui::UiNode>) -> LoadingIndicatorControl {
    h.ui
        .try_get_of_type::<LoadingIndicatorControl>(handle)
        .unwrap()
        .clone()
}

#[test]
fn default_color_is_theme_accent() {
    let mut h = Harness::new();
    let component = h.build(|cx| LoadingIndicator::new().build(cx));
    let control = control_of(&h, component.handle);

    let accent = h
        .theme
        .color("accent.solid")
        .expect("theme must define accent.solid");
    let expected = fyrox::core::color::Color::from_rgba(
        (accent.red * 255.0).round() as u8,
        (accent.green * 255.0).round() as u8,
        (accent.blue * 255.0).round() as u8,
        (accent.alpha * 255.0).round() as u8,
    );
    assert_eq!(
        control.color(),
        expected,
        "unset color must resolve to the theme accent token"
    );
}

#[test]
fn explicit_color_overrides_accent() {
    let mut h = Harness::new();
    let color = raikou_core::Color::new(0.9, 0.2, 0.1, 1.0);
    let component = h.build(|cx| {
        LoadingIndicator::new()
            .color(color)
            .build(cx)
    });
    let control = control_of(&h, component.handle);

    let round = |v: f32| (v * 255.0).round() as u8;
    assert_eq!(control.color().r, round(0.9));
    assert_eq!(control.color().g, round(0.2));
    assert_eq!(control.color().b, round(0.1));
    assert_eq!(control.color().a, 255);
}

#[test]
fn mode_and_size_apply() {
    let mut h = Harness::new();
    let component = h.build(|cx| {
        LoadingIndicator::new()
            .mode(LoadingIndicatorMode::Ring)
            .size(40.0)
            .stroke_width(3.0)
            .speed_ratio(2.0)
            .is_active(false)
            .build(cx)
    });
    let control = control_of(&h, component.handle);

    assert_eq!(control.mode(), LoadingIndicatorMode::Ring);
    assert!(!control.is_active());
}

#[test]
fn bar_mode_animates_over_time() {
    let mut h = Harness::new();
    let component = h.build(|cx| {
        LoadingIndicator::new()
            .mode(LoadingIndicatorMode::Bar)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(8.0))
            .stroke_width(4.0)
            .build(cx)
    });

    let before = control_of(&h, component.handle).animation_time();
    // Several update ticks must advance the animation clock.
    for _ in 0..5 {
        h.update_and_pump();
    }
    let after = control_of(&h, component.handle).animation_time();

    assert_eq!(before, 0.0, "clock starts at zero");
    assert!(after > 0.0, "update ticks must advance the Bar animation");
}
