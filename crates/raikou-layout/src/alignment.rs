#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HorizontalAlignment {
    #[default]
    Stretch,
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VerticalAlignment {
    #[default]
    Stretch,
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WrapItemsAlignment {
    #[default]
    Start,
    Center,
    End,
}
