pub trait ClipboardBackend: Send {
    fn get_text(&mut self) -> Option<String> {
        None
    }

    fn set_text(&mut self, _text: String) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct NoopClipboard;

impl ClipboardBackend for NoopClipboard {}
