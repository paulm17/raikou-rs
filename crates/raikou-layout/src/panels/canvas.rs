use std::any::Any;

use raikou_core::{Rect, Size};

use crate::layoutable::{LayoutContext, LayoutElement, Layoutable, arrange_element, measure_element};

pub struct Canvas {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
        }
    }

    pub fn push_child(&mut self, mut child: Box<dyn LayoutElement>) {
        child.layout_mut().set_parent_id(Some(self.layout.id()));
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
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for Canvas {
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

    fn measure_override(&mut self, ctx: &mut LayoutContext, _available: Size) -> Size {
        for child in &mut self.children {
            measure_element(child.as_mut(), ctx, Size::new(f32::INFINITY, f32::INFINITY));
        }
        Size::ZERO
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        for child in &mut self.children {
            let desired = child.layout().desired_size();
            let canvas = child.layout().attached.canvas;
            let x = if let Some(left) = canvas.left {
                left
            } else if let Some(right) = canvas.right {
                final_size.width - desired.width - right
            } else {
                0.0
            };
            let y = if let Some(top) = canvas.top {
                top
            } else if let Some(bottom) = canvas.bottom {
                final_size.height - desired.height - bottom
            } else {
                0.0
            };
            arrange_element(
                child.as_mut(),
                ctx,
                Rect::from_xywh(x, y, desired.width, desired.height),
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
