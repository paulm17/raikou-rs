use raikou_core::WindowId;
use raikou_window::events::{
    KeyState, PointerButton, PointerEventKind, RuntimeEvent, WindowEventTranslator,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
};

fn one(event: WindowEvent) -> RuntimeEvent {
    let mut translator = WindowEventTranslator::new();
    let mut translated = translator.translate(WindowId::next(), &event);
    assert_eq!(translated.len(), 1, "expected exactly one translated event");
    translated.remove(0)
}

#[test]
fn resized_event_maps_size() {
    match one(WindowEvent::Resized(PhysicalSize::new(1920u32, 1080u32))) {
        RuntimeEvent::Resize(event) => {
            assert_eq!(event.size.width, 1920);
            assert_eq!(event.size.height, 1080);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn cursor_moved_maps_position() {
    let event = WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position: PhysicalPosition::new(42.5_f64, 100.75_f64),
    };

    match one(event) {
        RuntimeEvent::Pointer(pointer) => {
            assert_eq!(pointer.position, Some((42.5, 100.75)));
            assert_eq!(pointer.kind, PointerEventKind::Move);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn left_button_press_maps_to_button_event() {
    let event = WindowEvent::MouseInput {
        device_id: winit::event::DeviceId::dummy(),
        state: ElementState::Pressed,
        button: MouseButton::Left,
    };

    match one(event) {
        RuntimeEvent::Pointer(pointer) => {
            assert_eq!(
                pointer.kind,
                PointerEventKind::Button {
                    state: KeyState::Down,
                    button: PointerButton::Left,
                }
            );
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn wheel_maps_scroll_delta() {
    let event = WindowEvent::MouseWheel {
        device_id: winit::event::DeviceId::dummy(),
        delta: MouseScrollDelta::LineDelta(1.0, -3.0),
        phase: TouchPhase::Moved,
    };

    match one(event) {
        RuntimeEvent::Pointer(pointer) => match pointer.kind {
            PointerEventKind::Wheel(delta) => {
                assert_eq!(
                    delta,
                    raikou_window::events::PointerScrollDelta::Line { x: 1.0, y: -3.0 }
                );
            }
            other => panic!("wrong pointer kind: {other:?}"),
        },
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn focus_maps_to_framework_focus_event() {
    match one(WindowEvent::Focused(true)) {
        RuntimeEvent::Focus(event) => assert!(event.focused),
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn close_requested_maps_to_framework_close_event() {
    match one(WindowEvent::CloseRequested) {
        RuntimeEvent::CloseRequested(_) => {}
        other => panic!("wrong variant: {other:?}"),
    }
}

#[test]
fn redraw_requested_maps_to_framework_redraw_event() {
    match one(WindowEvent::RedrawRequested) {
        RuntimeEvent::RedrawRequested(_) => {}
        other => panic!("wrong variant: {other:?}"),
    }
}
