use std::any::Any;

use raikou_core::{OverlayPaintPhase, PaintLayer, Rect, Size};

use crate::layoutable::{LayoutContext, LayoutElement, Layoutable, arrange_element, measure_element};

pub struct OverlayLayer {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    available_size: Size,
}

impl OverlayLayer {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            available_size: Size::ZERO,
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

    pub fn available_size(&self) -> Size {
        self.available_size
    }
}

impl Default for OverlayLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for OverlayLayer {
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
        for child in &mut self.children {
            measure_element(child.as_mut(), ctx, available);
        }
        available
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        self.available_size = final_size;
        for child in &mut self.children {
            let desired = child.layout().desired_size();
            arrange_element(
                child.as_mut(),
                ctx,
                Rect::from_xywh(0.0, 0.0, desired.width, desired.height),
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

    fn paint_layer(&self) -> PaintLayer {
        PaintLayer::Overlay(OverlayPaintPhase::AfterContent)
    }
}
