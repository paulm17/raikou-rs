//! Table component: a static grid of string cells backed by fyrox's `Grid`.
//!
//! Rows carry Fluent chrome: a hairline separator under the header,
//! alternating row fills and hover highlighting via a per-row `Decorator`.

use fyrox::core::pool::Handle;
use fyrox::gui::border::BorderBuilder;
use fyrox::gui::brush::Brush;
use fyrox::gui::decorator::DecoratorBuilder;
use fyrox::gui::grid::{Column, GridBuilder, Row};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::UiNode;

use raikou_core::Thickness;

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::{to_fyrox_color, to_fyrox_thickness};

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
        let theme = cx.theme().clone();

        let fallback_stroke = raikou_core::Color::new(0.0, 0.0, 0.0, 0.14);
        let stroke = to_fyrox_color(theme.color("border.subtle").unwrap_or(fallback_stroke));
        let fallback_alt = raikou_core::Color::new(0.0, 0.0, 0.0, 0.05);
        let alt_fill = to_fyrox_color(theme.color("fluent.list.low").unwrap_or(fallback_alt));
        let fallback_hover = raikou_core::Color::new(0.0, 0.0, 0.0, 0.10);
        let hover_fill =
            to_fyrox_color(theme.color("fluent.list.medium").unwrap_or(fallback_hover));
        let transparent = fyrox::core::color::Color::TRANSPARENT;

        let mut ctx = cx.ctx();

        // Header texts live in their own grid so the outer grid can use one
        // stretch column spanning the full width.
        let header_grid: Handle<UiNode> = {
            let mut header_nodes = Vec::new();
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
                header_nodes.push(header.to_base());
            }
            let mut builder = GridBuilder::new(
                WidgetBuilder::new()
                    .with_name("raikou_table_header")
                    .with_children(header_nodes),
            )
            .add_row(Row::strict(self.row_height));
            for column in &self.columns {
                builder = builder.add_column(Column::strict(column.width));
            }
            builder.build(&mut ctx).to_base()
        };

        // Hairline separator under the header.
        let separator: Handle<UiNode> = BorderBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_table_header_rule")
                .with_height(1.0)
                .with_background(Brush::Solid(stroke).into())
                .with_foreground(Brush::Solid(transparent).into()),
        )
        .with_stroke_thickness(fyrox::gui::Thickness::uniform(0.0).into())
        .with_pad_by_corner_radius(false)
        .build(&mut ctx)
        .to_base();

        // Data rows: each row is a full-width Border (wrapped in a Decorator
        // for hover chrome) containing an inner grid of its cells.
        let mut row_nodes = Vec::new();
        for (row_idx, cells) in self.rows.iter().enumerate() {
            let mut cell_nodes = Vec::new();
            for (col_idx, cell) in cells.iter().enumerate() {
                let text = fyrox::gui::text::TextBuilder::new(
                    WidgetBuilder::new()
                        .on_row(0)
                        .on_column(col_idx)
                        .with_margin(fyrox::gui::Thickness::uniform(4.0)),
                )
                .with_text(cell)
                .with_font(ctx.default_font())
                .build(&mut ctx);
                cell_nodes.push(text.to_base());
            }

            let mut cell_grid = GridBuilder::new(WidgetBuilder::new().with_children(cell_nodes))
                .add_row(Row::strict(self.row_height));
            for column in &self.columns {
                cell_grid = cell_grid.add_column(Column::strict(column.width));
            }
            let cell_grid: Handle<UiNode> = cell_grid.build(&mut ctx).to_base();

            let normal = if row_idx % 2 == 0 {
                alt_fill
            } else {
                transparent
            };
            let row_border: Handle<UiNode> = DecoratorBuilder::new(
                BorderBuilder::new(
                    WidgetBuilder::new()
                        .with_name("raikou_table_row")
                        .with_child(cell_grid),
                )
                .with_stroke_thickness(fyrox::gui::Thickness::uniform(0.0).into())
                .with_pad_by_corner_radius(false),
            )
            .with_normal_brush(Brush::Solid(normal).into())
            .with_hover_brush(Brush::Solid(hover_fill).into())
            .with_pressed_brush(Brush::Solid(normal).into())
            .with_selected_brush(Brush::Solid(normal).into())
            .with_pressable(false)
            .build(&mut ctx)
            .to_base();
            row_nodes.push(row_border);
        }

        // Outer grid: one stretch column; rows = header, separator, data.
        let mut grid_builder = GridBuilder::new(
            WidgetBuilder::new()
                .with_name("raikou_table")
                .with_margin(to_fyrox_thickness(self.margin))
                .on_column(0)
                .with_child(header_grid)
                .with_child(separator)
                .with_children(row_nodes.clone()),
        );
        grid_builder = grid_builder
            .add_row(Row::strict(self.row_height))
            .add_row(Row::strict(1.0))
            .add_column(Column::stretch());
        for _ in &self.rows {
            grid_builder = grid_builder.add_row(Row::strict(self.row_height));
        }
        // Place header + separator (children 0 and 1) onto their rows.
        ctx[header_grid].set_row(0).set_column(0);
        ctx[separator].set_row(1).set_column(0);
        for (i, node) in row_nodes.iter().enumerate() {
            ctx[*node].set_row(i + 2).set_column(0);
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
