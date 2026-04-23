use crate::geometry::{Color, Point};
use crate::ids::WidgetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintOrder {
    ParentBeforeChildren,
}

impl PaintOrder {
    pub const fn describes_retained_tree_order(self) -> bool {
        matches!(self, Self::ParentBeforeChildren)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintLayer {
    Content,
    Overlay(OverlayPaintPhase),
}

impl PaintLayer {
    pub const fn order_key(self) -> u8 {
        match self {
            Self::Content => 0,
            Self::Overlay(_) => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayPaintPhase {
    AfterContent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaintEntry {
    pub widget_id: WidgetId,
    pub layer: PaintLayer,
}

impl PaintEntry {
    pub const fn new(widget_id: WidgetId, layer: PaintLayer) -> Self {
        Self { widget_id, layer }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowPaintList {
    content: Vec<PaintEntry>,
    overlay: Vec<PaintEntry>,
}

impl WindowPaintList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, widget_id: WidgetId, layer: PaintLayer) {
        let entry = PaintEntry::new(widget_id, layer);
        match layer {
            PaintLayer::Content => self.content.push(entry),
            PaintLayer::Overlay(_) => self.overlay.push(entry),
        }
    }

    pub fn content_entries(&self) -> &[PaintEntry] {
        &self.content
    }

    pub fn overlay_entries(&self) -> &[PaintEntry] {
        &self.overlay
    }

    pub fn ordered(&self) -> Vec<PaintEntry> {
        let mut ordered = Vec::with_capacity(self.content.len() + self.overlay.len());
        ordered.extend_from_slice(&self.content);
        ordered.extend_from_slice(&self.overlay);
        ordered
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

impl GradientStop {
    pub const fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    pub fn new(start: Point, end: Point, stops: Vec<GradientStop>) -> Self {
        Self { start, end, stops }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageFit {
    #[default]
    Fill,
    Contain,
    Cover,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_layers_reserve_overlay_path_after_content() {
        assert!(
            PaintLayer::Content.order_key()
                < PaintLayer::Overlay(OverlayPaintPhase::AfterContent).order_key()
        );
        assert!(PaintOrder::ParentBeforeChildren.describes_retained_tree_order());
    }

    #[test]
    fn window_paint_list_orders_overlay_after_content() {
        let content = WidgetId::next();
        let overlay = WidgetId::next();

        let mut list = WindowPaintList::new();
        list.register(content, PaintLayer::Content);
        list.register(
            overlay,
            PaintLayer::Overlay(OverlayPaintPhase::AfterContent),
        );

        let ordered = list.ordered();
        assert_eq!(ordered[0].widget_id, content);
        assert_eq!(ordered[1].widget_id, overlay);
    }
}
