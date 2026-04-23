use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(NonZeroU64);

impl WindowId {
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(NonZeroU64::new(COUNTER.fetch_add(1, Ordering::Relaxed)).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WidgetId(NonZeroU64);

impl WidgetId {
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(NonZeroU64::new(COUNTER.fetch_add(1, Ordering::Relaxed)).unwrap())
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}
