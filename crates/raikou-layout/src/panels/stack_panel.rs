use std::any::Any;

use raikou_core::{Rect, Size};

use crate::alignment::Orientation;
use crate::layoutable::{LayoutElement, Layoutable, Visibility, arrange_element, measure_element};

pub struct StackPanel {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    pub orientation: Orientation,
    pub spacing: f32,
}

impl StackPanel {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            orientation: Orientation::Vertical,
            spacing: 0.0,
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

    pub fn children_mut(&mut self) -> &mut [Box<dyn LayoutElement>] {
        &mut self.children
    }
}

impl Default for StackPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for StackPanel {
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
        let mut desired = Size::ZERO;
        let mut visible_count = 0usize;

        for child in &mut self.children {
            let child_available = match self.orientation {
                Orientation::Vertical => Size::new(available.width, f32::INFINITY),
                Orientation::Horizontal => Size::new(f32::INFINITY, available.height),
            };
            let child_size = measure_element(child.as_mut(), child_available);
            if child.layout().visibility == Visibility::Collapsed {
                continue;
            }
            visible_count += 1;

            match self.orientation {
                Orientation::Vertical => {
                    desired.width = desired.width.max(child_size.width);
                    desired.height += child_size.height;
                }
                Orientation::Horizontal => {
                    desired.width += child_size.width;
                    desired.height = desired.height.max(child_size.height);
                }
            }
        }

        if visible_count > 1 {
            let spacing = self.spacing * (visible_count.saturating_sub(1) as f32);
            match self.orientation {
                Orientation::Vertical => desired.height += spacing,
                Orientation::Horizontal => desired.width += spacing,
            }
        }

        desired
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
        let mut offset = 0.0;
        let mut placed_any = false;

        for child in &mut self.children {
            if child.layout().visibility == Visibility::Collapsed {
                arrange_element(child.as_mut(), Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
                continue;
            }

            if placed_any {
                offset += self.spacing;
            }

            let desired = child.layout().desired_size();
            let rect = match self.orientation {
                Orientation::Vertical => {
                    let height = desired.height.min((final_size.height - offset).max(0.0));
                    let rect = Rect::from_xywh(0.0, offset, final_size.width, height);
                    offset += height;
                    rect
                }
                Orientation::Horizontal => {
                    let width = desired.width.min((final_size.width - offset).max(0.0));
                    let rect = Rect::from_xywh(offset, 0.0, width, final_size.height);
                    offset += width;
                    rect
                }
            };

            arrange_element(child.as_mut(), rect);
            placed_any = true;
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
