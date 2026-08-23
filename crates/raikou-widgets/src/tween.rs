//! Frame-driven micro-tweens (switch knob slide, accordion chevron spin).
//!
//! fyrox's animation machinery is heavyweight for these small geometry
//! transitions, so this control interpolates directly in [`Control::update`].
//! Callers post a [`TweenMessage`] at the tweener node; the job runs for a
//! fraction of a second and snaps the target widget to its final state.

use fyrox::core::algebra::Vector2;
use fyrox::core::pool::Handle;
use fyrox::core::reflect::prelude::*;
use fyrox::core::visitor::prelude::*;
use fyrox::gui::border::Border;
use fyrox::gui::message::{MessageData, MessageDirection, UiMessage};
use fyrox::gui::vector_image::{Primitive, VectorImage};
use fyrox::gui::widget::{Widget, WidgetBuilder};
use fyrox::gui::{BuildContext, Control, HorizontalAlignment, UiNode, UserInterface};
use fyrox::graph::SceneGraph;

use crate::convert::to_fyrox_thickness;

use raikou_core::Thickness;

/// Duration of every tween in seconds.
const TWEEN_DURATION: f32 = 0.12;

/// One interpolation: slide the switch knob between track ends or spin a
/// chevron mark between its collapsed and expanded angles.
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "8f4b1c62-9d3e-4a57-b2c1-6e0a95d3f7a4")]
pub enum TweenJob {
    /// Interpolate the knob's left margin from its current side to the other.
    KnobSlide {
        knob: Handle<UiNode>,
        /// Track inset on both sides of the knob.
        pad: f32,
        /// Distance between the two rest positions.
        span: f32,
    },
    /// Rotate `base` primitives around their 10x10 mark center (5, 5).
    ChevronSpin {
        image: Handle<UiNode>,
        base: Vec<Primitive>,
        from_deg: f32,
        to_deg: f32,
    },
}

impl Default for TweenJob {
    fn default() -> Self {
        TweenJob::KnobSlide {
            knob: Handle::NONE,
            pad: 0.0,
            span: 0.0,
        }
    }
}

/// Starts (or retargets) the tweener's current job.
#[derive(Debug, Clone, PartialEq)]
pub struct TweenMessage(pub TweenJob);

impl MessageData for TweenMessage {}

/// Control that owns and advances one tween at a time.
#[derive(Clone, Debug, PartialEq, Visit, Reflect)]
#[reflect(type_uuid = "5d90c3f1-2ab7-46de-9b84-c713e6a20f58")]
#[reflect(derived_type = "UiNode")]
pub struct TweenerControl {
    widget: Widget,
    job: Option<TweenJob>,
    elapsed: f32,
    /// KnobSlide only: whether the knob is travelling towards the right end.
    to_right: bool,
}

fyrox::gui::define_widget_deref!(TweenerControl);

fn rotate_prims(prims: &[Primitive], deg: f32) -> Vec<Primitive> {
    // Rotation around the chevron mark center; the marks are built in a
    // 10x10 box so (5, 5) is the pivot.
    let rad = deg.to_radians();
    let (sin, cos) = rad.sin_cos();
    let rot = |p: Vector2<f32>| {
        Vector2::new(
            5.0 + (p.x - 5.0) * cos - (p.y - 5.0) * sin,
            5.0 + (p.x - 5.0) * sin + (p.y - 5.0) * cos,
        )
    };
    prims
        .iter()
        .map(|prim| match prim {
            Primitive::Line {
                begin,
                end,
                thickness,
            } => Primitive::Line {
                begin: rot(*begin),
                end: rot(*end),
                thickness: *thickness,
            },
            other => other.clone(),
        })
        .collect()
}

impl TweenerControl {
    fn start_knob_slide(
        &mut self,
        ui: &mut UserInterface,
        knob: Handle<UiNode>,
        pad: f32,
        span: f32,
    ) {
        // The alignment owns the resting position; while tweening the margin
        // owns it instead, so pin the knob to Left with the margin set to
        // wherever it currently sits.
        let align = ui
            .try_get(knob)
            .map(|node| node.horizontal_alignment())
            .unwrap_or(HorizontalAlignment::Left);
        self.to_right = align == HorizontalAlignment::Left;
        let from = if self.to_right { pad } else { pad + span };
        if let Ok(b) = ui.try_get_mut_of_type::<Border>(knob) {
            b.widget
                .horizontal_alignment
                .set_value_and_mark_modified(HorizontalAlignment::Left);
            b.widget
                .margin
                .set_value_and_mark_modified(to_fyrox_thickness(Thickness::new(
                    from, pad, pad, pad,
                )));
        }
    }
}

impl Control for TweenerControl {
    fn handle_routed_message(&mut self, ui: &mut UserInterface, message: &mut UiMessage) {
        self.widget.handle_routed_message(ui, message);

        if message.direction() == MessageDirection::ToWidget
            && message.destination() == self.handle()
        {
            if let Some(TweenMessage(job)) = message.data::<TweenMessage>() {
                if let TweenJob::KnobSlide { knob, pad, span } = job {
                    self.start_knob_slide(ui, *knob, *pad, *span);
                }
                self.job = Some(job.clone());
                self.elapsed = 0.0;
            }
        }
    }

    fn update(&mut self, dt: f32, ui: &mut UserInterface) {
        let Some(job) = self.job.as_ref() else {
            return;
        };

        self.elapsed += dt;
        let t = (self.elapsed / TWEEN_DURATION).min(1.0);
        // Smoothstep easing matches the Fluent motion feel.
        let eased = t * t * (3.0 - 2.0 * t);

        match job {
            TweenJob::KnobSlide { knob, pad, span } => {
                let from = if self.to_right { *pad } else { *pad + *span };
                let to = if self.to_right { *pad + *span } else { *pad };
                let left = from + (to - from) * eased;
                if let Ok(b) = ui.try_get_mut_of_type::<Border>(*knob) {
                    b.widget
                        .margin
                        .set_value_and_mark_modified(to_fyrox_thickness(Thickness::new(
                            left, *pad, *pad, *pad,
                        )));
                }
                if t >= 1.0 {
                    // Hand position back to the alignment for layout stability.
                    if let Ok(b) = ui.try_get_mut_of_type::<Border>(*knob) {
                        b.widget.horizontal_alignment.set_value_and_mark_modified(
                            if self.to_right {
                                HorizontalAlignment::Right
                            } else {
                                HorizontalAlignment::Left
                            },
                        );
                        b.widget
                            .margin
                            .set_value_and_mark_modified(to_fyrox_thickness(Thickness::uniform(
                                *pad,
                            )));
                    }
                    self.job = None;
                }
            }
            TweenJob::ChevronSpin {
                image,
                base,
                from_deg,
                to_deg,
            } => {
                let angle = from_deg + (to_deg - from_deg) * eased;
                if let Ok(img) = ui.try_get_mut_of_type::<VectorImage>(*image) {
                    img.primitives
                        .set_value_and_mark_modified(rotate_prims(base, angle));
                }
                if t >= 1.0 {
                    if let Ok(img) = ui.try_get_mut_of_type::<VectorImage>(*image) {
                        img.primitives.set_value_and_mark_modified(base.clone());
                    }
                    self.job = None;
                }
            }
        }
    }
}

/// Builds an inert tweener node under `parent` and returns its handle.
pub(crate) fn spawn_tweener(ctx: &mut BuildContext, parent: Handle<UiNode>) -> Handle<UiNode> {
    let control = TweenerControl {
        widget: WidgetBuilder::new()
            .with_name("raikou_tweener")
            .with_need_update(true)
            .with_width(0.0)
            .with_height(0.0)
            .build(ctx),
        job: None,
        elapsed: 0.0,
        to_right: false,
    };
    let handle = ctx.add(control).transmute();
    ctx.link(handle, parent);
    handle
}
