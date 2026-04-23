use std::collections::BTreeSet;

use raikou_core::{PaintLayer, Rect, Size, WidgetId, WindowPaintList};

use crate::alignment::{HorizontalAlignment, VerticalAlignment};
use crate::layoutable::{
    align_axis, arrange_axis, arrange_element, measure_element, round_layout_slot, sanitize_rect,
    sanitize_size, LayoutElement, Visibility,
};
use crate::rounding::{round_rect, round_size};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutPassSummary {
    pub measured: bool,
    pub arranged: bool,
    pub queued_measure_count: usize,
    pub queued_arrange_count: usize,
}

#[derive(Default)]
pub struct LayoutManager {
    dirty_measure: BTreeSet<WidgetId>,
    dirty_arrange: BTreeSet<WidgetId>,
    last_available: Option<Size>,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_measure(&mut self, id: WidgetId) {
        self.dirty_measure.insert(id);
        self.dirty_arrange.insert(id);
    }

    pub fn queue_arrange(&mut self, id: WidgetId) {
        self.dirty_arrange.insert(id);
    }

    pub fn queued_measure_count(&self) -> usize {
        self.dirty_measure.len()
    }

    pub fn queued_arrange_count(&self) -> usize {
        self.dirty_arrange.len()
    }

    pub fn has_dirty_work(&self) -> bool {
        !(self.dirty_measure.is_empty() && self.dirty_arrange.is_empty())
    }

    pub fn purge_widget(&mut self, id: WidgetId) {
        self.dirty_measure.remove(&id);
        self.dirty_arrange.remove(&id);
    }

    pub fn update(&mut self, root: &mut dyn LayoutElement, available: Size) -> LayoutPassSummary {
        let available_changed = self.last_available != Some(available);
        let queued_measure_count = self.dirty_measure.len();
        let queued_arrange_count = self.dirty_arrange.len();

        let should_measure =
            available_changed || !self.dirty_measure.is_empty() || needs_measure(root);
        let should_arrange = available_changed
            || should_measure
            || !self.dirty_arrange.is_empty()
            || needs_arrange(root);

        if should_measure {
            measure_element(root, available);
        }

        if should_arrange {
            arrange_element(
                root,
                Rect::from_xywh(0.0, 0.0, available.width, available.height),
            );
        }

        self.last_available = Some(available);
        self.dirty_measure.clear();
        self.dirty_arrange.clear();

        LayoutPassSummary {
            measured: should_measure,
            arranged: should_arrange,
            queued_measure_count,
            queued_arrange_count,
        }
    }

    pub fn update_targeted(
        &mut self,
        root: &mut dyn LayoutElement,
        available: Size,
    ) -> LayoutPassSummary {
        let available_changed = self.last_available != Some(available);
        let queued_measure_count = self.dirty_measure.len();
        let queued_arrange_count = self.dirty_arrange.len();

        let has_any_dirty_measure = !self.dirty_measure.is_empty();
        let has_any_dirty_arrange = !self.dirty_arrange.is_empty();

        if available_changed || has_any_dirty_measure {
            self.measure_element_targeted(root, available, None, has_any_dirty_measure);
        }

        if available_changed || has_any_dirty_arrange || has_any_dirty_measure {
            self.arrange_element_targeted(
                root,
                Rect::from_xywh(0.0, 0.0, available.width, available.height),
                None,
                has_any_dirty_arrange || has_any_dirty_measure,
            );
        }

        self.last_available = Some(available);
        self.dirty_measure.clear();
        self.dirty_arrange.clear();

        LayoutPassSummary {
            measured: has_any_dirty_measure || available_changed,
            arranged: has_any_dirty_arrange || has_any_dirty_measure || available_changed,
            queued_measure_count,
            queued_arrange_count,
        }
    }

    fn measure_element_targeted(
        &mut self,
        element: &mut dyn LayoutElement,
        available: Size,
        ancestor_dirty: Option<bool>,
        force_measure: bool,
    ) {
        let available = sanitize_size(available);

        let is_dirty = self.dirty_measure.contains(&element.layout().id());
        let should_proceed = force_measure
            || is_dirty
            || ancestor_dirty == Some(true)
            || !element.layout().measure_valid
            || element.layout().previous_measure != Some(available);

        if !should_proceed {
            return;
        }

        if element.layout().visibility == Visibility::Collapsed {
            let layout = element.layout_mut();
            layout.desired_size = Size::ZERO;
            layout.previous_measure = Some(available);
            layout.measure_valid = true;
            layout.arrange_valid = false;
            return;
        }

        let (margin, width, height, constraints, round) = {
            let layout = element.layout();
            (
                layout.margin,
                layout.width,
                layout.height,
                layout.constraints,
                layout.use_layout_rounding,
            )
        };

        let inner_available = margin.deflate_size(available);
        let override_available = Size::new(
            width
                .unwrap_or(inner_available.width)
                .min(inner_available.width),
            height
                .unwrap_or(inner_available.height)
                .min(inner_available.height),
        );

        let measured = element.measure_override(constraints.constrain(override_available));
        let explicit = Size::new(
            width.unwrap_or(measured.width),
            height.unwrap_or(measured.height),
        );
        let constrained = constraints.clamp(explicit);
        let mut desired = Size::new(
            constrained.width + margin.horizontal(),
            constrained.height + margin.vertical(),
        );
        desired = sanitize_size(desired);
        if round {
            desired = round_size(desired);
        }

        let layout = element.layout_mut();
        layout.desired_size = desired;
        layout.previous_measure = Some(available);
        layout.measure_valid = true;
        layout.arrange_valid = false;

        // Do NOT re-visit children here. measure_override already measured every
        // child with the element-specific available size (e.g. ScrollArea gives
        // its child f32::INFINITY). Re-visiting them with the outer `available`
        // would force an incorrect re-measure whenever ancestor_dirty is true.
    }

    fn arrange_element_targeted(
        &mut self,
        element: &mut dyn LayoutElement,
        final_rect: Rect,
        ancestor_dirty: Option<bool>,
        force_arrange: bool,
    ) {
        let final_rect = sanitize_rect(final_rect);

        let is_dirty = self.dirty_arrange.contains(&element.layout().id())
            || self.dirty_measure.contains(&element.layout().id());
        let should_proceed = force_arrange
            || is_dirty
            || ancestor_dirty == Some(true)
            || !element.layout().arrange_valid
            || element.layout().previous_arrange != Some(final_rect);

        if !should_proceed {
            return;
        }

        if element.layout().visibility == Visibility::Collapsed {
            let layout = element.layout_mut();
            layout.layout_slot = final_rect;
            layout.bounds = Rect::new(final_rect.origin, Size::ZERO);
            layout.previous_arrange = Some(final_rect);
            layout.arrange_valid = true;
            return;
        }

        let (
            margin,
            width,
            height,
            constraints,
            horizontal_alignment,
            vertical_alignment,
            round,
            desired_size,
        ) = {
            let layout = element.layout();
            (
                layout.margin,
                layout.width,
                layout.height,
                layout.constraints,
                layout.horizontal_alignment,
                layout.vertical_alignment,
                layout.use_layout_rounding,
                layout.desired_size,
            )
        };

        let slot = if round {
            round_layout_slot(margin.deflate_rect(final_rect))
        } else {
            margin.deflate_rect(final_rect)
        };
        let desired_inner = margin.deflate_size(desired_size);
        let mut arranged_size = Size::new(
            arrange_axis(
                slot.size.width,
                desired_inner.width,
                width,
                constraints.min.width,
                constraints.max.width,
                horizontal_alignment == HorizontalAlignment::Stretch,
            ),
            arrange_axis(
                slot.size.height,
                desired_inner.height,
                height,
                constraints.min.height,
                constraints.max.height,
                vertical_alignment == VerticalAlignment::Stretch,
            ),
        );

        arranged_size = sanitize_size(arranged_size);
        let mut arranged_rect = Rect::from_xywh(
            align_axis(
                slot.origin.x,
                slot.size.width,
                arranged_size.width,
                horizontal_alignment,
            ),
            align_axis(
                slot.origin.y,
                slot.size.height,
                arranged_size.height,
                vertical_alignment,
            ),
            arranged_size.width,
            arranged_size.height,
        );

        if round {
            arranged_rect = round_rect(arranged_rect);
        }

        let arranged_size = element.arrange_override(arranged_rect.size);
        let bounds = Rect::new(arranged_rect.origin, arranged_size);
        let layout = element.layout_mut();
        layout.bounds = if round { round_rect(bounds) } else { bounds };

        let element_rect = final_rect;
        layout.previous_arrange = Some(element_rect);
        layout.arrange_valid = true;
    }

    pub fn collect_window_paint_list(&self, root: &dyn LayoutElement) -> WindowPaintList {
        let mut list = WindowPaintList::new();
        collect_paint_list(root, inherited_layer(root.paint_layer()), &mut list);
        list
    }
}

fn needs_measure(element: &dyn LayoutElement) -> bool {
    if !element.layout().is_measure_valid() {
        return true;
    }

    let mut needed = false;
    element.visit_children(&mut |child| {
        if !needed && needs_measure(child) {
            needed = true;
        }
    });
    needed
}

fn needs_arrange(element: &dyn LayoutElement) -> bool {
    if !element.layout().is_arrange_valid() {
        return true;
    }

    let mut needed = false;
    element.visit_children(&mut |child| {
        if !needed && needs_arrange(child) {
            needed = true;
        }
    });
    needed
}

fn collect_paint_list(
    element: &dyn LayoutElement,
    current_layer: PaintLayer,
    list: &mut WindowPaintList,
) {
    let effective_layer = match element.paint_layer() {
        PaintLayer::Content => current_layer,
        overlay @ PaintLayer::Overlay(_) => overlay,
    };

    list.register(element.layout().id(), effective_layer);
    element.visit_children(&mut |child| collect_paint_list(child, effective_layer, list));
}

fn inherited_layer(layer: PaintLayer) -> PaintLayer {
    match layer {
        PaintLayer::Content => PaintLayer::Content,
        PaintLayer::Overlay(phase) => PaintLayer::Overlay(phase),
    }
}
