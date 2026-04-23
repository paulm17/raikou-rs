use std::any::Any;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use raikou_core::Size;
use raikou_layout::{Canvas, LayoutContext, LayoutElement, LayoutManager, Layoutable, SizedBox, StackPanel};

struct CountingBox {
    layout: Layoutable,
    size: Size,
    measures: Arc<AtomicUsize>,
    arranges: Arc<AtomicUsize>,
}

impl CountingBox {
    fn new(size: Size, measures: Arc<AtomicUsize>, arranges: Arc<AtomicUsize>) -> Self {
        Self {
            layout: Layoutable::new(),
            size,
            measures,
            arranges,
        }
    }
}

impl LayoutElement for CountingBox {
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

    fn measure_override(&mut self, _ctx: &mut LayoutContext, available: Size) -> Size {
        self.measures.fetch_add(1, Ordering::Relaxed);
        Size::new(
            self.size.width.min(available.width),
            self.size.height.min(available.height),
        )
    }

    fn arrange_override(&mut self, _ctx: &mut LayoutContext, final_size: Size) -> Size {
        self.arranges.fetch_add(1, Ordering::Relaxed);
        final_size
    }

    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn LayoutElement)) {}

    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {}
}

#[test]
fn invalidate_measure_marks_measure_and_arrange_invalid() {
    let mut node = SizedBox::new(Size::new(10.0, 10.0));
    node.layout_mut().invalidate_measure();

    assert!(!node.layout().is_measure_valid());
    assert!(!node.layout().is_arrange_valid());
}

#[test]
fn invalidate_arrange_does_not_force_measure_invalid() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut node = SizedBox::new(Size::new(10.0, 10.0));
    let manager = LayoutManager::new();
    manager.update(&mut node, &mut ctx, Size::new(100.0, 100.0));

    node.layout_mut().invalidate_arrange();
    assert!(node.layout().is_measure_valid());
    assert!(!node.layout().is_arrange_valid());
}

#[test]
fn invalidate_arrange_rearranges_without_remeasuring_when_possible() {
    let measures = Arc::new(AtomicUsize::new(0));
    let arranges = Arc::new(AtomicUsize::new(0));
    let mut node = CountingBox::new(
        Size::new(10.0, 10.0),
        Arc::clone(&measures),
        Arc::clone(&arranges),
    );
    let manager = LayoutManager::new();

    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    manager.update(&mut node, &mut ctx, Size::new(100.0, 100.0));
    let measured_after_first = measures.load(Ordering::Relaxed);
    let arranged_after_first = arranges.load(Ordering::Relaxed);

    node.layout_mut().invalidate_arrange();
    manager.update(&mut node, &mut ctx, Size::new(100.0, 100.0));

    assert_eq!(measures.load(Ordering::Relaxed), measured_after_first);
    assert!(arranges.load(Ordering::Relaxed) > arranged_after_first);
}

#[test]
fn parent_remeasures_when_child_desired_size_changes() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(100.0, 100.0));
    assert_eq!(root.layout().desired_size(), Size::new(10.0, 10.0));

    let sized = root.children_mut()[0]
        .as_any_mut()
        .downcast_mut::<SizedBox>()
        .expect("expected sized box");
    sized.set_intrinsic_size(Size::new(40.0, 10.0));
    manager.update(&mut root, &mut ctx, Size::new(100.0, 100.0));

    assert_eq!(root.layout().desired_size(), Size::new(40.0, 10.0));
}

#[test]
fn queue_measure_bubbles_to_ancestors() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(100.0, 100.0));

    let child_id = root.children()[0].layout().id();
    manager.queue_measure(child_id);

    assert_eq!(manager.queued_measure_count(), 2);
    assert!(manager.has_dirty_work());
}

#[test]
fn update_targeted_remeasures_ancestors_when_child_queued() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(100.0, 100.0));
    assert_eq!(root.layout().desired_size(), Size::new(10.0, 10.0));

    let child_id = root.children()[0].layout().id();
    manager.queue_measure(child_id);

    let summary = manager.update_targeted(&mut root, &mut ctx, Size::new(100.0, 100.0));
    assert!(summary.measured);
    assert!(summary.arranged);
    assert_eq!(root.layout().desired_size(), Size::new(10.0, 10.0));
}

#[test]
fn bubbling_three_levels_deep() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = Canvas::new();
    let mut middle = StackPanel::new();
    let leaf = SizedBox::new(Size::new(100.0, 100.0));
    middle.push_child(Box::new(leaf));
    root.push_child(Box::new(middle));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(500.0, 500.0));

    let leaf_id = root.children()[0].as_any().downcast_ref::<StackPanel>().unwrap().children()[0]
        .layout()
        .id();
    manager.queue_measure(leaf_id);

    // Leaf + StackPanel + Panel = 3 queued
    assert_eq!(manager.queued_measure_count(), 3);

    let summary = manager.update_targeted(&mut root, &mut ctx, Size::new(500.0, 500.0));
    assert!(summary.measured);
    assert!(summary.arranged);
}

#[test]
fn invalidate_measure_on_leaf_bubbles_to_ancestors_and_triggers_targeted_update() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(100.0, 100.0));
    assert_eq!(root.layout().desired_size(), Size::new(10.0, 10.0));

    let sized = root.children_mut()[0]
        .as_any_mut()
        .downcast_mut::<SizedBox>()
        .expect("expected sized box");
    sized.set_intrinsic_size(Size::new(40.0, 10.0));

    let summary = manager.update_targeted(&mut root, &mut ctx, Size::new(100.0, 100.0));
    assert!(summary.measured);
    assert!(summary.arranged);
    assert_eq!(root.layout().desired_size(), Size::new(40.0, 10.0));
}

#[test]
fn invalidate_measure_bubbles_three_levels_deep_for_targeted_update() {
    let mut font_system = raikou_layout::FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let mut root = StackPanel::new();
    let mut middle = StackPanel::new();
    let leaf = SizedBox::new(Size::new(100.0, 100.0));
    middle.push_child(Box::new(leaf));
    root.push_child(Box::new(middle));

    let manager = LayoutManager::new();
    manager.update(&mut root, &mut ctx, Size::new(500.0, 500.0));
    assert_eq!(root.layout().desired_size(), Size::new(100.0, 100.0));

    let leaf = root.children_mut()[0]
        .as_any_mut()
        .downcast_mut::<StackPanel>()
        .unwrap()
        .children_mut()[0]
        .as_any_mut()
        .downcast_mut::<SizedBox>()
        .unwrap();
    leaf.set_intrinsic_size(Size::new(200.0, 200.0));

    let summary = manager.update_targeted(&mut root, &mut ctx, Size::new(500.0, 500.0));
    assert!(summary.measured);
    assert!(summary.arranged);
    assert_eq!(root.layout().desired_size(), Size::new(200.0, 200.0));
}
