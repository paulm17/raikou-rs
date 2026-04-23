use raikou_core::{Point, Rect, Size, WidgetId, WindowId};
use raikou_layout::{FontSystem, LayoutContext, LayoutElement, SizedBox, StackPanel};
use raikou_skia::Painter;
use raikou_text::SwashCache;
use raikou_window::{
    KeyState, KeyboardEvent, PointerButton, PointerEvent, PointerEventKind,
};
use raikou_widgets::{Button, Label, TextBoxWidget, Widget, WidgetTree};
use std::cell::RefCell;
use std::rc::Rc;

struct MinimalWidget {
    inner: SizedBox,
}

impl MinimalWidget {
    fn new(size: Size) -> Self {
        Self {
            inner: SizedBox::new(size),
        }
    }
}

impl Widget for MinimalWidget {
    fn as_layout_element(&self) -> &dyn LayoutElement {
        &self.inner
    }

    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement {
        &mut self.inner
    }

    fn paint(
        &self,
        _painter: &Painter<'_>,
        _bounds: Rect,
        _font_system: &mut FontSystem,
        _swash_cache: &mut SwashCache,
    ) {
        // no-op for test
    }
}

struct TrackingWidget {
    inner: SizedBox,
    pointer_events: Rc<RefCell<Vec<PointerEvent>>>,
    keyboard_events: Rc<RefCell<Vec<KeyboardEvent>>>,
}

impl TrackingWidget {
    fn new(size: Size) -> Self {
        Self {
            inner: SizedBox::new(size),
            pointer_events: Rc::new(RefCell::new(Vec::new())),
            keyboard_events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn pointer_events(&self) -> Rc<RefCell<Vec<PointerEvent>>> {
        self.pointer_events.clone()
    }

    fn keyboard_events(&self) -> Rc<RefCell<Vec<KeyboardEvent>>> {
        self.keyboard_events.clone()
    }
}

impl Widget for TrackingWidget {
    fn as_layout_element(&self) -> &dyn LayoutElement {
        &self.inner
    }

    fn as_layout_element_mut(&mut self) -> &mut dyn LayoutElement {
        &mut self.inner
    }

    fn on_pointer_event(&mut self, event: &PointerEvent) {
        self.pointer_events.borrow_mut().push(event.clone());
    }

    fn on_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.keyboard_events.borrow_mut().push(event.clone());
    }

    fn paint(
        &self,
        _painter: &Painter<'_>,
        _bounds: Rect,
        _font_system: &mut FontSystem,
        _swash_cache: &mut SwashCache,
    ) {
    }
}

#[test]
fn widget_tree_measure_arrange_paint_sets_bounds() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = MinimalWidget::new(Size::new(100.0, 50.0));
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let bounds = tree.root().as_layout_element().layout().bounds();
    // SizedBox defaults to Stretch alignment, so it fills the arranged slot.
    assert_eq!(bounds, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let mut surface = skia_safe::surfaces::raster_n32_premul((4, 4)).expect("surface");
    let painter = Painter::new(surface.canvas());
    let mut swash_cache = SwashCache::new();
    tree.paint(&painter, &mut font_system, &mut swash_cache); // assert no panic
}

#[test]
fn hit_test_finds_widget_when_point_inside() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(10.0, 20.0, 200.0, 200.0));

    let id = tree.root().as_layout_element().layout().id();
    assert_eq!(tree.hit_test(Point::new(50.0, 40.0)), Some(id));
}

#[test]
fn hit_test_returns_none_when_point_outside() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(10.0, 20.0, 200.0, 200.0));

    assert_eq!(tree.hit_test(Point::new(5.0, 15.0)), None);
}

#[test]
fn pointer_event_routes_to_hit_widget() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let events = widget.pointer_events();
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(10.0, 20.0, 200.0, 200.0));

    let event = PointerEvent {
        window_id: WindowId::next(),
        position: Some((50.0, 40.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Down,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    };
    tree.on_pointer_event(&event);

    assert_eq!(events.borrow().len(), 1);
    assert_eq!(events.borrow()[0].kind, event.kind);
}

#[test]
fn pointer_event_misses_when_outside_bounds() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let events = widget.pointer_events();
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(10.0, 20.0, 200.0, 200.0));

    let event = PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 15.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Down,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    };
    tree.on_pointer_event(&event);

    assert_eq!(events.borrow().len(), 0);
}

#[test]
fn keyboard_event_routes_to_focused_widget() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let events = widget.keyboard_events();
    let id = widget.as_layout_element().layout().id();
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    assert!(tree.focus(id));
    let event = KeyboardEvent {
        window_id: WindowId::next(),
        state: KeyState::Down,
        repeat: false,
        logical_key: "a".to_string(),
        physical_key: "KeyA".to_string(),
        text: Some("a".to_string()),
        modifiers: Default::default(),
    };
    tree.on_keyboard_event(&event);

    assert_eq!(events.borrow().len(), 1);
    assert_eq!(events.borrow()[0].logical_key, "a");
}

#[test]
fn keyboard_event_dropped_without_focus() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TrackingWidget::new(Size::new(100.0, 50.0));
    let events = widget.keyboard_events();
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let event = KeyboardEvent {
        window_id: WindowId::next(),
        state: KeyState::Down,
        repeat: false,
        logical_key: "a".to_string(),
        physical_key: "KeyA".to_string(),
        text: Some("a".to_string()),
        modifiers: Default::default(),
    };
    tree.on_keyboard_event(&event);

    assert_eq!(events.borrow().len(), 0);
}

#[test]
fn focus_rejects_unknown_widget() {
    let widget = MinimalWidget::new(Size::new(100.0, 50.0));
    let mut tree = WidgetTree::new(Box::new(widget));
    let fake_id = WidgetId::next();
    assert!(!tree.focus(fake_id));
    assert_eq!(tree.focused_widget(), None);
}

// ---------------------------------------------------------------------------
// Concrete widget tests
// ---------------------------------------------------------------------------

#[test]
fn label_wraps_text_block() {
    let mut label = Label::new("Hello");
    assert_eq!(label.text(), "Hello");

    label.set_text("World");
    assert_eq!(label.text(), "World");
}

#[test]
fn label_layout_element_is_text_block() {
    let label = Label::new("Test");
    let layout = label.as_layout_element().layout();
    assert!(!layout.is_measure_valid());
}

#[test]
fn label_paints_without_panic() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let label = Label::new("Hi");
    let mut tree = WidgetTree::new(Box::new(label));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let mut surface = skia_safe::surfaces::raster_n32_premul((100, 50)).expect("surface");
    let painter = Painter::new(surface.canvas());
    let mut swash_cache = SwashCache::new();
    tree.paint(&painter, &mut font_system, &mut swash_cache);
}

#[test]
fn button_has_text_and_background() {
    let mut button = Button::new("Click me");
    assert_eq!(button.text(), "Click me");

    button.set_text("Done");
    assert_eq!(button.text(), "Done");
    assert!(!button.is_pressed());
    assert!(!button.is_hovered());
}

#[test]
fn button_changes_background_on_press() {
    let mut button = Button::new("Ok");
    let normal = button.current_bg();

    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Down,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });
    assert!(button.is_pressed());
    let pressed = button.current_bg();
    assert_ne!(pressed, normal);

    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Up,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });
    assert!(!button.is_pressed());
    button.set_hovered(false);
    let released = button.current_bg();
    assert_eq!(released, normal);
}

#[test]
fn button_click_fires_callback() {
    let clicked = Rc::new(RefCell::new(false));
    let clicked_clone = clicked.clone();

    let mut button = Button::new("Ok");
    button.on_click(move || {
        *clicked_clone.borrow_mut() = true;
    });

    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Down,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });
    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Up,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });

    assert!(*clicked.borrow());
}

#[test]
fn button_click_outside_does_not_fire() {
    let clicked = Rc::new(RefCell::new(false));
    let clicked_clone = clicked.clone();

    let mut button = Button::new("Ok");
    button.on_click(move || {
        *clicked_clone.borrow_mut() = true;
    });

    // press inside
    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Down,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });

    // release outside — in a real tree the widget would not receive the
    // event at all because hit-test would miss, but we test the widget
    // state machine directly here.
    button.set_hovered(false);
    button.on_pointer_event(&PointerEvent {
        window_id: WindowId::next(),
        position: Some((5.0, 5.0)),
        kind: PointerEventKind::Button {
            state: KeyState::Up,
            button: PointerButton::Left,
        },
        modifiers: Default::default(),
    });

    assert!(!*clicked.borrow());
}

#[test]
fn button_paints_without_panic() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let button = Button::new("Ok");
    let mut tree = WidgetTree::new(Box::new(button));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let mut surface = skia_safe::surfaces::raster_n32_premul((100, 50)).expect("surface");
    let painter = Painter::new(surface.canvas());
    let mut swash_cache = SwashCache::new();
    tree.paint(&painter, &mut font_system, &mut swash_cache);
}

#[test]
fn text_box_widget_exists() {
    let mut widget = TextBoxWidget::new();
    assert!(widget.text().is_empty());

    let mut font_system = FontSystem::new();
    widget.set_text(&mut font_system, "hello");
    assert_eq!(widget.text(), "hello");
}

#[test]
fn text_box_widget_paints_without_panic() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);
    let widget = TextBoxWidget::new();
    let mut tree = WidgetTree::new(Box::new(widget));

    tree.measure(&mut ctx, Size::new(200.0, 200.0));
    tree.arrange(&mut ctx, Rect::from_xywh(0.0, 0.0, 200.0, 200.0));

    let mut surface = skia_safe::surfaces::raster_n32_premul((100, 50)).expect("surface");
    let painter = Painter::new(surface.canvas());
    let mut swash_cache = SwashCache::new();
    tree.paint(&painter, &mut font_system, &mut swash_cache);
}

#[test]
fn label_inside_stack_panel_grows_panel() {
    let mut font_system = FontSystem::new();
    let mut ctx = LayoutContext::new(&mut font_system);

    let mut panel = StackPanel::new();
    panel.push_child(Box::new(Label::new("Hello")));

    let size = raikou_layout::measure_element(&mut panel, &mut ctx, Size::new(400.0, 400.0));
    assert!(size.width > 0.0);
    assert!(size.height > 0.0);
}
