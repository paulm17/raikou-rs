use std::any::Any;
use std::sync::Arc;

use raikou_core::{PaintLayer, Point, Rect, Size, Thickness, WidgetId};
use raikou_text::FontSystem;

use crate::alignment::{HorizontalAlignment, VerticalAlignment};
use crate::attached::AttachedLayout;
use crate::constraints::LayoutConstraints;
use crate::rounding::{round_rect, round_size, round_value};
use crate::text_measure_cache::TextMeasureCache;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Collapsed,
}

pub struct LayoutContext<'a> {
    pub font_system: &'a mut FontSystem,
    pub text_measure_cache: TextMeasureCache,
}

impl<'a> LayoutContext<'a> {
    pub fn new(font_system: &'a mut FontSystem) -> Self {
        Self {
            font_system,
            text_measure_cache: TextMeasureCache::new(),
        }
    }
}

#[derive(Clone)]
pub struct Layoutable {
    id: WidgetId,
    pub(crate) desired_size: Size,
    pub(crate) bounds: Rect,
    pub(crate) layout_slot: Rect,
    pub(crate) previous_measure: Option<Size>,
    pub(crate) previous_arrange: Option<Rect>,
    pub(crate) measure_valid: bool,
    pub(crate) arrange_valid: bool,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub constraints: LayoutConstraints,
    pub margin: Thickness,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,
    pub use_layout_rounding: bool,
    pub visibility: Visibility,
    pub     attached: AttachedLayout,
    overlay_layer: Option<PaintLayer>,
    parent_id: Option<WidgetId>,
    invalidation_cb: Option<Arc<dyn Fn(WidgetId, bool)>>,
}

impl std::fmt::Debug for Layoutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layoutable")
            .field("id", &self.id)
            .field("desired_size", &self.desired_size)
            .field("bounds", &self.bounds)
            .field("layout_slot", &self.layout_slot)
            .field("previous_measure", &self.previous_measure)
            .field("previous_arrange", &self.previous_arrange)
            .field("measure_valid", &self.measure_valid)
            .field("arrange_valid", &self.arrange_valid)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("constraints", &self.constraints)
            .field("margin", &self.margin)
            .field("horizontal_alignment", &self.horizontal_alignment)
            .field("vertical_alignment", &self.vertical_alignment)
            .field("use_layout_rounding", &self.use_layout_rounding)
            .field("visibility", &self.visibility)
            .field("attached", &self.attached)
            .field("overlay_layer", &self.overlay_layer)
            .field("parent_id", &self.parent_id)
            .finish_non_exhaustive()
    }
}

impl Default for Layoutable {
    fn default() -> Self {
        Self {
            id: WidgetId::next(),
            desired_size: Size::ZERO,
            bounds: Rect::default(),
            layout_slot: Rect::default(),
            previous_measure: None,
            previous_arrange: None,
            measure_valid: false,
            arrange_valid: false,
            width: None,
            height: None,
            constraints: LayoutConstraints::default(),
            margin: Thickness::ZERO,
            horizontal_alignment: HorizontalAlignment::Stretch,
            vertical_alignment: VerticalAlignment::Stretch,
            use_layout_rounding: true,
            visibility: Visibility::Visible,
            attached: AttachedLayout::default(),
            overlay_layer: None,
            parent_id: None,
            invalidation_cb: None,
        }
    }
}

impl Layoutable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }

    pub fn desired_size(&self) -> Size {
        self.desired_size
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn layout_slot(&self) -> Rect {
        self.layout_slot
    }

    pub fn is_measure_valid(&self) -> bool {
        self.measure_valid
    }

    pub fn is_arrange_valid(&self) -> bool {
        self.arrange_valid
    }

    pub fn parent_id(&self) -> Option<WidgetId> {
        self.parent_id
    }

    pub fn set_parent_id(&mut self, parent_id: Option<WidgetId>) {
        self.parent_id = parent_id;
    }

    pub fn set_invalidation_callback(&mut self, cb: Option<Arc<dyn Fn(WidgetId, bool)>>) {
        self.invalidation_cb = cb;
    }

    pub fn clear_invalidation_callback(&mut self) {
        self.invalidation_cb = None;
    }

    pub fn paint_layer(&self) -> PaintLayer {
        self.overlay_layer.unwrap_or(PaintLayer::Content)
    }

    pub fn set_overlay_layer(&mut self, layer: PaintLayer) {
        self.overlay_layer = Some(layer);
    }

    pub fn invalidate_measure(&mut self) {
        self.measure_valid = false;
        self.arrange_valid = false;
        if let Some(ref cb) = self.invalidation_cb {
            cb(self.id, true);
        }
    }

    pub fn invalidate_arrange(&mut self) {
        self.arrange_valid = false;
        if let Some(ref cb) = self.invalidation_cb {
            cb(self.id, false);
        }
    }
}

pub trait LayoutElement {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn layout(&self) -> &Layoutable;
    fn layout_mut(&mut self) -> &mut Layoutable;
    fn measure_override(&mut self, ctx: &mut LayoutContext, available: Size) -> Size;
    fn arrange_override(&mut self, ctx: &mut LayoutContext, final_size: Size) -> Size;

    fn visit_children(&self, visitor: &mut dyn FnMut(&dyn LayoutElement));
    fn visit_children_mut(&mut self, visitor: &mut dyn FnMut(&mut dyn LayoutElement));

    fn paint_layer(&self) -> PaintLayer {
        self.layout().paint_layer()
    }
}

#[derive(Clone, Debug)]
pub struct SizedBox {
    layout: Layoutable,
    intrinsic_size: Size,
}

impl SizedBox {
    pub fn new(intrinsic_size: Size) -> Self {
        Self {
            layout: Layoutable::new(),
            intrinsic_size,
        }
    }

    pub fn intrinsic_size(&self) -> Size {
        self.intrinsic_size
    }

    pub fn set_intrinsic_size(&mut self, size: Size) {
        self.intrinsic_size = size;
        self.layout.invalidate_measure();
    }
}

impl LayoutElement for SizedBox {
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
        Size::new(
            self.intrinsic_size.width.min(available.width),
            self.intrinsic_size.height.min(available.height),
        )
    }

    fn arrange_override(&mut self, _ctx: &mut LayoutContext, final_size: Size) -> Size {
        final_size
    }

    fn visit_children(&self, _visitor: &mut dyn FnMut(&dyn LayoutElement)) {}

    fn visit_children_mut(&mut self, _visitor: &mut dyn FnMut(&mut dyn LayoutElement)) {}
}

pub fn measure_element(element: &mut dyn LayoutElement, ctx: &mut LayoutContext, available: Size) -> Size {
    let available = sanitize_size(available);
    if element.layout().measure_valid
        && element.layout().previous_measure == Some(available)
        && !subtree_needs_measure(element)
    {
        return element.layout().desired_size;
    }

    if element.layout().visibility == Visibility::Collapsed {
        let layout = element.layout_mut();
        layout.desired_size = Size::ZERO;
        layout.previous_measure = Some(available);
        layout.measure_valid = true;
        layout.arrange_valid = false;
        return Size::ZERO;
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
    desired
}

pub fn arrange_element(element: &mut dyn LayoutElement, ctx: &mut LayoutContext, final_rect: Rect) {
    let final_rect = sanitize_rect(final_rect);
    if element.layout().arrange_valid
        && element.layout().previous_arrange == Some(final_rect)
        && !subtree_needs_arrange(element)
    {
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
}

pub(crate) fn sanitize_size(size: Size) -> Size {
    Size::new(
        sanitize_dimension(size.width),
        sanitize_dimension(size.height),
    )
}

pub(crate) fn subtree_needs_measure(element: &dyn LayoutElement) -> bool {
    let mut needed = false;
    element.visit_children(&mut |child| {
        if !child.layout().is_measure_valid() || subtree_needs_measure(child) {
            needed = true;
        }
    });
    needed
}

pub(crate) fn subtree_needs_arrange(element: &dyn LayoutElement) -> bool {
    let mut needed = false;
    element.visit_children(&mut |child| {
        if !child.layout().is_arrange_valid() || subtree_needs_arrange(child) {
            needed = true;
        }
    });
    needed
}

pub(crate) fn sanitize_rect(rect: Rect) -> Rect {
    Rect::new(
        Point::new(
            if rect.origin.x.is_finite() {
                rect.origin.x
            } else {
                0.0
            },
            if rect.origin.y.is_finite() {
                rect.origin.y
            } else {
                0.0
            },
        ),
        sanitize_size(rect.size),
    )
}

pub(crate) fn round_layout_slot(rect: Rect) -> Rect {
    let left = round_value(rect.origin.x);
    let top = round_value(rect.origin.y);
    let right = round_value(rect.right());
    let bottom = round_value(rect.bottom());
    Rect::from_xywh(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
}

fn sanitize_dimension(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value.is_sign_negative() && value.is_finite() {
        0.0
    } else {
        value
    }
}

pub(crate) fn arrange_axis(
    slot: f32,
    desired: f32,
    explicit: Option<f32>,
    min: f32,
    max: f32,
    stretch: bool,
) -> f32 {
    let basis = if stretch && explicit.is_none() {
        slot
    } else {
        desired.min(slot)
    };
    basis.clamp(min, max.min(slot))
}

pub(crate) fn align_axis<T>(origin: f32, slot: f32, size: f32, alignment: T) -> f32
where
    T: Into<AxisAlignment>,
{
    match alignment.into() {
        AxisAlignment::Start | AxisAlignment::Stretch => origin,
        AxisAlignment::Center => origin + ((slot - size).max(0.0) / 2.0),
        AxisAlignment::End => origin + (slot - size).max(0.0),
    }
}

pub(crate) enum AxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

impl From<HorizontalAlignment> for AxisAlignment {
    fn from(value: HorizontalAlignment) -> Self {
        match value {
            HorizontalAlignment::Stretch => Self::Stretch,
            HorizontalAlignment::Left => Self::Start,
            HorizontalAlignment::Center => Self::Center,
            HorizontalAlignment::Right => Self::End,
        }
    }
}

impl From<VerticalAlignment> for AxisAlignment {
    fn from(value: VerticalAlignment) -> Self {
        match value {
            VerticalAlignment::Stretch => Self::Stretch,
            VerticalAlignment::Top => Self::Start,
            VerticalAlignment::Center => Self::Center,
            VerticalAlignment::Bottom => Self::End,
        }
    }
}
