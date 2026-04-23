use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use raikou_core::{PaintLayer, Rect, Size, WidgetId, WindowPaintList};

use crate::alignment::{HorizontalAlignment, VerticalAlignment};
use crate::layoutable::{
    align_axis, arrange_axis, arrange_element, measure_element, round_layout_slot, sanitize_rect,
    sanitize_size, subtree_needs_arrange, subtree_needs_measure, LayoutContext, LayoutElement,
    Visibility,
};
use crate::rounding::{round_rect, round_size};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutPassSummary {
    pub measured: bool,
    pub arranged: bool,
    pub queued_measure_count: usize,
    pub queued_arrange_count: usize,
}

#[derive(Default, Clone)]
pub struct LayoutManager {
    dirty_measure: Arc<RefCell<BTreeSet<WidgetId>>>,
    dirty_arrange: Arc<RefCell<BTreeSet<WidgetId>>>,
    last_available: Cell<Option<Size>>,
    parent_map: Arc<RefCell<HashMap<WidgetId, WidgetId>>>,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_measure(&self, id: WidgetId) {
        self.queue_measure_recursive(id);
    }

    fn queue_measure_recursive(&self, id: WidgetId) {
        let mut dirty = self.dirty_measure.borrow_mut();
        if !dirty.insert(id) {
            return;
        }
        drop(dirty);
        self.dirty_arrange.borrow_mut().insert(id);
        if let Some(parent) = self.parent_map.borrow().get(&id).copied() {
            self.queue_measure_recursive(parent);
        }
    }

    pub fn queue_arrange(&self, id: WidgetId) {
        self.queue_arrange_recursive(id);
    }

    fn queue_arrange_recursive(&self, id: WidgetId) {
        let mut dirty = self.dirty_arrange.borrow_mut();
        if !dirty.insert(id) {
            return;
        }
        drop(dirty);
        if let Some(parent) = self.parent_map.borrow().get(&id).copied() {
            self.queue_arrange_recursive(parent);
        }
    }

    pub fn queued_measure_count(&self) -> usize {
        self.dirty_measure.borrow().len()
    }

    pub fn queued_arrange_count(&self) -> usize {
        self.dirty_arrange.borrow().len()
    }

    pub fn has_dirty_work(&self) -> bool {
        !(self.dirty_measure.borrow().is_empty() && self.dirty_arrange.borrow().is_empty())
    }

    pub fn purge_widget(&self, id: WidgetId) {
        self.dirty_measure.borrow_mut().remove(&id);
        self.dirty_arrange.borrow_mut().remove(&id);
    }

    fn create_invalidation_callback(&self) -> Arc<dyn Fn(WidgetId, bool)> {
        let measure_set = Arc::clone(&self.dirty_measure);
        let arrange_set = Arc::clone(&self.dirty_arrange);
        let parent_map = Arc::clone(&self.parent_map);
        Arc::new(move |id, is_measure| {
            if is_measure {
                bubble_measure(id, &measure_set, &arrange_set, &parent_map);
            } else {
                bubble_arrange(id, &arrange_set, &parent_map);
            }
        })
    }

    fn build_tree_state(&self, element: &mut dyn LayoutElement, cb: &Arc<dyn Fn(WidgetId, bool)>) {
        let parent_id = element.layout().id();
        element.layout_mut().set_invalidation_callback(Some(Arc::clone(cb)));
        element.visit_children_mut(&mut |child| {
            let child_id = child.layout().id();
            self.parent_map.borrow_mut().insert(child_id, parent_id);
            self.build_tree_state(child, cb);
        });
    }

    pub fn update(&self, root: &mut dyn LayoutElement, ctx: &mut LayoutContext, available: Size) -> LayoutPassSummary {
        let cb = self.create_invalidation_callback();
        self.parent_map.borrow_mut().clear();
        self.build_tree_state(root, &cb);

        let available_changed = self.last_available.get() != Some(available);
        let queued_measure_count = self.dirty_measure.borrow().len();
        let queued_arrange_count = self.dirty_arrange.borrow().len();

        let should_measure = available_changed
            || !self.dirty_measure.borrow().is_empty()
            || needs_measure(root);
        let should_arrange = available_changed
            || should_measure
            || !self.dirty_arrange.borrow().is_empty()
            || needs_arrange(root);

        if should_measure {
            measure_element(root, ctx, available);
        }

        if should_arrange {
            arrange_element(
                root,
                ctx,
                Rect::from_xywh(0.0, 0.0, available.width, available.height),
            );
        }

        self.last_available.set(Some(available));
        self.dirty_measure.borrow_mut().clear();
        self.dirty_arrange.borrow_mut().clear();

        LayoutPassSummary {
            measured: should_measure,
            arranged: should_arrange,
            queued_measure_count,
            queued_arrange_count,
        }
    }

    pub fn update_targeted(
        &self,
        root: &mut dyn LayoutElement,
        ctx: &mut LayoutContext,
        available: Size,
    ) -> LayoutPassSummary {
        let cb = self.create_invalidation_callback();
        self.parent_map.borrow_mut().clear();
        self.build_tree_state(root, &cb);

        let available_changed = self.last_available.get() != Some(available);
        let queued_measure_count = self.dirty_measure.borrow().len();
        let queued_arrange_count = self.dirty_arrange.borrow().len();

        let has_any_dirty_measure = !self.dirty_measure.borrow().is_empty();
        let has_any_dirty_arrange = !self.dirty_arrange.borrow().is_empty();

        let should_measure = available_changed
            || has_any_dirty_measure
            || needs_measure(root);
        let should_arrange = available_changed
            || has_any_dirty_arrange
            || has_any_dirty_measure
            || needs_arrange(root);

        if should_measure {
            self.measure_element_targeted(root, ctx, available, None, has_any_dirty_measure);
        }

        if should_arrange {
            self.arrange_element_targeted(
                root,
                ctx,
                Rect::from_xywh(0.0, 0.0, available.width, available.height),
                None,
                has_any_dirty_arrange || has_any_dirty_measure,
            );
        }

        self.last_available.set(Some(available));
        self.dirty_measure.borrow_mut().clear();
        self.dirty_arrange.borrow_mut().clear();

        LayoutPassSummary {
            measured: should_measure,
            arranged: should_arrange,
            queued_measure_count,
            queued_arrange_count,
        }
    }

    fn measure_element_targeted(
        &self,
        element: &mut dyn LayoutElement,
        ctx: &mut LayoutContext,
        available: Size,
        ancestor_dirty: Option<bool>,
        force_measure: bool,
    ) {
        let available = sanitize_size(available);

        let is_dirty = self.dirty_measure.borrow().contains(&element.layout().id());
        let should_proceed = force_measure
            || is_dirty
            || ancestor_dirty == Some(true)
            || !element.layout().measure_valid
            || element.layout().previous_measure != Some(available)
            || subtree_needs_measure(element);

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

        let measured = element.measure_override(ctx, constraints.constrain(override_available));
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

        element.visit_children_mut(&mut |child| {
            let child_id = child.layout().id();
            let child_dirty = self.dirty_measure.borrow().contains(&child_id);
            let child_needs = child_dirty
                || !child.layout().measure_valid
                || subtree_needs_measure(child);
            if child_needs {
                let child_available = child.layout().previous_measure.unwrap_or(available);
                self.measure_element_targeted(child, ctx, child_available, None, false);
            }
        });
    }

    fn arrange_element_targeted(
        &self,
        element: &mut dyn LayoutElement,
        ctx: &mut LayoutContext,
        final_rect: Rect,
        ancestor_dirty: Option<bool>,
        force_arrange: bool,
    ) {
        let final_rect = sanitize_rect(final_rect);

        let is_dirty = self.dirty_arrange.borrow().contains(&element.layout().id())
            || self.dirty_measure.borrow().contains(&element.layout().id());
        let should_proceed = force_arrange
            || is_dirty
            || ancestor_dirty == Some(true)
            || !element.layout().arrange_valid
            || element.layout().previous_arrange != Some(final_rect)
            || subtree_needs_arrange(element);

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

        let arranged_size = element.arrange_override(ctx, arranged_rect.size);
        let bounds = Rect::new(arranged_rect.origin, arranged_size);
        let layout = element.layout_mut();
        layout.layout_slot = final_rect;
        layout.bounds = if round { round_rect(bounds) } else { bounds };
        layout.previous_arrange = Some(final_rect);
        layout.arrange_valid = true;

        element.visit_children_mut(&mut |child| {
            let child_id = child.layout().id();
            let child_dirty = self.dirty_arrange.borrow().contains(&child_id)
                || self.dirty_measure.borrow().contains(&child_id);
            let child_needs = child_dirty
                || !child.layout().arrange_valid
                || subtree_needs_arrange(child);
            if child_needs {
                let child_final_rect = child.layout().previous_arrange.unwrap_or(final_rect);
                self.arrange_element_targeted(child, ctx, child_final_rect, None, false);
            }
        });
    }

    pub fn collect_window_paint_list(&self, root: &dyn LayoutElement) -> WindowPaintList {
        let mut list = WindowPaintList::new();
        collect_paint_list(root, inherited_layer(root.paint_layer()), &mut list);
        list
    }
}

fn bubble_measure(
    id: WidgetId,
    measure_set: &RefCell<BTreeSet<WidgetId>>,
    arrange_set: &RefCell<BTreeSet<WidgetId>>,
    parent_map: &RefCell<HashMap<WidgetId, WidgetId>>,
) {
    let mut dirty = measure_set.borrow_mut();
    if !dirty.insert(id) {
        return;
    }
    drop(dirty);
    arrange_set.borrow_mut().insert(id);
    if let Some(parent) = parent_map.borrow().get(&id).copied() {
        bubble_measure(parent, measure_set, arrange_set, parent_map);
    }
}

fn bubble_arrange(
    id: WidgetId,
    arrange_set: &RefCell<BTreeSet<WidgetId>>,
    parent_map: &RefCell<HashMap<WidgetId, WidgetId>>,
) {
    let mut dirty = arrange_set.borrow_mut();
    if !dirty.insert(id) {
        return;
    }
    drop(dirty);
    if let Some(parent) = parent_map.borrow().get(&id).copied() {
        bubble_arrange(parent, arrange_set, parent_map);
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
