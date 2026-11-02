use std::sync::Mutex;
use std::time::Duration;

use tokio::time::Instant;

/// A token bucket in its GCRA form: requests are spaced `1 / rate` apart once
/// `burst` back-to-back requests are used up. Each caller reserves its slot
/// under the lock, then sleeps outside it, so waiters never overtake.
pub(super) struct TokenBucket {
    interval: Duration,
    burst: u32,
    /// Theoretical arrival time of the next request.
    tat: Mutex<Option<Instant>>,
}

impl TokenBucket {
    pub(super) fn new(rate_per_sec: u32, burst: u32) -> Self {
        Self {
            interval: Duration::from_secs(1) / rate_per_sec.max(1),
            burst: burst.max(1),
            tat: Mutex::new(None),
        }
    }

    pub(super) async fn acquire(&self) {
        let wait = {
            let mut tat = self.tat.lock().expect("limiter lock");
            let now = Instant::now();
            let next = tat.unwrap_or(now).max(now);
            let slack = self.interval * (self.burst - 1);
            let allowed_at = next.checked_sub(slack).unwrap_or(now).max(now);
            *tat = Some(next + self.interval);
            allowed_at - now
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }
}
