use raikou_core::{Color, Rect, RoundedRect, Size};
use raikou_layout::{
    Dock, DockPanel, HorizontalAlignment, LayoutContext, LayoutElement, Orientation, SizedBox,
    StackPanel, VerticalAlignment, arrange_element, measure_element,
};
use raikou_skia::SkiaRenderer;
use raikou_window::{RuntimeEvent, RuntimeLifecycle, WindowConfig, WindowRuntime};

struct App {
    renderer: Option<SkiaRenderer>,
    root: DockPanel,
    window_size: Size,
    font_system: raikou_layout::FontSystem,
}

impl App {
    fn new() -> Self {
        let mut root = DockPanel::new();
        root.last_child_fill = true;

        // ---- Header (40px, top) ----
        let mut header = StackPanel::new();
        let mut header_box = SizedBox::new(Size::new(0.0, 40.0));
        header_box.layout_mut().horizontal_alignment = HorizontalAlignment::Stretch;
        header.push_child(Box::new(header_box));
        header.layout_mut().attached.dock = Dock::Top;
        root.push_child(Box::new(header));

        // ---- Sidebar (150px, left) ----
        let mut sidebar = StackPanel::new();
        let mut sidebar_box = SizedBox::new(Size::new(150.0, 0.0));
        sidebar_box.layout_mut().vertical_alignment = VerticalAlignment::Stretch;
        sidebar.push_child(Box::new(sidebar_box));
        sidebar.layout_mut().attached.dock = Dock::Left;
        root.push_child(Box::new(sidebar));

        // ---- Main area (fills remainder) ----
        let mut main_area = StackPanel::new();
        main_area.orientation = Orientation::Vertical;
        main_area.spacing = 8.0;
        main_area.layout_mut().margin = raikou_core::Thickness::uniform(12.0);

        for _i in 1..=5 {
            let mut item = StackPanel::new();
            let mut item_box = SizedBox::new(Size::new(200.0, 48.0));
            item_box.layout_mut().horizontal_alignment = HorizontalAlignment::Stretch;
            item.push_child(Box::new(item_box));
            main_area.push_child(Box::new(item));
        }
        root.push_child(Box::new(main_area));

        Self {
            renderer: None,
            root,
            window_size: Size::new(800.0, 600.0),
            font_system: raikou_layout::FontSystem::new(),
        }
    }

    fn layout(&mut self) {
        let mut ctx = LayoutContext::new(&mut self.font_system);
        measure_element(&mut self.root, &mut ctx, self.window_size);
        arrange_element(
            &mut self.root,
            &mut ctx,
            Rect::from_xywh(0.0, 0.0, self.window_size.width, self.window_size.height),
        );
    }
}

fn paint(root: &DockPanel, painter: &raikou_skia::Painter<'_>) {
    painter.clear(Color::new(0.08, 0.09, 0.12, 1.0));
    let commands = raikou_layout::collect_paint_commands(root);
    let palette = [
        Color::new(0.15, 0.18, 0.24, 1.0),
        Color::new(0.12, 0.14, 0.18, 1.0),
        Color::new(0.3, 0.55, 0.85, 1.0),
        Color::new(0.25, 0.65, 0.45, 1.0),
        Color::new(0.85, 0.55, 0.25, 1.0),
    ];
    for cmd in commands {
        let bounds = cmd.element.layout().bounds();
        if bounds.size.width > 0.0 && bounds.size.height > 0.0 {
            let color = palette[cmd.depth % palette.len()];
            let rect = Rect::from_xywh(
                cmd.absolute_position.x,
                cmd.absolute_position.y,
                bounds.size.width,
                bounds.size.height,
            );
            let rounded = RoundedRect::from_rect_xy(rect, 6.0);
            painter.fill_rounded_rect(rounded, color);
        }
    }
}

impl RuntimeLifecycle for App {
    fn on_start(&mut self, runtime: &mut WindowRuntime) {
        let window = runtime.window().expect("window not created");
        self.renderer = Some(SkiaRenderer::new(window).expect("failed to create renderer"));
        runtime.request_redraw();
    }

    fn on_event(&mut self, runtime: &mut WindowRuntime, event: &RuntimeEvent) {
        match event {
            RuntimeEvent::Resize(resize) => {
                self.window_size = Size::new(resize.size.width as f32, resize.size.height as f32);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(resize.size, runtime.primary_window().scale_factor());
                }
                runtime.request_redraw();
            }
            RuntimeEvent::RedrawRequested(_) => {
                self.layout();
                if let Some(mut renderer) = self.renderer.take() {
                    let mut frame = renderer.begin_frame().expect("failed to begin frame");
                    paint(&self.root, &frame.painter());
                    frame.present().expect("failed to present frame");
                    self.renderer = Some(renderer);
                }
                runtime.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let config = WindowConfig::default()
        .title("Raikou Layout Demo")
        .initial_size(800.0, 600.0);

    let runtime = WindowRuntime::new(config);
    runtime.run(App::new()).unwrap();
}
