use raikou_core::{Color, Rect, RoundedRect};
use raikou_skia::SkiaRenderer;
use raikou_window::{RuntimeEvent, RuntimeLifecycle, WindowConfig, WindowRuntime};

struct App {
    renderer: Option<SkiaRenderer>,
}

impl App {
    fn new() -> Self {
        Self { renderer: None }
    }
}

impl RuntimeLifecycle for App {
    fn on_start(&mut self, runtime: &mut WindowRuntime) {
        let window = runtime.window().expect("window not created");
        self.renderer = Some(SkiaRenderer::new(window).expect("failed to create renderer"));
        runtime.request_redraw();
    }

    fn on_event(
        &mut self,
        runtime: &mut WindowRuntime,
        event: &RuntimeEvent,
    ) {
        match event {
            RuntimeEvent::Resize(resize) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(resize.size, runtime.primary_window().scale_factor());
                }
                runtime.request_redraw();
            }
            RuntimeEvent::RedrawRequested(_) => {
                if let Some(renderer) = &mut self.renderer {
                    let mut frame = renderer.begin_frame().expect("failed to begin frame");
                    let painter = frame.painter();

                    painter.clear(Color::new(0.1, 0.1, 0.15, 1.0));

                    let rect = Rect::from_xywh(100.0, 100.0, 200.0, 150.0);
                    let rounded = RoundedRect::from_rect_xy(rect, 20.0);
                    painter.fill_rounded_rect(rounded, Color::new(0.3, 0.5, 0.8, 1.0));

                    let rect2 = Rect::from_xywh(350.0, 200.0, 150.0, 100.0);
                    let rounded2 = RoundedRect::from_rect_xy(rect2, 10.0);
                    painter.stroke_rounded_rect(rounded2, Color::new(0.8, 0.2, 0.4, 1.0), 3.0);

                    frame.present().expect("failed to present frame");
                }
                runtime.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let config = WindowConfig::default()
        .title("Raikou Skia")
        .initial_size(800.0, 600.0);

    let runtime = WindowRuntime::new(config);
    runtime.run(App::new()).unwrap();
}
