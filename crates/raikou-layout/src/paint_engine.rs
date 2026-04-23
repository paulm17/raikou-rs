use raikou_core::{PaintLayer, Point};

use crate::LayoutElement;

/// A single paint command for a layout element.
pub struct PaintCommand<'a> {
    pub element: &'a dyn LayoutElement,
    pub absolute_position: Point,
    pub layer: PaintLayer,
    pub depth: usize,
}

/// Collects paint commands from the layout tree in the correct layer order.
///
/// Content elements are ordered before overlay elements. Elements within the
/// same layer maintain tree order. Each command carries the element's
/// absolute position (accumulated from the root) and its depth in the tree.
pub fn collect_paint_commands(root: &dyn LayoutElement) -> Vec<PaintCommand<'_>> {
    let mut content = Vec::new();
    let mut overlay = Vec::new();
    collect_commands(root, Point::new(0.0, 0.0), 0, &mut content, &mut overlay);

    let mut commands = Vec::with_capacity(content.len() + overlay.len());
    commands.extend(content);
    commands.extend(overlay);
    commands
}

fn collect_commands<'a>(
    element: &'a dyn LayoutElement,
    parent_offset: Point,
    depth: usize,
    content: &mut Vec<PaintCommand<'a>>,
    overlay: &mut Vec<PaintCommand<'a>>,
) {
    let bounds = element.layout().bounds();
    let abs = Point::new(
        parent_offset.x + bounds.origin.x,
        parent_offset.y + bounds.origin.y,
    );
    let layer = element.paint_layer();

    let cmd = PaintCommand {
        element,
        absolute_position: abs,
        layer,
        depth,
    };

    match layer {
        PaintLayer::Content => content.push(cmd),
        PaintLayer::Overlay(_) => overlay.push(cmd),
    }

    element.visit_children(&mut |child| {
        // SAFETY: `child` is a reference to data owned by `element` (inside a
        // `Box<dyn LayoutElement>`). The box is stable for the lifetime of
        // `element` (`'a`), so extending the reference lifetime is sound.
        let child: &'a dyn LayoutElement = unsafe { std::mem::transmute(child) };
        collect_commands(child, abs, depth + 1, content, overlay);
    });
}
