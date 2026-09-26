use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Metrics {
    pub captured: AtomicU64,
    pub filtered: AtomicU64,
    pub published: AtomicU64,
    pub injected: AtomicU64,
    pub rejected_deliveries: AtomicU64,
    pub retries: AtomicU64,
}

impl Metrics {
    pub fn pending_publish_count(&self) -> u64 {
        self.captured.load(Ordering::Relaxed).saturating_sub(self.published.load(Ordering::Relaxed))
    }
}
