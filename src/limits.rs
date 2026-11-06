//! Local request-weight throttling.
//!
//! `exchangeInfo.rateLimits` publishes the budget (mainnet futures: 2400 weight
//! per minute). Attaching a [`RateLimiter`](crate::limits::RateLimiter) makes every request wait for its
//! documented weight *before* it is sent, so the connection rarely reaches a
//! server-side `429`. The `ORDERS` budget counts order submissions, not weight;
//! that one stays the caller's job.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::model::{RateLimit, RateLimitType};

/// Token bucket over documented request weights.
#[derive(Debug)]
pub struct RateLimiter {
    capacity: u32,
    /// Tokens per second.
    refill: f64,
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    /// `limit` weight units per `window`.
    pub fn new(limit: u32, window: Duration) -> Self {
        assert!(limit > 0, "limit must be positive");
        assert!(!window.is_zero(), "window must be non-zero");
        Self {
            capacity: limit,
            refill: f64::from(limit) / window.as_secs_f64(),
            state: Mutex::new(State {
                tokens: f64::from(limit),
                last: Instant::now(),
            }),
        }
    }

    /// `limit` weight units per minute.
    pub fn per_minute(limit: u32) -> Self {
        Self::new(limit, Duration::from_secs(60))
    }

    /// Limiter sized from `exchangeInfo.rateLimits` (`REQUEST_WEIGHT`, falling
    /// back to the first entry, then to 2400/min).
    pub fn from_rate_limits(limits: &[RateLimit]) -> Self {
        let entry = limits
            .iter()
            .find(|l| l.rate_limit_type == RateLimitType::RequestWeight)
            .or_else(|| limits.first());
        let Some(entry) = entry else {
            return Self::per_minute(2400);
        };
        let unit = match entry.interval.as_str() {
            "SECOND" => Duration::from_secs(1),
            "HOUR" => Duration::from_secs(3_600),
            "DAY" => Duration::from_secs(86_400),
            _ => Duration::from_secs(60),
        };
        let window = unit.checked_mul(entry.interval_num.max(1)).unwrap_or(unit);
        Self::new(entry.limit.max(1), window)
    }

    /// Take `weight` units if they are available. `None` means sent now;
    /// `Some(wait)` means the bucket needs `wait` to refill.
    ///
    /// A request heavier than the whole budget is clamped to the budget, so it
    /// can always pass instead of stalling forever.
    pub fn try_acquire(&self, weight: u32) -> Option<Duration> {
        let want = f64::from(weight.clamp(1, self.capacity));
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        state.tokens = (state.tokens + now.duration_since(state.last).as_secs_f64() * self.refill)
            .min(f64::from(self.capacity));
        state.last = now;
        if state.tokens >= want {
            state.tokens -= want;
            None
        } else {
            Some(Duration::from_secs_f64((want - state.tokens) / self.refill))
        }
    }

    /// Wait until `weight` units are available.
    pub async fn acquire(&self, weight: u32) {
        loop {
            match self.try_acquire(weight) {
                None => return,
                Some(wait) => tokio::time::sleep(wait).await,
            }
        }
    }
}

impl From<&[RateLimit]> for RateLimiter {
    fn from(limits: &[RateLimit]) -> Self {
        Self::from_rate_limits(limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_blocks_then_refills() {
        let limiter = RateLimiter::new(1_000, Duration::from_millis(200));
        assert!(limiter.try_acquire(600).is_none());
        assert!(limiter.try_acquire(600).is_some(), "budget is 1000");
        let waited = limiter.try_acquire(600).unwrap();
        assert!(waited <= Duration::from_millis(200), "{waited:?}");
        std::thread::sleep(waited + Duration::from_millis(20));
        assert!(limiter.try_acquire(600).is_none());
    }

    #[test]
    fn heavier_than_budget_still_passes() {
        let limiter = RateLimiter::per_minute(10);
        assert!(limiter.try_acquire(1_000).is_none());
    }

    #[test]
    fn sizes_from_exchange_info() {
        let limits = vec![
            RateLimit {
                rate_limit_type: RateLimitType::Orders,
                interval: "MINUTE".into(),
                interval_num: 1,
                limit: 1200,
            },
            RateLimit {
                rate_limit_type: RateLimitType::RequestWeight,
                interval: "MINUTE".into(),
                interval_num: 1,
                limit: 2400,
            },
        ];
        let limiter = RateLimiter::from_rate_limits(&limits);
        assert_eq!(limiter.capacity, 2400);
        assert_eq!(limiter.refill, 40.0);
    }
}
