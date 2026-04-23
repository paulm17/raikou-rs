use std::any::Any;

use raikou_core::{Rect, Size};

use crate::attached::GridPlacement;
use crate::layoutable::{LayoutElement, Layoutable, arrange_element, measure_element};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridLength {
    Auto,
    Pixel(f32),
    Star(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnDefinition {
    pub width: GridLength,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowDefinition {
    pub height: GridLength,
}

impl ColumnDefinition {
    pub fn auto() -> Self {
        Self {
            width: GridLength::Auto,
        }
    }

    pub fn pixel(width: f32) -> Self {
        Self {
            width: GridLength::Pixel(width),
        }
    }

    pub fn star(weight: f32) -> Self {
        Self {
            width: GridLength::Star(weight),
        }
    }
}

impl RowDefinition {
    pub fn auto() -> Self {
        Self {
            height: GridLength::Auto,
        }
    }

    pub fn pixel(height: f32) -> Self {
        Self {
            height: GridLength::Pixel(height),
        }
    }

    pub fn star(weight: f32) -> Self {
        Self {
            height: GridLength::Star(weight),
        }
    }
}

pub struct Grid {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    pub columns: Vec<ColumnDefinition>,
    pub rows: Vec<RowDefinition>,
    pub column_spacing: f32,
    pub row_spacing: f32,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            column_spacing: 0.0,
            row_spacing: 0.0,
        }
    }

    pub fn push_child(&mut self, child: Box<dyn LayoutElement>) {
        self.children.push(child);
        self.layout.invalidate_measure();
    }

    pub fn remove_child(&mut self, index: usize) -> Box<dyn LayoutElement> {
        let child = self.children.remove(index);
        self.layout.invalidate_measure();
        child
    }

    pub fn children(&self) -> &[Box<dyn LayoutElement>] {
        &self.children
    }

    fn effective_columns(&self) -> usize {
        self.columns.len().max(1)
    }

    fn effective_rows(&self) -> usize {
        self.rows.len().max(1)
    }

    fn placement(placement: GridPlacement, columns: usize, rows: usize) -> GridPlacement {
        GridPlacement {
            row: placement.row.min(rows.saturating_sub(1)),
            column: placement.column.min(columns.saturating_sub(1)),
            row_span: placement.row_span.max(1).min(rows),
            column_span: placement.column_span.max(1).min(columns),
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for Grid {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn layout(&self) -> &Layoutable {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layoutable {
        &mut self.layout
    }

    fn measure_override(&mut self, available: Size) -> Size {
        let column_count = self.effective_columns();
        let row_count = self.effective_rows();
        let mut column_sizes = vec![0.0; column_count];
        let mut row_sizes = vec![0.0; row_count];
        let width_is_finite = available.width.is_finite();
        let height_is_finite = available.height.is_finite();

        for (index, definition) in self.columns.iter().enumerate() {
            if let GridLength::Pixel(width) = definition.width {
                column_sizes[index] = width;
            }
        }
        for (index, definition) in self.rows.iter().enumerate() {
            if let GridLength::Pixel(height) = definition.height {
                row_sizes[index] = height;
            }
        }

        for child in &mut self.children {
            let child_available = Size::new(
                if width_is_finite {
                    available.width
                } else {
                    f32::INFINITY
                },
                if height_is_finite {
                    available.height
                } else {
                    f32::INFINITY
                },
            );
            let child_size = measure_element(child.as_mut(), child_available);
            let placement = Self::placement(child.layout().attached.grid, column_count, row_count);

            for column in
                placement.column..(placement.column + placement.column_span).min(column_count)
            {
                let treat_as_auto = if width_is_finite {
                    matches!(
                        self.columns.get(column).map(|x| x.width),
                        Some(GridLength::Auto) | None
                    )
                } else {
                    matches!(
                        self.columns.get(column).map(|x| x.width),
                        Some(GridLength::Auto | GridLength::Star(_)) | None
                    )
                };
                if treat_as_auto {
                    column_sizes[column] =
                        column_sizes[column].max(child_size.width / placement.column_span as f32);
                }
            }

            for row in placement.row..(placement.row + placement.row_span).min(row_count) {
                let treat_as_auto = if height_is_finite {
                    matches!(
                        self.rows.get(row).map(|x| x.height),
                        Some(GridLength::Auto) | None
                    )
                } else {
                    matches!(
                        self.rows.get(row).map(|x| x.height),
                        Some(GridLength::Auto | GridLength::Star(_)) | None
                    )
                };
                if treat_as_auto {
                    row_sizes[row] =
                        row_sizes[row].max(child_size.height / placement.row_span as f32);
                }
            }
        }

        let fixed_width: f32 = column_sizes.iter().sum();
        let fixed_height: f32 = row_sizes.iter().sum();
        let remaining_width = (available.width
            - fixed_width
            - self.column_spacing * column_count.saturating_sub(1) as f32)
            .max(0.0);
        let remaining_height = (available.height
            - fixed_height
            - self.row_spacing * row_count.saturating_sub(1) as f32)
            .max(0.0);
        let total_star_width: f32 = self
            .columns
            .iter()
            .map(|definition| match definition.width {
                GridLength::Star(weight) => weight.max(0.0),
                _ => 0.0,
            })
            .sum();
        let total_star_height: f32 = self
            .rows
            .iter()
            .map(|definition| match definition.height {
                GridLength::Star(weight) => weight.max(0.0),
                _ => 0.0,
            })
            .sum();

        if width_is_finite && total_star_width > 0.0 {
            for (index, definition) in self.columns.iter().enumerate() {
                if let GridLength::Star(weight) = definition.width {
                    column_sizes[index] = remaining_width * (weight / total_star_width);
                }
            }
        }
        if height_is_finite && total_star_height > 0.0 {
            for (index, definition) in self.rows.iter().enumerate() {
                if let GridLength::Star(weight) = definition.height {
                    row_sizes[index] = remaining_height * (weight / total_star_height);
                }
            }
        }

        Size::new(
            column_sizes.iter().sum::<f32>()
                + self.column_spacing * column_count.saturating_sub(1) as f32,
            row_sizes.iter().sum::<f32>() + self.row_spacing * row_count.saturating_sub(1) as f32,
        )
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
        let column_count = self.effective_columns();
        let row_count = self.effective_rows();
        let mut column_sizes = vec![0.0; column_count];
        let mut row_sizes = vec![0.0; row_count];
        let mut star_columns = 0.0;
        let mut star_rows = 0.0;
        for (index, definition) in self.columns.iter().enumerate() {
            match definition.width {
                GridLength::Pixel(width) => {
                    column_sizes[index] = width;
                }
                GridLength::Star(weight) => star_columns += weight.max(0.0),
                GridLength::Auto => {}
            }
        }

        for (index, definition) in self.rows.iter().enumerate() {
            match definition.height {
                GridLength::Pixel(height) => {
                    row_sizes[index] = height;
                }
                GridLength::Star(weight) => star_rows += weight.max(0.0),
                GridLength::Auto => {}
            }
        }

        for child in &self.children {
            let desired = child.layout().desired_size();
            let placement = Self::placement(child.layout().attached.grid, column_count, row_count);
            for column in
                placement.column..(placement.column + placement.column_span).min(column_count)
            {
                if matches!(
                    self.columns.get(column).map(|x| x.width),
                    Some(GridLength::Auto) | None
                ) {
                    column_sizes[column] =
                        column_sizes[column].max(desired.width / placement.column_span as f32);
                }
            }
            for row in placement.row..(placement.row + placement.row_span).min(row_count) {
                if matches!(
                    self.rows.get(row).map(|x| x.height),
                    Some(GridLength::Auto) | None
                ) {
                    row_sizes[row] = row_sizes[row].max(desired.height / placement.row_span as f32);
                }
            }
        }

        let fixed_width = column_sizes.iter().sum::<f32>();
        let fixed_height = row_sizes.iter().sum::<f32>();

        let remaining_width = (final_size.width
            - fixed_width
            - self.column_spacing * column_count.saturating_sub(1) as f32)
            .max(0.0);
        let remaining_height = (final_size.height
            - fixed_height
            - self.row_spacing * row_count.saturating_sub(1) as f32)
            .max(0.0);

        if star_columns > 0.0 {
            for (index, definition) in self.columns.iter().enumerate() {
                if let GridLength::Star(weight) = definition.width {
                    column_sizes[index] = remaining_width * (weight / star_columns);
                }
            }
        }
        if star_rows > 0.0 {
            for (index, definition) in self.rows.iter().enumerate() {
                if let GridLength::Star(weight) = definition.height {
                    row_sizes[index] = remaining_height * (weight / star_rows);
                }
            }
        }

        let mut column_offsets = vec![0.0; column_count];
        let mut row_offsets = vec![0.0; row_count];
        for index in 1..column_count {
            column_offsets[index] =
                column_offsets[index - 1] + column_sizes[index - 1] + self.column_spacing;
        }
        for index in 1..row_count {
            row_offsets[index] = row_offsets[index - 1] + row_sizes[index - 1] + self.row_spacing;
        }

        for child in &mut self.children {
            let placement = Self::placement(child.layout().attached.grid, column_count, row_count);
            let width = column_sizes
                [placement.column..(placement.column + placement.column_span).min(column_count)]
                .iter()
                .sum::<f32>()
                + self.column_spacing * placement.column_span.saturating_sub(1) as f32;
            let height = row_sizes
                [placement.row..(placement.row + placement.row_span).min(row_count)]
                .iter()
                .sum::<f32>()
                + self.row_spacing * placement.row_span.saturating_sub(1) as f32;
            arrange_element(
                child.as_mut(),
                Rect::from_xywh(
                    column_offsets[placement.column],
                    row_offsets[placement.row],
                    width,
                    height,
                ),
            );
        }

        final_size
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn LayoutElement)) {
        for child in &self.children {
            visitor(child.as_ref());
        }
    }

    fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {
        for child in &mut self.children {
            visitor(child.as_mut());
        }
    }
}
