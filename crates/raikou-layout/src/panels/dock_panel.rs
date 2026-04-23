use std::any::Any;

use raikou_core::{Rect, Size};

use crate::attached::Dock;
use crate::layoutable::{LayoutContext, LayoutElement, Layoutable, Visibility, arrange_element, measure_element};

pub struct DockPanel {
    layout: Layoutable,
    children: Vec<Box<dyn LayoutElement>>,
    pub last_child_fill: bool,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
}

impl DockPanel {
    pub fn new() -> Self {
        Self {
            layout: Layoutable::new(),
            children: Vec::new(),
            last_child_fill: true,
            horizontal_spacing: 0.0,
            vertical_spacing: 0.0,
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

impl Default for DockPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutElement for DockPanel {
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
        let measure_count = if self.last_child_fill && !self.children.is_empty() {
            self.children.len() - 1
        } else {
            self.children.len()
        };

        let mut parent_width: f32 = 0.0;
        let mut parent_height: f32 = 0.0;
        let mut accumulated_width: f32 = 0.0;
        let mut accumulated_height: f32 = 0.0;
        let mut any_horizontal_spacing = false;
        let mut any_vertical_spacing = false;

        for child in self.children.iter_mut().take(measure_count) {
            let child_constraint = Size::new(
                (available.width - accumulated_width).max(0.0),
                (available.height - accumulated_height).max(0.0),
            );
            let child_size = measure_element(child.as_mut(), ctx, child_constraint);
            match child.layout().attached.dock {
                Dock::Left | Dock::Right => {
                    parent_height = parent_height.max(accumulated_height + child_size.height);
                    if child.layout().visibility != Visibility::Collapsed {
                        accumulated_width += self.horizontal_spacing;
                        any_horizontal_spacing = true;
                    }
                    accumulated_width += child_size.width;
                }
                Dock::Top | Dock::Bottom => {
                    parent_width = parent_width.max(accumulated_width + child_size.width);
                    if child.layout().visibility != Visibility::Collapsed {
                        accumulated_height += self.vertical_spacing;
                        any_vertical_spacing = true;
                    }
                    accumulated_height += child_size.height;
                }
            }
        }

        if self.last_child_fill && !self.children.is_empty() {
            if let Some(child) = self.children.last_mut() {
                let child_constraint = Size::new(
                    (available.width - accumulated_width).max(0.0),
                    (available.height - accumulated_height).max(0.0),
                );
                let child_size = measure_element(child.as_mut(), ctx, child_constraint);
                parent_width = parent_width.max(accumulated_width + child_size.width);
                parent_height = parent_height.max(accumulated_height + child_size.height);
                accumulated_width += child_size.width;
                accumulated_height += child_size.height;
            }
        } else {
            if any_horizontal_spacing {
                accumulated_width -= self.horizontal_spacing;
            }
            if any_vertical_spacing {
                accumulated_height -= self.vertical_spacing;
            }
        }

        Size::new(
            parent_width.max(accumulated_width),
            parent_height.max(accumulated_height),
        )
    }

    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size {
        let mut bounds = Rect::from_xywh(0.0, 0.0, final_size.width, final_size.height);
        let arrange_count = if self.last_child_fill && !self.children.is_empty() {
            self.children.len() - 1
        } else {
            self.children.len()
        };

        for child in self.children.iter_mut().take(arrange_count) {
            if child.layout().visibility == Visibility::Collapsed {
                continue;
            }

            let desired = child.layout().desired_size();
            match child.layout().attached.dock {
                Dock::Left => {
                    let width = desired.width.min(bounds.size.width);
                    arrange_element(
                        child.as_mut(),
                        ctx,
                        Rect::from_xywh(
                            bounds.origin.x,
                            bounds.origin.y,
                            width,
                            bounds.size.height,
                        ),
                    );
                    let consumed = width + self.horizontal_spacing;
                    bounds.origin.x += consumed;
                    bounds.size.width = (bounds.size.width - consumed).max(0.0);
                }
                Dock::Right => {
                    let width = desired.width.min(bounds.size.width);
                    arrange_element(
                        child.as_mut(),
                        ctx,
                        Rect::from_xywh(
                            bounds.origin.x + bounds.size.width - width,
                            bounds.origin.y,
                            width,
                            bounds.size.height,
                        ),
                    );
                    bounds.size.width =
                        (bounds.size.width - width - self.horizontal_spacing).max(0.0);
                }
                Dock::Top => {
                    let height = desired.height.min(bounds.size.height);
                    arrange_element(
                        child.as_mut(),
                        ctx,
                        Rect::from_xywh(
                            bounds.origin.x,
                            bounds.origin.y,
                            bounds.size.width,
                            height,
                        ),
                    );
                    let consumed = height + self.vertical_spacing;
                    bounds.origin.y += consumed;
                    bounds.size.height = (bounds.size.height - consumed).max(0.0);
                }
                Dock::Bottom => {
                    let height = desired.height.min(bounds.size.height);
                    arrange_element(
                        child.as_mut(),
                        ctx,
                        Rect::from_xywh(
                            bounds.origin.x,
                            bounds.origin.y + bounds.size.height - height,
                            bounds.size.width,
                            height,
                        ),
                    );
                    bounds.size.height =
                        (bounds.size.height - height - self.vertical_spacing).max(0.0);
                }
            }
        }

        if self.last_child_fill {
            if let Some(child) = self.children.last_mut() {
                arrange_element(child.as_mut(), ctx, bounds);
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
