use fyrox::{
    core::{algebra::Vector2, color::Color, pool::Handle},
    gui::{
        brush::Brush,
        canvas::CanvasBuilder,
        scroll_panel::ScrollPanel,
        scroll_viewer::{ScrollViewer, ScrollViewerBuilder},
        text::{TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        UiNode, UserInterface,
    },
};

/// A scrollable list that virtualizes its rows: only a small bounded pool of row
/// widgets exists in the scene, regardless of how many logical rows there are. The
/// scrollbar range is driven by the content canvas' explicit height
/// (`total_rows * row_height`), while the pool is repositioned and re-texted on
/// every scroll to cover the visible window.
pub struct VirtualList {
    pub scroll_viewer: Handle<UiNode>,
    pub scroll_panel: Handle<UiNode>,
    rows: Vec<Handle<UiNode>>,
    row_height: f32,
    total_rows: usize,
    last_start: usize,
}

impl VirtualList {
    /// Builds the virtualized list. `width`/`height` are the viewport size in
    /// logical pixels; the pool is sized to cover the viewport plus two buffer rows.
    pub fn build(
        ui: &mut UserInterface,
        width: f32,
        height: f32,
        total_rows: usize,
        row_height: f32,
    ) -> Self {
        let pool_len = (height / row_height).ceil() as usize + 2;
        let ctx = &mut ui.build_ctx();

        let mut rows = Vec::with_capacity(pool_len);
        for i in 0..pool_len {
            let background = if i % 2 == 0 {
                Color::opaque(48, 48, 54)
            } else {
                Color::opaque(40, 40, 46)
            };
            let row: Handle<UiNode> = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(width)
                    .with_height(row_height)
                    .with_background(Brush::Solid(background).into()),
            )
            .with_text(String::new())
            .build(ctx)
            .transmute();
            rows.push(row);
        }

        let content: Handle<UiNode> = CanvasBuilder::new(
            WidgetBuilder::new()
                .with_width(width)
                .with_height(total_rows as f32 * row_height)
                .with_children(rows.iter().copied()),
        )
        .build(ctx)
        .transmute();

        let scroll_viewer: Handle<UiNode> = ScrollViewerBuilder::new(
            WidgetBuilder::new()
                .with_name("list")
                .with_width(width)
                .with_height(height),
        )
        .with_content(content)
        .with_v_scroll_speed(30.0)
        .build(ctx)
        .transmute();

        let scroll_panel = match ui.nodes()[scroll_viewer].cast::<ScrollViewer>() {
            Some(viewer) => viewer.scroll_panel.transmute(),
            None => Handle::NONE,
        };

        Self {
            scroll_viewer,
            scroll_panel,
            rows,
            row_height,
            total_rows,
            last_start: usize::MAX,
        }
    }

    /// Current scroll position in logical pixels along the vertical axis.
    pub fn scroll_y(&self, ui: &UserInterface) -> f32 {
        ui.nodes()[self.scroll_panel]
            .cast::<ScrollPanel>()
            .map(|panel| panel.scroll.y)
            .unwrap_or(0.0)
    }

    /// Re-syncs the row pool with the current scroll position. Cheap when the
    /// visible window has not changed; repositions and re-texts all pool rows when
    /// it has.
    pub fn refresh(&mut self, ui: &mut UserInterface) {
        let scroll_y = self.scroll_y(ui);
        let start = (scroll_y / self.row_height).floor().max(0.0) as usize;
        if start == self.last_start {
            return;
        }
        self.last_start = start;

        for (i, row) in self.rows.iter().enumerate() {
            let row_idx = start + i;
            if row_idx < self.total_rows {
                ui.send(
                    *row,
                    WidgetMessage::DesiredPosition(Vector2::new(
                        0.0,
                        row_idx as f32 * self.row_height,
                    )),
                );
                ui.send(
                    *row,
                    TextMessage::Text(format!("Row {row_idx}: item #{row_idx}")),
                );
            } else {
                // Beyond the last row: park the spare rows well below the content
                // extent so they stay out of the clipped viewport.
                ui.send(
                    *row,
                    WidgetMessage::DesiredPosition(Vector2::new(
                        0.0,
                        self.total_rows as f32 * self.row_height + 200.0,
                    )),
                );
                ui.send(*row, TextMessage::Text(String::new()));
            }
        }
    }

    /// Rebuilds a `VirtualList` from a UI that was loaded from a `.ui` file. The
    /// scroll viewer's handle identifies the list; everything else (content canvas,
    /// scroll panel, row pool, row height, total extent) is recovered
    /// from the loaded widget tree, so the only thing that must survive serialization
    /// is the scroll viewer's name and its shape.
    pub fn from_loaded(ui: &UserInterface, scroll_viewer: Handle<UiNode>) -> Self {
        let mut scroll_panel: Handle<UiNode> = Handle::NONE;
        let mut rows: Vec<Handle<UiNode>> = Vec::new();
        let mut row_height = 22.0;
        let mut total_rows = 200;

        if let Some(viewer) = ui
            .nodes()
            .try_get(scroll_viewer)
            .ok()
            .and_then(|node| node.cast::<ScrollViewer>())
        {
            scroll_panel = viewer.scroll_panel.transmute();
            if let Some(content_node) = ui.nodes().try_get(viewer.content).ok() {
                rows = content_node.children().iter().copied().collect();
                let content_height = content_node.height();
                if let Some(first) = rows.first() {
                    if let Some(row_node) = ui.nodes().try_get(*first).ok() {
                        let h = row_node.height();
                        if h > 0.0 {
                            row_height = h;
                        }
                    }
                }
                if content_height > 0.0 && row_height > 0.0 {
                    total_rows = (content_height / row_height).round() as usize;
                }
            }
        }

        Self {
            scroll_viewer,
            scroll_panel,
            rows,
            row_height,
            total_rows,
            last_start: usize::MAX,
        }
    }
}