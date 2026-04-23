mod support;

use raikou_core::{Color, Rect, RoundedRect, Size};
use raikou_layout::{
    Canvas, ColumnDefinition, Dock, DockPanel, Grid, LayoutElement, OverlayLayer, RowDefinition,
    SizedBox, StackPanel, WrapPanel, arrange_element, measure_element,
};
use raikou_skia::Painter;
use skia_safe::surfaces;

#[test]
fn stack_panel_snapshot_matches() {
    let mut surface = render_stack_panel_surface();
    support::assert_surface_snapshot(&mut surface, "stack_panel.png");
}

#[test]
fn wrap_panel_snapshot_matches() {
    let mut surface = render_wrap_panel_surface();
    support::assert_surface_snapshot(&mut surface, "wrap_panel.png");
}

#[test]
fn dock_panel_snapshot_matches() {
    let mut surface = render_dock_panel_surface();
    support::assert_surface_snapshot(&mut surface, "dock_panel.png");
}

#[test]
fn grid_snapshot_matches() {
    let mut surface = render_grid_surface();
    support::assert_surface_snapshot(&mut surface, "grid.png");
}

#[test]
fn overlay_layer_snapshot_matches() {
    let mut surface = render_overlay_surface();
    support::assert_surface_snapshot(&mut surface, "overlay_layer.png");
}

#[test]
#[ignore]
fn regenerate_layout_snapshots() {
    let mut stack = render_stack_panel_surface();
    support::write_surface_snapshot(&mut stack, "stack_panel.png");
    let mut wrap = render_wrap_panel_surface();
    support::write_surface_snapshot(&mut wrap, "wrap_panel.png");
    let mut dock = render_dock_panel_surface();
    support::write_surface_snapshot(&mut dock, "dock_panel.png");
    let mut grid = render_grid_surface();
    support::write_surface_snapshot(&mut grid, "grid.png");
    let mut overlay = render_overlay_surface();
    support::write_surface_snapshot(&mut overlay, "overlay_layer.png");
}

fn render_stack_panel_surface() -> skia_safe::Surface {
    let mut panel = StackPanel::new();
    panel.spacing = 6.0;
    panel.push_child(Box::new(SizedBox::new(Size::new(90.0, 28.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(60.0, 20.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(110.0, 24.0))));
    render_panel(&mut panel, Size::new(160.0, 120.0))
}

fn render_wrap_panel_surface() -> skia_safe::Surface {
    let mut panel = WrapPanel::new();
    panel.item_spacing = 6.0;
    panel.line_spacing = 8.0;
    panel.push_child(Box::new(SizedBox::new(Size::new(50.0, 24.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(40.0, 30.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(70.0, 18.0))));
    panel.push_child(Box::new(SizedBox::new(Size::new(30.0, 22.0))));
    render_panel(&mut panel, Size::new(140.0, 100.0))
}

fn render_dock_panel_surface() -> skia_safe::Surface {
    let mut left = SizedBox::new(Size::new(26.0, 100.0));
    left.layout_mut().attached.dock = Dock::Left;
    let mut top = SizedBox::new(Size::new(90.0, 22.0));
    top.layout_mut().attached.dock = Dock::Top;
    let mut bottom = SizedBox::new(Size::new(90.0, 18.0));
    bottom.layout_mut().attached.dock = Dock::Bottom;

    let mut panel = DockPanel::new();
    panel.push_child(Box::new(left));
    panel.push_child(Box::new(top));
    panel.push_child(Box::new(bottom));
    panel.push_child(Box::new(SizedBox::new(Size::new(60.0, 40.0))));
    render_panel(&mut panel, Size::new(160.0, 110.0))
}

fn render_grid_surface() -> skia_safe::Surface {
    let mut a = SizedBox::new(Size::new(30.0, 26.0));
    a.layout_mut().attached.grid.column = 0;
    a.layout_mut().attached.grid.row = 0;
    let mut b = SizedBox::new(Size::new(50.0, 20.0));
    b.layout_mut().attached.grid.column = 1;
    b.layout_mut().attached.grid.row = 0;
    let mut c = SizedBox::new(Size::new(30.0, 40.0));
    c.layout_mut().attached.grid.column = 2;
    c.layout_mut().attached.grid.row = 1;
    let mut d = SizedBox::new(Size::new(60.0, 24.0));
    d.layout_mut().attached.grid.column = 0;
    d.layout_mut().attached.grid.row = 1;
    d.layout_mut().attached.grid.column_span = 2;

    let mut grid = Grid::new();
    grid.columns = vec![
        ColumnDefinition::pixel(40.0),
        ColumnDefinition::auto(),
        ColumnDefinition::star(1.0),
    ];
    grid.rows = vec![RowDefinition::auto(), RowDefinition::star(1.0)];
    grid.column_spacing = 4.0;
    grid.row_spacing = 4.0;
    grid.push_child(Box::new(a));
    grid.push_child(Box::new(b));
    grid.push_child(Box::new(c));
    grid.push_child(Box::new(d));
    render_panel(&mut grid, Size::new(170.0, 120.0))
}

fn render_overlay_surface() -> skia_safe::Surface {
    let mut content = Canvas::new();
    let mut content_a = SizedBox::new(Size::new(80.0, 50.0));
    content_a.layout_mut().attached.canvas.left = Some(16.0);
    content_a.layout_mut().attached.canvas.top = Some(20.0);
    let mut content_b = SizedBox::new(Size::new(60.0, 40.0));
    content_b.layout_mut().attached.canvas.left = Some(70.0);
    content_b.layout_mut().attached.canvas.top = Some(54.0);
    content.push_child(Box::new(content_a));
    content.push_child(Box::new(content_b));

    let mut overlay = OverlayLayer::new();
    let mut bubble = SizedBox::new(Size::new(90.0, 36.0));
    bubble.layout_mut().attached.canvas.left = Some(54.0);
    bubble.layout_mut().attached.canvas.top = Some(10.0);
    overlay.push_child(Box::new(bubble));

    let mut surface = surfaces::raster_n32_premul((180, 120)).expect("surface");
    let painter = Painter::new(surface.canvas());
    painter.clear(Color::new(1.0, 1.0, 1.0, 1.0));

    measure_element(&mut content, Size::new(180.0, 120.0));
    arrange_element(&mut content, Rect::from_xywh(0.0, 0.0, 180.0, 120.0));
    paint_canvas_like(&painter, &content, Color::new(0.3, 0.6, 0.9, 1.0));

    measure_element(&mut overlay, Size::new(180.0, 120.0));
    arrange_element(&mut overlay, Rect::from_xywh(0.0, 0.0, 180.0, 120.0));
    paint_overlay(&painter, &overlay, Color::new(0.95, 0.45, 0.2, 0.95));
    surface
}

fn render_panel(
    panel: &mut dyn raikou_layout::LayoutElement,
    available: Size,
) -> skia_safe::Surface {
    let mut surface =
        surfaces::raster_n32_premul((available.width as i32, available.height as i32))
            .expect("surface");
    let painter = Painter::new(surface.canvas());
    painter.clear(Color::new(1.0, 1.0, 1.0, 1.0));

    measure_element(panel, available);
    arrange_element(
        panel,
        Rect::from_xywh(0.0, 0.0, available.width, available.height),
    );
    paint_rect_tree(&painter, panel);
    surface
}

fn paint_rect_tree(painter: &Painter<'_>, element: &dyn raikou_layout::LayoutElement) {
    let palette = [
        Color::new(0.2, 0.47, 0.82, 1.0),
        Color::new(0.2, 0.7, 0.42, 1.0),
        Color::new(0.92, 0.62, 0.2, 1.0),
        Color::new(0.78, 0.31, 0.32, 1.0),
    ];
    let mut index = 0usize;
    element.visit_children(&mut |child| {
        let color = palette[index % palette.len()];
        painter.fill_rounded_rect(
            RoundedRect::from_rect_xy(child.layout().bounds(), 4.0),
            color,
        );
        index += 1;
    });
}

fn paint_canvas_like(painter: &Painter<'_>, canvas: &Canvas, color: Color) {
    canvas.children().iter().for_each(|child| {
        painter.fill_rounded_rect(
            RoundedRect::from_rect_xy(child.layout().bounds(), 6.0),
            color,
        );
    });
}

fn paint_overlay(painter: &Painter<'_>, overlay: &OverlayLayer, color: Color) {
    overlay.visit_children(&mut |child| {
        painter.fill_rounded_rect(
            RoundedRect::from_rect_xy(child.layout().bounds(), 10.0),
            color,
        );
    });
}
