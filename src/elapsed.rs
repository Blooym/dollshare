use std::time::SystemTime;
use tracing::debug;

/// Measures processing time (in ms) from creation time to
/// when this struct is dropped.
pub struct Elapsed(&'static str, SystemTime);

impl Elapsed {
    pub fn start(op: &'static str) -> Self {
        Self(op, SystemTime::now())
    }
}

impl Drop for Elapsed {
    fn drop(&mut self) {
        debug!(
            "{} finished after {}us",
            self.0,
            self.1.elapsed().unwrap_or_default().as_micros()
        );
    }
}
