#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dock {
    #[default]
    Left,
    Bottom,
    Right,
    Top,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CanvasPosition {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPlacement {
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

impl Default for GridPlacement {
    fn default() -> Self {
        Self {
            row: 0,
            column: 0,
            row_span: 1,
            column_span: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AttachedLayout {
    pub dock: Dock,
    pub canvas: CanvasPosition,
    pub grid: GridPlacement,
}
