use raikou_core::WindowId;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent as WinitWindowEvent,
};
use winit::keyboard::{Key, ModifiersState, NamedKey, PhysicalKey};

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEvent {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    TextInput(TextInputEvent),
    Resize(ResizeEvent),
    ScaleFactorChanged(ScaleFactorChangedEvent),
    Focus(FocusEvent),
    CloseRequested(WindowId),
    RedrawRequested(RedrawEvent),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerEvent {
    pub window_id: WindowId,
    pub position: Option<(f64, f64)>,
    pub kind: PointerEventKind,
    pub modifiers: Modifiers,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PointerEventKind {
    Move,
    Enter,
    Leave,
    Button {
        state: KeyState,
        button: PointerButton,
    },
    Wheel(PointerScrollDelta),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PointerScrollDelta {
    Line { x: f32, y: f32 },
    Pixel { x: f64, y: f64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardEvent {
    pub window_id: WindowId,
    pub state: KeyState,
    pub repeat: bool,
    pub logical_key: String,
    pub physical_key: String,
    pub text: Option<String>,
    pub modifiers: Modifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextInputEvent {
    Text {
        window_id: WindowId,
        text: String,
    },
    Ime {
        window_id: WindowId,
        event: ImeEvent,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    Enabled,
    Disabled,
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeEvent {
    pub window_id: WindowId,
    pub size: PhysicalSize<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactorChangedEvent {
    pub window_id: WindowId,
    pub scale_factor: f64,
    pub suggested_inner_size: Option<PhysicalSize<u32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusEvent {
    pub window_id: WindowId,
    pub focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedrawEvent {
    pub window_id: WindowId,
}

#[derive(Debug, Default)]
pub struct WindowEventTranslator {
    modifiers: Modifiers,
    last_pointer_position: Option<(f64, f64)>,
}

impl WindowEventTranslator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    pub fn translate(
        &mut self,
        window_id: WindowId,
        event: &WinitWindowEvent,
    ) -> Vec<RuntimeEvent> {
        match event {
            WinitWindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = map_modifiers(modifiers.state());
                Vec::new()
            }
            WinitWindowEvent::Resized(size) => {
                vec![RuntimeEvent::Resize(ResizeEvent {
                    window_id,
                    size: *size,
                })]
            }
            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => vec![RuntimeEvent::ScaleFactorChanged(ScaleFactorChangedEvent {
                window_id,
                scale_factor: *scale_factor,
                suggested_inner_size: None,
            })],
            WinitWindowEvent::Focused(focused) => {
                vec![RuntimeEvent::Focus(FocusEvent {
                    window_id,
                    focused: *focused,
                })]
            }
            WinitWindowEvent::CloseRequested => vec![RuntimeEvent::CloseRequested(window_id)],
            WinitWindowEvent::RedrawRequested => {
                vec![RuntimeEvent::RedrawRequested(RedrawEvent { window_id })]
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                let position = (position.x, position.y);
                self.last_pointer_position = Some(position);
                vec![RuntimeEvent::Pointer(PointerEvent {
                    window_id,
                    position: Some(position),
                    kind: PointerEventKind::Move,
                    modifiers: self.modifiers,
                })]
            }
            WinitWindowEvent::CursorEntered { .. } => {
                vec![RuntimeEvent::Pointer(PointerEvent {
                    window_id,
                    position: self.last_pointer_position,
                    kind: PointerEventKind::Enter,
                    modifiers: self.modifiers,
                })]
            }
            WinitWindowEvent::CursorLeft { .. } => {
                vec![RuntimeEvent::Pointer(PointerEvent {
                    window_id,
                    position: self.last_pointer_position,
                    kind: PointerEventKind::Leave,
                    modifiers: self.modifiers,
                })]
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
                vec![RuntimeEvent::Pointer(PointerEvent {
                    window_id,
                    position: self.last_pointer_position,
                    kind: PointerEventKind::Button {
                        state: map_key_state(*state),
                        button: map_pointer_button(*button),
                    },
                    modifiers: self.modifiers,
                })]
            }
            WinitWindowEvent::MouseWheel { delta, .. } => {
                vec![RuntimeEvent::Pointer(PointerEvent {
                    window_id,
                    position: self.last_pointer_position,
                    kind: PointerEventKind::Wheel(map_scroll_delta(*delta)),
                    modifiers: self.modifiers,
                })]
            }
            WinitWindowEvent::KeyboardInput {
                event,
                is_synthetic: _,
                ..
            } => {
                let mut translated = vec![RuntimeEvent::Keyboard(KeyboardEvent {
                    window_id,
                    state: map_key_state(event.state),
                    repeat: event.repeat,
                    logical_key: map_logical_key(&event.logical_key),
                    physical_key: map_physical_key(event.physical_key),
                    text: event.text.as_ref().map(ToString::to_string),
                    modifiers: self.modifiers,
                })];

                if event.state == ElementState::Pressed
                    && let Some(text) = event.text.as_ref()
                    && !text.is_empty()
                    && !text.chars().all(|c| c.is_control())
                {
                    translated.push(RuntimeEvent::TextInput(TextInputEvent::Text {
                        window_id,
                        text: text.to_string(),
                    }));
                }

                translated
            }
            WinitWindowEvent::Ime(ime) => vec![RuntimeEvent::TextInput(TextInputEvent::Ime {
                window_id,
                event: map_ime_event(ime),
            })],
            _ => Vec::new(),
        }
    }
}

fn map_key_state(state: ElementState) -> KeyState {
    match state {
        ElementState::Pressed => KeyState::Down,
        ElementState::Released => KeyState::Up,
    }
}

fn map_pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(value) => PointerButton::Other(value),
    }
}

fn map_scroll_delta(delta: MouseScrollDelta) -> PointerScrollDelta {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => PointerScrollDelta::Line { x, y },
        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
            PointerScrollDelta::Pixel { x, y }
        }
    }
}

fn map_modifiers(modifiers: ModifiersState) -> Modifiers {
    Modifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        super_key: modifiers.super_key(),
    }
}

fn map_logical_key(key: &Key) -> String {
    match key {
        Key::Named(named) => map_named_key(*named).to_string(),
        Key::Character(value) => value.to_string(),
        Key::Dead(value) => value
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| "Dead".to_string()),
        Key::Unidentified(_) => "Unidentified".to_string(),
    }
}

fn map_named_key(key: NamedKey) -> &'static str {
    match key {
        NamedKey::Enter => "Enter",
        NamedKey::Tab => "Tab",
        NamedKey::Space => "Space",
        NamedKey::ArrowDown => "ArrowDown",
        NamedKey::ArrowLeft => "ArrowLeft",
        NamedKey::ArrowRight => "ArrowRight",
        NamedKey::ArrowUp => "ArrowUp",
        NamedKey::Escape => "Escape",
        NamedKey::Backspace => "Backspace",
        _ => "Named",
    }
}

fn map_physical_key(key: PhysicalKey) -> String {
    format!("{key:?}")
}

fn map_ime_event(event: &Ime) -> ImeEvent {
    match event {
        Ime::Enabled => ImeEvent::Enabled,
        Ime::Disabled => ImeEvent::Disabled,
        Ime::Commit(text) => ImeEvent::Commit(text.clone()),
        Ime::Preedit(text, cursor) => ImeEvent::Preedit {
            text: text.clone(),
            cursor: *cursor,
        },
    }
}