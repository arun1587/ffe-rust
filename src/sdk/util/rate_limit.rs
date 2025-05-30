use std::num::NonZeroU32;
use std::sync::Arc;
use governor::{Quota, RateLimiter};
use governor::state::{NotKeyed, InMemoryState};
use governor::clock::DefaultClock;

pub type Limiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

pub fn ors_limiter() -> Limiter {
    let quota = Quota::per_minute(NonZeroU32::new(40).unwrap());
    Arc::new(RateLimiter::direct(quota))
}
