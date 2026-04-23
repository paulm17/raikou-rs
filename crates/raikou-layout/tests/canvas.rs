use raikou_core::{Rect, Size};
use raikou_layout::{Canvas, LayoutElement, SizedBox, arrange_element, measure_element};

#[test]
fn avalonia_canvas_honors_attached_left_top_and_right_bottom() {
    let mut a = SizedBox::new(Size::new(20.0, 10.0));
    a.layout_mut().attached.canvas.left = Some(5.0);
    a.layout_mut().attached.canvas.top = Some(7.0);

    let mut b = SizedBox::new(Size::new(10.0, 15.0));
    b.layout_mut().attached.canvas.right = Some(8.0);
    b.layout_mut().attached.canvas.bottom = Some(3.0);

    let mut canvas = Canvas::new();
    canvas.push_child(Box::new(a));
    canvas.push_child(Box::new(b));

    assert_eq!(
        measure_element(&mut canvas, Size::new(100.0, 100.0)),
        Size::ZERO
    );
    arrange_element(&mut canvas, Rect::from_xywh(0.0, 0.0, 100.0, 60.0));

    assert_eq!(
        canvas.children()[0].layout().bounds(),
        Rect::from_xywh(5.0, 7.0, 20.0, 10.0)
    );
    assert_eq!(
        canvas.children()[1].layout().bounds(),
        Rect::from_xywh(82.0, 42.0, 10.0, 15.0)
    );
}

#[test]
fn avalonia_canvas_preserves_child_order_for_overlapping_children() {
    let mut canvas = Canvas::new();
    canvas.push_child(Box::new(SizedBox::new(Size::new(20.0, 20.0))));
    canvas.push_child(Box::new(SizedBox::new(Size::new(20.0, 20.0))));
    canvas.push_child(Box::new(SizedBox::new(Size::new(20.0, 20.0))));

    assert_eq!(canvas.children().len(), 3);
    assert!(canvas.children()[0].layout().id().get() < canvas.children()[1].layout().id().get());
    assert!(canvas.children()[1].layout().id().get() < canvas.children()[2].layout().id().get());
}
