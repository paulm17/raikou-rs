use std::any::Any;

use raikou_core::{Color, Rect, Size};

use crate::layoutable::{LayoutElement, Layoutable, arrange_element, measure_element};

pub struct Panel {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    background: Option<Color>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            background: None,
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

    pub fn background(&self) -> Option<Color> {
        self.background
    }

    pub fn set_background(&mut self, background: Option<Color>) {
        self.background = background;
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for Panel {
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
        for child in &mut self.children {
            let child_size = measure_element(child.as_mut(), available);
            desired.width = desired.width.max(child_size.width);
            desired.height = desired.height.max(child_size.height);
        }
        desired
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
        for child in &mut self.children {
            arrange_element(
                child.as_mut(),
                Rect::from_xywh(0.0, 0.0, final_size.width, final_size.height),
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
