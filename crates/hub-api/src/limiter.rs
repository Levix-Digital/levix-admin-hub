//! Request limit per source address on the unauthenticated endpoints (sign-in and links;
//! threat model T-03). A fixed one-minute window per address, kept in memory.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::ApiError;

/// Unauthenticated requests one source address may make per minute.
pub const UNAUTHENTICATED_PER_MINUTE: u32 = 20;

const WINDOW: Duration = Duration::from_secs(60);
const MAX_TRACKED: usize = 10_000;

#[derive(Debug)]
pub struct RateLimiter {
    per_window: u32,
    windows: Mutex<HashMap<IpAddr, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(per_window: u32) -> Self {
        Self {
            per_window,
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Counts one request from `address`; refuses it once the window is full. Requests whose
    /// address is unknown (tests, local sockets) share one bucket.
    pub fn check(&self, address: Option<IpAddr>) -> Result<(), ApiError> {
        self.check_at(address, Instant::now())
    }

    fn check_at(&self, address: Option<IpAddr>, now: Instant) -> Result<(), ApiError> {
        let key = address.unwrap_or(IpAddr::from([0, 0, 0, 0]));
        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        if windows.len() >= MAX_TRACKED {
            windows.retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
        }
        let entry = windows.entry(key).or_insert((now, 0));
        if now.duration_since(entry.0) >= WINDOW {
            *entry = (now, 0);
        }
        if entry.1 >= self.per_window {
            let remaining = WINDOW.saturating_sub(now.duration_since(entry.0));
            return Err(ApiError::throttled(remaining.as_secs().max(1)));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_each_address_for_one_window() {
        let limiter = RateLimiter::new(3);
        let start = Instant::now();
        let a = Some(IpAddr::from([192, 0, 2, 1]));
        let b = Some(IpAddr::from([192, 0, 2, 2]));
        for _ in 0..3 {
            assert!(limiter.check_at(a, start).is_ok());
        }
        assert!(limiter.check_at(a, start).is_err());
        assert!(limiter.check_at(b, start).is_ok());
        assert!(limiter.check_at(a, start + WINDOW).is_ok());
    }
}
