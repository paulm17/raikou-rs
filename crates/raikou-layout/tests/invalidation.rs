use std::any::Any;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use raikou_core::Size;
use raikou_layout::{LayoutElement, LayoutManager, Layoutable, SizedBox, StackPanel};

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

    fn measure_override(&mut self, available: Size) -> Size {
        self.measures.fetch_add(1, Ordering::Relaxed);
        Size::new(
            self.size.width.min(available.width),
            self.size.height.min(available.height),
        )
    }

    fn arrange_override(&mut self, final_size: Size) -> Size {
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
    let mut node = SizedBox::new(Size::new(10.0, 10.0));
    let mut manager = LayoutManager::new();
    manager.update(&mut node, Size::new(100.0, 100.0));

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
    let mut manager = LayoutManager::new();

    manager.update(&mut node, Size::new(100.0, 100.0));
    let measured_after_first = measures.load(Ordering::Relaxed);
    let arranged_after_first = arranges.load(Ordering::Relaxed);

    node.layout_mut().invalidate_arrange();
    manager.update(&mut node, Size::new(100.0, 100.0));

    assert_eq!(measures.load(Ordering::Relaxed), measured_after_first);
    assert!(arranges.load(Ordering::Relaxed) > arranged_after_first);
}

#[test]
fn parent_remeasures_when_child_desired_size_changes() {
    let mut root = StackPanel::new();
    root.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));

    let mut manager = LayoutManager::new();
    manager.update(&mut root, Size::new(100.0, 100.0));
    assert_eq!(root.layout().desired_size(), Size::new(10.0, 10.0));

    let sized = root.children_mut()[0]
        .as_any_mut()
        .downcast_mut::<SizedBox>()
        .expect("expected sized box");
    sized.set_intrinsic_size(Size::new(40.0, 10.0));
    manager.update(&mut root, Size::new(100.0, 100.0));

    assert_eq!(root.layout().desired_size(), Size::new(40.0, 10.0));
}
