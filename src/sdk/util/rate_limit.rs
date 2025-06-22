use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// A synchronous wrapper around the governor RateLimiter.
pub struct Limiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl Limiter {
    /// Creates a new limiter configured for ORS (40 calls/minute).
    pub fn new() -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(40).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    /// Blocks the current thread until a cell is available.
    pub fn wait(&self) {
        while self.limiter.check().is_err() {
            // Sleep briefly to avoid a spin-loop.
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}
