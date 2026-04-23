use skia_safe::Canvas;

use crate::error::FrameError;
use crate::painter::Painter;
use crate::surface::{ActiveFrame, SkiaSurfaceManager};

pub struct Frame<'a> {
    active: Option<ActiveFrame<'a>>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(active: ActiveFrame<'a>) -> Self {
        Self {
            active: Some(active),
        }
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.active
            .as_mut()
            .expect("frame must be live while borrowed")
            .surface
            .canvas()
    }

    pub fn painter(&mut self) -> Painter<'_> {
        Painter::new(self.canvas())
    }

    pub fn present(mut self) -> Result<(), FrameError> {
        self.active
            .take()
            .expect("frame must be live while presented")
            .present()
    }
}

impl SkiaSurfaceManager {
    pub fn begin_frame(&mut self) -> Result<Frame<'_>, FrameError> {
        self.acquire_frame().map(Frame::new)
    }
}
