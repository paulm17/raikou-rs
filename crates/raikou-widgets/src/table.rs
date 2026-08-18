//! Table component: a static grid of string cells backed by fyrox's `Grid`.

use fyrox::core::pool::Handle;
use fyrox::gui::grid::{Column, GridBuilder, Row};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

/// A column definition: header text plus a fixed width.
#[derive(Clone, Debug)]
pub struct TableColumn {
    /// The header label.
    pub header: String,
    /// The column width in logical pixels.
    pub width: f32,
}

impl TableColumn {
    /// Creates a new column with the given header and width.
    pub fn new(header: impl Into<String>, width: f32) -> Self {
        Self {
            header: header.into(),
            width: width.max(1.0),
        }
    }
}

/// Builder for a [`Table`] component (static grid, no row selection).
#[derive(Clone)]
pub struct Table {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    row_height: f32,
    margin: Thickness,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// Creates a new table builder.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_height: 32.0,
            margin: Thickness::ZERO,
        }
    }

    /// Adds a column.
    pub fn column(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Adds a row of cell strings.
    pub fn row(mut self, cells: Vec<impl Into<String>>) -> Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the row height (clamped to a minimum of 16).
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(16.0);
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the table, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let mut ctx = cx.ctx();

        let mut child_nodes = Vec::new();

        // Header row.
        for (col_idx, column) in self.columns.iter().enumerate() {
            let header = fyrox::gui::text::TextBuilder::new(
                WidgetBuilder::new()
                    .on_row(0)
                    .on_column(col_idx)
                    .with_margin(fyrox::gui::Thickness::uniform(4.0)),
            )
            .with_text(&column.header)
            .with_font(ctx.default_font())
            .build(&mut ctx);
            child_nodes.push(header.to_base());
        }

        // Data rows.
        for (row_idx, cells) in self.rows.iter().enumerate() {
            let row = row_idx + 1;
            for (col_idx, cell) in cells.iter().enumerate() {
                let text = fyrox::gui::text::TextBuilder::new(
                    WidgetBuilder::new()
                        .on_row(row)
                        .on_column(col_idx)
                        .with_margin(fyrox::gui::Thickness::uniform(4.0)),
                )
                .with_text(cell)
                .with_font(ctx.default_font())
                .build(&mut ctx);
                child_nodes.push(text.to_base());
            }
        }

        let mut grid_builder = GridBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_table")
                .with_margin(to_fyrox_thickness(self.margin))
                .with_children(child_nodes),
        );

        grid_builder = grid_builder.add_row(Row::strict(self.row_height));
        for column in &self.columns {
            grid_builder = grid_builder.add_column(Column::strict(column.width));
        }
        for _ in &self.rows {
            grid_builder = grid_builder.add_row(Row::strict(self.row_height));
        }

        let handle = grid_builder.build(&mut ctx).to_base();

        // Static component: no dispatchable handlers (matches reference, which
        // only tracks hover for styling).
        let component = Component {
            handle,
            kind: ComponentKind::Static,
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built table.
pub type TableHandle = Handle<UiNode>;
