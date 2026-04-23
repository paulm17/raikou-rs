use std::any::Any;

use raikou_core::{Rect, Size};

use crate::layoutable::{LayoutElement, Layoutable, arrange_element, measure_element};

pub struct ScrollContentPresenter {
    layout: Layoutable,
    child: Option<Box<dyn LayoutElement>>,
    viewport: Size,
    extent: Size,
    scroll_offset_x: f32,
    scroll_offset_y: f32,
}

impl ScrollContentPresenter {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            child: None,
            viewport: Size::ZERO,
            extent: Size::ZERO,
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
        }
    }

    pub fn set_child(&mut self, child: Box<dyn LayoutElement>) {
        self.child = Some(child);
        self.layout.invalidate_measure();
    }

    pub fn viewport(&self) -> Size {
        self.viewport
    }

    pub fn extent(&self) -> Size {
        self.extent
    }

    pub fn scroll_offset(&self) -> (f32, f32) {
        (self.scroll_offset_x, self.scroll_offset_y)
    }

    pub fn set_scroll_offset(&mut self, x: f32, y: f32) {
        let max_x = (self.extent.width - self.viewport.width).max(0.0);
        let max_y = (self.extent.height - self.viewport.height).max(0.0);
        self.scroll_offset_x = x.clamp(0.0, max_x);
        self.scroll_offset_y = y.clamp(0.0, max_y);
        self.layout.invalidate_arrange();
    }
}

impl Default for ScrollContentPresenter {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for ScrollContentPresenter {
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
        if let Some(child) = &mut self.child {
            self.extent = measure_element(child.as_mut(), Size::new(f32::INFINITY, f32::INFINITY));
            self.viewport = available;
            Size::new(
                self.extent.width.min(available.width),
                self.extent.height.min(available.height),
            )
        } else {
            Size::ZERO
        }
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
        self.viewport = final_size;
        if let Some(child) = &mut self.child {
            let extent = child.layout().desired_size();
            self.extent = extent;

            // Clamp scroll offsets based on current extent/viewport
            let max_x = (self.extent.width - self.viewport.width).max(0.0);
            let max_y = (self.extent.height - self.viewport.height).max(0.0);
            self.scroll_offset_x = self.scroll_offset_x.clamp(0.0, max_x);
            self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_y);

            arrange_element(
                child.as_mut(),
                Rect::from_xywh(
                    -self.scroll_offset_x,
                    -self.scroll_offset_y,
                    extent.width.max(final_size.width),
                    extent.height.max(final_size.height),
                ),
            );
        }
        final_size
    }

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn LayoutElement)) {
        if let Some(child) = &self.child {
            visitor(child.as_ref());
        }
    }

    fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {
        if let Some(child) = &mut self.child {
            visitor(child.as_mut());
        }
    }
}
