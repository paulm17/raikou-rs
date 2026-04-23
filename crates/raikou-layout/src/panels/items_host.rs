use std::any::Any;
use std::ops::Range;

use raikou_core::{Rect, Size};

use crate::alignment::Orientation;
use crate::layoutable::{LayoutContext, LayoutElement, Layoutable, arrange_element, measure_element};

pub trait VirtualizationHost {
    fn realized_range(&self) -> Range<usize>;
}

pub struct ItemsHost {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    realized_range: Range<usize>,
    pub orientation: Orientation,
    pub estimated_item_size: f32,
    pub viewport_offset: f32,
    pub viewport_size: f32,
}

impl ItemsHost {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            realized_range: 0..0,
            orientation: Orientation::Vertical,
            estimated_item_size: 24.0,
            viewport_offset: 0.0,
            viewport_size: 0.0,
        }
    }

    pub fn push_child(&mut self, mut child: Box<dyn LayoutElement>) {
        child.layout_mut().set_parent_id(Some(self.layout.id()));
        self.children.push(child);
        self.recalculate_realized_range();
        self.layout.invalidate_measure();
    }

    pub fn remove_child(&mut self, index: usize) -> Box<dyn LayoutElement> {
        let child = self.children.remove(index);
        self.recalculate_realized_range();
        self.layout.invalidate_measure();
        child
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn set_viewport_offset(&mut self, offset: f32) {
        self.viewport_offset = offset.max(0.0);
        self.recalculate_realized_range();
        self.layout.invalidate_arrange();
    }

    fn recalculate_realized_range(&mut self) {
        if self.children.is_empty() {
            self.realized_range = 0..0;
            return;
        }
        if self.viewport_size <= 0.0 {
            self.realized_range = 0..self.children.len();
            return;
        }

        let item_size = self.estimate_item_size();
        let start = (self.viewport_offset / item_size).floor() as usize;
        let visible_count = (self.viewport_size / item_size).ceil() as usize + 1;
        let end = (start + visible_count).min(self.children.len());
        let start = start.min(end);
        self.realized_range = start..end;
    }

    fn estimate_item_size(&self) -> f32 {
        if self.children.is_empty() {
            return self.estimated_item_size.max(1.0);
        }
        // Use the first realized child as a sample
        for child in &self.children {
            let size = child.layout().desired_size();
            let item_size = match self.orientation {
                Orientation::Vertical => size.height,
                Orientation::Horizontal => size.width,
            };
            if item_size > 0.0 {
                return item_size;
            }
        }
        self.estimated_item_size.max(1.0)
    }
}

impl Default for ItemsHost {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualizationHost for ItemsHost {
    fn realized_range(&self) -> Range<usize> {
        self.realized_range.clone()
    }
}

impl LayoutElement for ItemsHost {
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

    fn measure_override(&mut self, ctx: &mut LayoutContext, available: Size) -> Size {
        // Only measure realized children
        let mut desired = Size::ZERO;
        for index in self.realized_range.clone() {
            if let Some(child) = self.children.get_mut(index) {
                let child_size = measure_element(child.as_mut(), ctx, available);
                desired.width = desired.width.max(child_size.width);
                desired.height = desired.height.max(child_size.height);
            }
        }

        // The desired size should account for the total logical extent if possible
        let total_count = self.children.len();
        if total_count > 0 {
            let item_size = self.estimate_item_size();
            match self.orientation {
                Orientation::Vertical => {
                    desired.height = (item_size * total_count as f32).max(desired.height);
                }
                Orientation::Horizontal => {
                    desired.width = (item_size * total_count as f32).max(desired.width);
                }
            }
        }

        desired
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        let item_size = self.estimate_item_size();
        let total_logical_size = item_size * self.children.len() as f32;

        self.viewport_size = match self.orientation {
            Orientation::Vertical => final_size.height,
            Orientation::Horizontal => final_size.width,
        };
        self.recalculate_realized_range();

        // Arrange unrealized children with zero rect so they don't paint
        for (index, child) in self.children.iter_mut().enumerate() {
            if !self.realized_range.contains(&index) {
                arrange_element(child.as_mut(), ctx, Rect::from_xywh(0.0, 0.0, 0.0, 0.0));
            }
        }

        // Arrange realized children at the correct offset
        for index in self.realized_range.clone() {
            if let Some(child) = self.children.get_mut(index) {
                let logical_offset = item_size * index as f32 - self.viewport_offset;
                let rect = match self.orientation {
                    Orientation::Vertical => Rect::from_xywh(
                        0.0,
                        logical_offset,
                        final_size.width,
                        item_size,
                    ),
                    Orientation::Horizontal => Rect::from_xywh(
                        logical_offset,
                        0.0,
                        item_size,
                        final_size.height,
                    ),
                };
                arrange_element(child.as_mut(), ctx, rect);
            }
        }

        match self.orientation {
            Orientation::Vertical => Size::new(final_size.width, total_logical_size),
            Orientation::Horizontal => Size::new(total_logical_size, final_size.height),
        }
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
