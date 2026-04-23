use raikou_core::{Rect, Size};
use raikou_layout::{
    ItemsHost, ScrollContentPresenter, SizedBox, VirtualizationHost, arrange_element,
    measure_element,
};

#[test]
fn items_host_exposes_realized_range_contract() {
    let mut host = ItemsHost::new();
    host.push_child(Box::new(SizedBox::new(Size::new(10.0, 10.0))));
    host.push_child(Box::new(SizedBox::new(Size::new(20.0, 20.0))));

    assert_eq!(host.realized_range(), 0..2);
    assert_eq!(
        measure_element(&mut host, Size::new(50.0, 50.0)),
        Size::new(20.0, 20.0)
    );
}

#[test]
fn scroll_content_presenter_tracks_viewport_and_extent() {
    let mut presenter = ScrollContentPresenter::new();
    presenter.set_child(Box::new(SizedBox::new(Size::new(200.0, 150.0))));

    assert_eq!(
        measure_element(&mut presenter, Size::new(100.0, 80.0)),
        Size::new(100.0, 80.0)
    );
    arrange_element(&mut presenter, Rect::from_xywh(0.0, 0.0, 100.0, 80.0));

    assert_eq!(presenter.viewport(), Size::new(100.0, 80.0));
    assert_eq!(presenter.extent(), Size::new(200.0, 150.0));
}
