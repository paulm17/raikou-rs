use raikou_core::{Rect, Size};
use raikou_layout::{
    ColumnDefinition, Grid, LayoutElement, RowDefinition, SizedBox, arrange_element,
    measure_element,
};

#[test]
fn avalonia_grid_mixes_fixed_auto_and_star_tracks() {
    let mut first = SizedBox::new(Size::new(10.0, 20.0));
    first.layout_mut().attached.grid.column = 0;
    first.layout_mut().attached.grid.row = 0;

    let mut second = SizedBox::new(Size::new(30.0, 10.0));
    second.layout_mut().attached.grid.column = 1;
    second.layout_mut().attached.grid.row = 0;

    let mut third = SizedBox::new(Size::new(10.0, 10.0));
    third.layout_mut().attached.grid.column = 2;
    third.layout_mut().attached.grid.row = 1;

    let mut grid = Grid::new();
    grid.columns = vec![
        ColumnDefinition::pixel(40.0),
        ColumnDefinition::auto(),
        ColumnDefinition::star(1.0),
    ];
    grid.rows = vec![RowDefinition::auto(), RowDefinition::star(1.0)];
    grid.push_child(Box::new(first));
    grid.push_child(Box::new(second));
    grid.push_child(Box::new(third));

    assert_eq!(
        measure_element(&mut grid, Size::new(120.0, 80.0)),
        Size::new(120.0, 80.0)
    );
    arrange_element(&mut grid, Rect::from_xywh(0.0, 0.0, 120.0, 80.0));

    assert_eq!(
        grid.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 40.0, 20.0)
    );
    assert_eq!(
        grid.children()[1].layout().bounds(),
        Rect::from_xywh(40.0, 0.0, 30.0, 20.0)
    );
    assert_eq!(
        grid.children()[2].layout().bounds(),
        Rect::from_xywh(70.0, 20.0, 50.0, 60.0)
    );
}

#[test]
fn avalonia_grid_supports_spans() {
    let mut child = SizedBox::new(Size::new(50.0, 20.0));
    child.layout_mut().attached.grid.column_span = 2;

    let mut grid = Grid::new();
    grid.columns = vec![ColumnDefinition::auto(), ColumnDefinition::auto()];
    grid.rows = vec![RowDefinition::auto()];
    grid.push_child(Box::new(child));

    measure_element(&mut grid, Size::new(100.0, 40.0));
    arrange_element(&mut grid, Rect::from_xywh(0.0, 0.0, 100.0, 40.0));

    assert_eq!(
        grid.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 50.0, 20.0)
    );
}

#[test]
fn avalonia_grid_normalizes_out_of_range_cell_placement() {
    let mut child = SizedBox::new(Size::new(10.0, 10.0));
    child.layout_mut().attached.grid.column = 99;
    child.layout_mut().attached.grid.row = 99;

    let mut grid = Grid::new();
    grid.columns = vec![ColumnDefinition::pixel(30.0)];
    grid.rows = vec![RowDefinition::pixel(20.0)];
    grid.push_child(Box::new(child));

    arrange_element(&mut grid, Rect::from_xywh(0.0, 0.0, 30.0, 20.0));
    assert_eq!(
        grid.children()[0].layout().bounds(),
        Rect::from_xywh(0.0, 0.0, 30.0, 20.0)
    );
}

#[test]
fn avalonia_grid_empty_and_unconstrained_measurement_remain_stable() {
    let mut empty = Grid::new();
    assert_eq!(
        measure_element(&mut empty, Size::new(200.0, 100.0)),
        Size::ZERO
    );

    let mut child = SizedBox::new(Size::new(40.0, 30.0));
    child.layout_mut().attached.grid.column = 0;
    child.layout_mut().attached.grid.row = 0;
    let mut grid = Grid::new();
    grid.columns = vec![ColumnDefinition::star(1.0)];
    grid.rows = vec![RowDefinition::star(1.0)];
    grid.push_child(Box::new(child));

    assert_eq!(
        measure_element(&mut grid, Size::new(f32::INFINITY, f32::INFINITY)),
        Size::new(40.0, 30.0)
    );
}
