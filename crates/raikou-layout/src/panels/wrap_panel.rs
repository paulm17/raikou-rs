use std::any::Any;

use raikou_core::{Rect, Size};

use crate::alignment::{Orientation, WrapItemsAlignment};
use crate::layoutable::{LayoutElement, Layoutable, Visibility, arrange_element, measure_element};

pub struct WrapPanel {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    pub orientation: Orientation,
    pub item_spacing: f32,
    pub line_spacing: f32,
    pub items_alignment: WrapItemsAlignment,
    pub item_width: Option<f32>,
    pub item_height: Option<f32>,
}

#[derive(Clone, Debug)]
struct Line {
    child_indexes: Vec<usize>,
    main: f32,
    cross: f32,
}

impl WrapPanel {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            orientation: Orientation::Horizontal,
            item_spacing: 0.0,
            line_spacing: 0.0,
            items_alignment: WrapItemsAlignment::Start,
            item_width: None,
            item_height: None,
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

    fn lines(&self, available: Size) -> Vec<Line> {
        let main_limit = match self.orientation {
            Orientation::Horizontal => available.width,
            Orientation::Vertical => available.height,
        };

        let mut lines = Vec::new();
        let mut current = Line {
            child_indexes: Vec::new(),
            main: 0.0,
            cross: 0.0,
        };
        let mut any_visible = false;

        for (index, child) in self.children.iter().enumerate() {
            if child.layout().visibility == Visibility::Collapsed {
                continue;
            }

            let desired = child.layout().desired_size();
            let child_main = match self.orientation {
                Orientation::Horizontal => self.item_width.unwrap_or(desired.width),
                Orientation::Vertical => self.item_height.unwrap_or(desired.height),
            };
            let child_cross = match self.orientation {
                Orientation::Horizontal => self.item_height.unwrap_or(desired.height),
                Orientation::Vertical => self.item_width.unwrap_or(desired.width),
            };

            let spacing = if current.child_indexes.is_empty() {
                0.0
            } else {
                self.item_spacing
            };
            let wraps = main_limit.is_finite()
                && !current.child_indexes.is_empty()
                && current.main + spacing + child_main > main_limit;

            if wraps {
                lines.push(current);
                current = Line {
                    child_indexes: vec![index],
                    main: child_main,
                    cross: child_cross,
                };
            } else {
                current.child_indexes.push(index);
                current.main += spacing + child_main;
                current.cross = current.cross.max(child_cross);
            }

            any_visible = true;
        }

        if any_visible {
            lines.push(current);
        }

        lines
    }
}

impl Default for WrapPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for WrapPanel {
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
        let child_constraint = Size::new(
            self.item_width.unwrap_or(available.width),
            self.item_height.unwrap_or(available.height),
        );

        for child in &mut self.children {
            measure_element(child.as_mut(), child_constraint);
        }

        let lines = self.lines(available);
        let mut desired = Size::ZERO;

        for (index, line) in lines.iter().enumerate() {
            match self.orientation {
                Orientation::Horizontal => {
                    desired.width = desired.width.max(line.main);
                    desired.height += line.cross;
                    if index > 0 {
                        desired.height += self.line_spacing;
                    }
                }
                Orientation::Vertical => {
                    desired.width += line.cross;
                    if index > 0 {
                        desired.width += self.line_spacing;
                    }
                    desired.height = desired.height.max(line.main);
                }
            }
        }

        desired
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
        let lines = self.lines(final_size);
        let mut cross_offset = 0.0;

        for (line_index, line) in lines.iter().enumerate() {
            let available_main = match self.orientation {
                Orientation::Horizontal => final_size.width,
                Orientation::Vertical => final_size.height,
            };
            let main_origin = match self.items_alignment {
                WrapItemsAlignment::Start => 0.0,
                WrapItemsAlignment::Center => ((available_main - line.main).max(0.0)) / 2.0,
                WrapItemsAlignment::End => (available_main - line.main).max(0.0),
            };

            let mut main_offset = main_origin;
            for child_index in &line.child_indexes {
                let child = &mut self.children[*child_index];
                let desired = child.layout().desired_size();
                let child_size = Size::new(
                    self.item_width.unwrap_or(desired.width),
                    self.item_height.unwrap_or(desired.height),
                );
                let rect = match self.orientation {
                    Orientation::Horizontal => {
                        Rect::from_xywh(main_offset, cross_offset, child_size.width, line.cross)
                    }
                    Orientation::Vertical => {
                        Rect::from_xywh(cross_offset, main_offset, line.cross, child_size.height)
                    }
                };
                arrange_element(child.as_mut(), rect);
                main_offset += match self.orientation {
                    Orientation::Horizontal => child_size.width + self.item_spacing,
                    Orientation::Vertical => child_size.height + self.item_spacing,
                };
            }

            cross_offset += line.cross;
            if line_index + 1 < lines.len() {
                cross_offset += self.line_spacing;
            }
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
