use raikou_skia::SkiaRenderer;
use raikou_text::{
    Action, Attrs, Color, Cursor, Family, FontSystem, Metrics, Motion, Selection, SwashCache,
    TextBuffer,
};
use raikou_window::{
    KeyState, Modifiers, PointerButton, PointerEventKind, PointerScrollDelta, RuntimeEvent,
    RuntimeLifecycle, TextInputEvent, WindowConfig, WindowRuntime,
};
use std::cell::RefCell;
use std::rc::Rc;

struct TextEditorApp {
    renderer: Option<SkiaRenderer>,
    font_system: FontSystem,
    swash_cache: Rc<RefCell<SwashCache>>,
    buffer: TextBuffer,
    width: u32,
    height: u32,
    blink_on: bool,
    blink_timer: usize,
    dragging: bool,
}

impl TextEditorApp {
    fn new() -> Self {
        let mut font_system = FontSystem::new()
            .with_default_size(14.0)
            .with_default_line_height(20.0);
        let metrics = Metrics::new(14.0, 20.0);
        let mut buffer = TextBuffer::with_metrics(&mut font_system, metrics);

        let attrs = Attrs::new().family(Family::Monospace);
        buffer.set_text(
            &mut font_system,
            "Hello, Raikou Text Editor!\n\
             \n\
             This example showcases raikou-text features:\n\
             \n\
             - Typing and text insertion\n\
             - Arrow key navigation (Left/Right/Up/Down)\n\
             - Home / End (line start/end)\n\
             - PageUp / PageDown\n\
             - Ctrl+A to select all\n\
             - Shift+Arrow for selection\n\
             - Backspace / Delete\n\
             - Enter for new lines\n\
             - Click to position cursor\n\
             - Shift+Click to extend selection\n\
             - Blinking caret\n\
             - Selection highlighting\n\
             - Word wrap\n\
             - Scroll with mouse wheel\n\
             - Escape to clear selection",
            &attrs,
        );

        Self {
            renderer: None,
            font_system,
            swash_cache: Rc::new(RefCell::new(SwashCache::new())),
            buffer,
            width: 800,
            height: 600,
            blink_on: true,
            blink_timer: 0,
            dragging: false,
        }
    }

    fn start_selection_if_none(&mut self) {
        if self.buffer.selection() == Selection::None {
            self.buffer
                .set_selection(Selection::Normal(self.buffer.cursor()));
        }
    }

    fn reset_blink(&mut self) {
        self.blink_on = true;
        self.blink_timer = 0;
    }

    fn handle_keyboard(&mut self, key: &str, modifiers: &Modifiers) {
        if modifiers.control {
            match key {
                "a" => self.buffer.select_all(),
                "x" => {
                    let _ = self.buffer.copy_selection();
                    self.buffer.delete_selection(&mut self.font_system);
                }
                "c" => {
                    let _ = self.buffer.copy_selection();
                }
                "v" => {
                    // Clipboard not available through winit Window directly.
                    // A clipboard backend integration is needed.
                }
                _ => return,
            }
            self.reset_blink();
            return;
        }

        if modifiers.shift && !matches!(key, "Backspace" | "Delete" | "Enter" | "Tab" | "Escape") {
            match key {
                "ArrowLeft" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::Left));
                }
                "ArrowRight" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::Right));
                }
                "ArrowUp" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::Up));
                }
                "ArrowDown" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::Down));
                }
                "Home" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::Home));
                }
                "End" => {
                    self.start_selection_if_none();
                    self.buffer
                        .action(&mut self.font_system, Action::Motion(Motion::End));
                }
                _ => return,
            }
            self.reset_blink();
            return;
        }

        match key {
            "ArrowLeft" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::Left)),
            "ArrowRight" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::Right)),
            "ArrowUp" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::Up)),
            "ArrowDown" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::Down)),
            "Home" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::Home)),
            "End" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::End)),
            "PageUp" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::PageUp)),
            "PageDown" => self
                .buffer
                .action(&mut self.font_system, Action::Motion(Motion::PageDown)),
            "Backspace" => self.buffer.action(&mut self.font_system, Action::Backspace),
            "Delete" => self.buffer.action(&mut self.font_system, Action::Delete),
            "Enter" => self.buffer.action(&mut self.font_system, Action::Enter),
            "Escape" => {
                self.buffer.set_selection(Selection::None);
            }
            "Tab" => self.buffer.insert_string(&mut self.font_system, "    "),
            _ => return,
        }
        self.reset_blink();
    }

    fn draw_editor(
        font_system: &mut FontSystem,
        swash_cache: &Rc<RefCell<SwashCache>>,
        buffer: &TextBuffer,
        canvas: &skia_safe::Canvas,
        blink_on: bool,
    ) {
        canvas.clear(skia_safe::Color::from_argb(255, 30, 30, 36));

        let selection_rects = buffer.selection_rects();
        for rect in &selection_rects {
            let mut paint = skia_safe::Paint::default();
            paint.set_anti_alias(false);
            paint.set_color(skia_safe::Color::from_argb(255, 50, 70, 110));
            canvas.draw_rect(
                skia_safe::Rect::from_xywh(rect.x(), rect.y(), rect.width(), rect.height()),
                &paint,
            );
        }

        {
            let mut swash = swash_cache.borrow_mut();
            for run in buffer.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((0.0, run.line_y), 1.0);
                    let color = glyph.color_opt.unwrap_or(Color::rgb(212, 190, 152));

                    swash.with_pixels(
                        font_system.inner_mut(),
                        physical.cache_key,
                        color,
                        |px, py, pixel_color| {
                            let x = (physical.x + px) as f32;
                            let y = (physical.y + py) as f32;
                            let mut paint = skia_safe::Paint::default();
                            paint.set_anti_alias(false);
                            paint.set_color(skia_safe::Color::from_argb(
                                pixel_color.a(),
                                pixel_color.r(),
                                pixel_color.g(),
                                pixel_color.b(),
                            ));
                            canvas.draw_rect(skia_safe::Rect::from_xywh(x, y, 1.0, 1.0), &paint);
                        },
                    );
                }
            }
        }

        if blink_on
            && let Some(caret) = buffer.caret_rect()
        {
            let mut paint = skia_safe::Paint::default();
            paint.set_anti_alias(false);
            paint.set_color(skia_safe::Color::from_argb(255, 220, 200, 160));
            canvas.draw_rect(
                skia_safe::Rect::from_xywh(caret.x(), caret.y(), 2.0, caret.height()),
                &paint,
            );
        }
    }
}

impl RuntimeLifecycle for TextEditorApp {
    fn on_start(&mut self, runtime: &mut WindowRuntime) {
        let window = runtime.window().expect("window not created");
        self.renderer = Some(SkiaRenderer::new(window).expect("failed to create renderer"));
        self.buffer.set_size(
            &mut self.font_system,
            Some(self.width as f32),
            Some(self.height as f32),
        );
        runtime.request_redraw();
    }

    fn on_event(&mut self, runtime: &mut WindowRuntime, event: &RuntimeEvent) {
        match event {
            RuntimeEvent::Resize(resize) => {
                self.width = resize.size.width;
                self.height = resize.size.height;
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(resize.size, runtime.primary_window().scale_factor());
                }
                self.buffer.set_size(
                    &mut self.font_system,
                    Some(self.width as f32),
                    Some(self.height as f32),
                );
                runtime.request_redraw();
            }
            RuntimeEvent::RedrawRequested(_) => {
                self.blink_timer += 1;
                if self.blink_timer >= 30 {
                    self.blink_timer = 0;
                    self.blink_on = !self.blink_on;
                }

                if let Some(renderer) = &mut self.renderer {
                    let mut frame = renderer.begin_frame().expect("failed to begin frame");
                    Self::draw_editor(
                        &mut self.font_system,
                        &self.swash_cache,
                        &self.buffer,
                        frame.canvas(),
                        self.blink_on,
                    );
                    frame.present().expect("failed to present frame");
                }
                runtime.request_redraw();
            }
            RuntimeEvent::Keyboard(keyboard) => {
                if keyboard.state == KeyState::Down {
                    self.handle_keyboard(&keyboard.logical_key, &keyboard.modifiers);
                    self.reset_blink();
                    runtime.request_redraw();
                }
            }
            RuntimeEvent::TextInput(TextInputEvent::Text { text, .. }) => {
                self.buffer.insert_string(&mut self.font_system, text);
                self.reset_blink();
                runtime.request_redraw();
            }
            RuntimeEvent::Pointer(pointer) => match &pointer.kind {
                PointerEventKind::Button {
                    state: KeyState::Down,
                    button: PointerButton::Left,
                } => {
                    if let Some((x, y)) = pointer.position {
                        if pointer.modifiers.shift {
                            let old_cursor = self.buffer.cursor();
                            if let Some(new_cursor) =
                                self.buffer.hit(&mut self.font_system, x as f32, y as f32)
                            {
                                self.buffer.set_cursor(old_cursor);
                                self.buffer.set_selection(Selection::Normal(Cursor::new(
                                    new_cursor.line,
                                    new_cursor.index,
                                )));
                                self.buffer.set_cursor(new_cursor);
                            }
                        } else {
                            let _ = self.buffer.hit(&mut self.font_system, x as f32, y as f32);
                        }
                        self.dragging = true;
                        self.reset_blink();
                        runtime.request_redraw();
                    }
                }
                PointerEventKind::Button {
                    state: KeyState::Up,
                    button: PointerButton::Left,
                } => {
                    self.dragging = false;
                }
                PointerEventKind::Move => {
                    if self.dragging
                        && let Some((x, y)) = pointer.position
                    {
                        self.buffer.drag(&mut self.font_system, x as f32, y as f32);
                        self.reset_blink();
                        runtime.request_redraw();
                    }
                }
                PointerEventKind::Wheel(delta) => {
                    let pixels = match delta {
                        PointerScrollDelta::Line { y, .. } => y * 20.0,
                        PointerScrollDelta::Pixel { y, .. } => *y as f32,
                    };
                    if pixels != 0.0 {
                        self.buffer
                            .action(&mut self.font_system, Action::Scroll { pixels: -pixels });
                        runtime.request_redraw();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    let config = WindowConfig::default()
        .title("Raikou Text Editor")
        .initial_size(800.0, 600.0);

    let runtime = WindowRuntime::new(config);
    runtime.run(TextEditorApp::new()).unwrap();
}
