use governor::{Quota, RateLimiter, state::InMemoryState, clock::DefaultClock};
use once_cell::sync::Lazy;
use std::num::NonZeroU32;
use std::sync::Arc;
use governor::middleware::NoOpMiddleware;

// Type alias for clarity
pub type Limiter = Arc<RateLimiter<governor::state::NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

pub static GEOCODE_LIMITER: Lazy<Limiter> = Lazy::new(|| {
    Arc::new(RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(100).unwrap()),
    ))
});

pub static DIRECTIONS_LIMITER: Lazy<Limiter> = Lazy::new(|| {
    Arc::new(RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(40).unwrap()),
    ))
});
