use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("failed to create event loop: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error("failed to create window: {0}")]
    Os(#[from] winit::error::OsError),
}
