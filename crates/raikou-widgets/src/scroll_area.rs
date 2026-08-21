//! The ScrollArea component.
//!
//! Maps onto fyrox's `ScrollViewer`. Scroll offsets on either axis are reported
//! through a per-component `on_scroll` handler with the current `(vertical,
//! horizontal)` values.

use std::cell::Cell;
use std::rc::Rc;

use fyrox::core::pool::Handle;
use fyrox::gui::message::{MessageDirection, UiMessage};
use fyrox::gui::scroll_viewer::{ScrollViewerBuilder, ScrollViewerMessage};
use fyrox::gui::widget::WidgetBuilder;
use fyrox::gui::{UiNode, UserInterface};

use raikou_core::{Length, Thickness};

use crate::build_cx::BuildCx;
use crate::component::{Component, ComponentKind};
use crate::convert::to_fyrox_thickness;

type ScrollCallback = dyn Fn(&mut UserInterface, f32, f32);
/// Handlers of a ScrollArea component.
#[derive(Clone)]
pub struct ScrollAreaHandlers {
    /// Current vertical scroll offset.
    pub v_offset: Cell<f32>,
    /// Current horizontal scroll offset.
    pub h_offset: Cell<f32>,
    /// Invoked with `(vertical, horizontal)` offsets on scroll.
    pub on_scroll: Option<Rc<ScrollCallback>>,
}

impl ScrollAreaHandlers {
    /// Routes a UI message to the matching handler.
    pub fn dispatch(&self, ui: &mut UserInterface, message: &UiMessage) {
        if message.direction() != MessageDirection::FromWidget {
            return;
        }
        let mut changed = None;
        if let Some(ScrollViewerMessage::VerticalScroll(v)) = message.data::<ScrollViewerMessage>() {
            self.v_offset.set(*v);
            changed = Some((*v, self.h_offset.get()));
        } else if let Some(ScrollViewerMessage::HorizontalScroll(h)) =
            message.data::<ScrollViewerMessage>()
        {
            self.h_offset.set(*h);
            changed = Some((self.v_offset.get(), *h));
        }
        if let (Some((v, h)), Some(callback)) = (changed, &self.on_scroll) {
            callback(ui, v, h);
        }
    }
}

/// Builder for a [`ScrollArea`] component.
#[derive(Clone)]
pub struct ScrollArea {
    content: Option<Handle<UiNode>>,
    width: Length,
    height: Length,
    v_scroll_allowed: bool,
    h_scroll_allowed: bool,
    v_scroll_speed: f32,
    h_scroll_speed: f32,
    on_scroll: Option<Rc<ScrollCallback>>,
    margin: Thickness,
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollArea {
    /// Creates a new scroll area builder.
    pub fn new() -> Self {
        Self {
            content: None,
            width: Length::Auto,
            height: Length::Auto,
            v_scroll_allowed: true,
            h_scroll_allowed: true,
            v_scroll_speed: 20.0,
            h_scroll_speed: 20.0,
            on_scroll: None,
            margin: Thickness::ZERO,
        }
    }

    /// Sets the scrollable content.
    pub fn content(mut self, content: Handle<UiNode>) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets whether vertical scrolling is allowed (default true).
    pub fn vertical_scroll_allowed(mut self, allowed: bool) -> Self {
        self.v_scroll_allowed = allowed;
        self
    }

    /// Sets whether horizontal scrolling is allowed (default true).
    pub fn horizontal_scroll_allowed(mut self, allowed: bool) -> Self {
        self.h_scroll_allowed = allowed;
        self
    }

    /// Sets the vertical wheel scroll speed (default 20).
    pub fn v_scroll_speed(mut self, speed: f32) -> Self {
        self.v_scroll_speed = speed;
        self
    }

    /// Sets the horizontal wheel scroll speed (default 20).
    pub fn h_scroll_speed(mut self, speed: f32) -> Self {
        self.h_scroll_speed = speed;
        self
    }

    /// Sets the callback invoked with `(vertical, horizontal)` offsets on scroll.
    pub fn on_scroll<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut UserInterface, f32, f32) + 'static,
    {
        self.on_scroll = Some(Rc::new(callback));
        self
    }

    /// Sets the outer margin.
    pub fn margin(mut self, margin: Thickness) -> Self {
        self.margin = margin;
        self
    }

    /// Builds the scroll area, adds it to the UI and registers its handlers.
    pub fn build(self, cx: &mut BuildCx) -> Component {
        let widget_builder = WidgetBuilder::new()
            .with_name("raikou_scroll_area")
            .with_margin(to_fyrox_thickness(self.margin));
        let widget_builder = match self.width.resolve() {
            Some(width) => widget_builder.with_width(width),
            None => widget_builder,
        };
        let widget_builder = match self.height.resolve() {
            Some(height) => widget_builder.with_height(height),
            None => widget_builder,
        };

        let mut builder = ScrollViewerBuilder::new(widget_builder)
            .with_vertical_scroll_allowed(self.v_scroll_allowed)
            .with_horizontal_scroll_allowed(self.h_scroll_allowed)
            .with_v_scroll_speed(self.v_scroll_speed)
            .with_h_scroll_speed(self.h_scroll_speed);

        if let Some(content) = self.content {
            builder = builder.with_content(content);
        }

        let handle = {
            let mut ctx = cx.ctx();
            builder.build(&mut ctx).to_base()
        };

        let component = Component {
            handle,
            kind: ComponentKind::ScrollArea(ScrollAreaHandlers {
                v_offset: Cell::new(0.0),
                h_offset: Cell::new(0.0),
                on_scroll: self.on_scroll,
            }),
        };
        cx.register(&component);
        component
    }
}

/// A handle to a built scroll area.
pub type ScrollAreaHandle = Handle<UiNode>;
