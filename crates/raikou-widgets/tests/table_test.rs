//! Headless tests for the Table component: structure, zebra opt-in and
//! row-height clamping.

mod common;

use fyrox::graph::SceneGraph;

use common::Harness;
use raikou_widgets::{Table, TableColumn};

type NodeHandle = fyrox::core::pool::Handle<fyrox::gui::UiNode>;

/// Walks the subtree under `root` and returns every node whose widget name
/// matches, in depth-first order.
fn find_by_name(h: &Harness, root: NodeHandle, needle: &str) -> Vec<NodeHandle> {
    let mut stack = vec![root];
    let mut visited = vec![root];
    let mut found = Vec::new();
    while let Some(node) = stack.pop() {
        if node.is_none() {
            continue;
        }
        if h.ui.node(node).name() == needle {
            found.push(node);
        }
        // Reverse so the stack pops children in document order.
        for child in h.ui.node(node).children().to_vec().into_iter().rev() {
            if !child.is_none() && !visited.contains(&child) {
                visited.push(child);
                stack.push(child);
            }
        }
    }
    found
}

/// Collects the text labels of every `Text` descendant of `root`.
fn text_labels(h: &Harness, root: NodeHandle) -> Vec<String> {
    use fyrox::gui::text::Text;

    let mut labels = Vec::new();
    for node in walk_all(h, root) {
        if let Ok(text) = h.ui.try_get_of_type::<Text>(node) {
            labels.push(text.text());
        }
    }
    labels
}

fn walk_all(h: &Harness, root: NodeHandle) -> Vec<NodeHandle> {
    let mut stack = vec![root];
    let mut visited = vec![root];
    let mut all = Vec::new();
    while let Some(node) = stack.pop() {
        if node.is_none() {
            continue;
        }
        all.push(node);
        // Reverse so the stack pops children in document order.
        for child in h.ui.node(node).children().to_vec().into_iter().rev() {
            if !child.is_none() && !visited.contains(&child) {
                visited.push(child);
                stack.push(child);
            }
        }
    }
    all
}

#[test]
fn table_builds_header_rule_rows_and_dividers() {
    let mut h = Harness::new();
    let table = h.build(|cx| {
        Table::new()
            .column(TableColumn::new("Name", 120.0))
            .column(TableColumn::new("Age", 80.0))
            .row(vec!["Ada", "36"])
            .row(vec!["Alan", "41"])
            .build(cx)
    });

    // Outer frame wraps a single inner grid.
    let frame_children = h.ui.node(table.handle).children().to_vec();
    assert_eq!(frame_children.len(), 1, "frame must wrap one grid");
    let inner = h
        .ui
        .try_get_of_type::<fyrox::gui::grid::Grid>(frame_children[0])
        .expect("inner table grid");

    // Header + hairline rule + two data rows.
    assert_eq!(inner.children.len(), 4);
    assert_eq!(find_by_name(&h, table.handle, "raikou_table_header").len(), 1);
    assert_eq!(
        find_by_name(&h, table.handle, "raikou_table_header_rule").len(),
        1
    );
    assert_eq!(find_by_name(&h, table.handle, "raikou_table_row").len(), 2);

    // Header carries the column captions.
    let header = find_by_name(&h, table.handle, "raikou_table_header")[0];
    assert_eq!(text_labels(&h, header), vec!["Name".to_string(), "Age".to_string()]);

    // Each data row interleaves cells with 1px dividers (2 cols → 3 nodes).
    let rows = find_by_name(&h, table.handle, "raikou_table_row");
    for row in &rows {
        let cell_grid = *h.ui.node(*row).children().first().unwrap();
        assert_eq!(
            h.ui.node(cell_grid).children().len(),
            3,
            "two cells plus one divider"
        );
    }
    // One divider between every column pair in the header plus one per row.
    assert_eq!(find_by_name(&h, table.handle, "raikou_table_divider").len(), 3);
}

#[test]
fn table_zebra_is_opt_in() {
    use fyrox::gui::brush::Brush;
    use fyrox::gui::decorator::Decorator;

    let normal_alpha = |h: &Harness, row: NodeHandle| -> u8 {
        h.ui
            .try_get_of_type::<Decorator>(row)
            .map(|d| match &**d.normal_brush {
                Brush::Solid(color) => color.a,
                _ => 255,
            })
            .unwrap_or(255)
    };

    // Default: plain rows (Avalonia DataGrid look).
    let mut h = Harness::new();
    let table = h.build(|cx| {
        Table::new()
            .column(TableColumn::new("A", 60.0))
            .row(vec!["1"])
            .row(vec!["2"])
            .build(cx)
    });
    let rows = find_by_name(&h, table.handle, "raikou_table_row");
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(normal_alpha(&h, *row), 0, "zebra must stay off by default");
    }

    // Opt-in: even data rows get the alternating fill.
    let mut h = Harness::new();
    let table = h.build(|cx| {
        Table::new()
            .column(TableColumn::new("A", 60.0))
            .row(vec!["1"])
            .row(vec!["2"])
            .zebra(true)
            .build(cx)
    });
    let rows = find_by_name(&h, table.handle, "raikou_table_row");
    assert!(normal_alpha(&h, rows[0]) > 0, "first data row tinted");
    assert_eq!(normal_alpha(&h, rows[1]), 0, "second data row stays plain");
}

#[test]
fn table_row_height_clamps_to_minimum() {
    use fyrox::gui::grid::{Grid, SizeMode};

    let header_row_height = |table: NodeHandle, h: &Harness| -> f32 {
        let children = h.ui.node(table).children().to_vec();
        let grid = h.ui.try_get_of_type::<Grid>(children[0]).unwrap();
        let rows = grid.rows.borrow().clone();
        assert_eq!(rows[0].size_mode, SizeMode::Strict);
        rows[0].desired_size
    };

    let mut h = Harness::new();
    let table = h.build(|cx| {
        Table::new()
            .column(TableColumn::new("A", 60.0))
            .row(vec!["1"])
            .build(cx)
    });
    assert_eq!(header_row_height(table.handle, &h), 32.0);

    let mut h = Harness::new();
    let table = h.build(|cx| {
        Table::new()
            .column(TableColumn::new("A", 60.0))
            .row(vec!["1"])
            .row_height(8.0)
            .build(cx)
    });
    assert_eq!(header_row_height(table.handle, &h), 16.0);
}
