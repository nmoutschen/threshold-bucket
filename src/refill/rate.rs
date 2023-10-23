//! Constant refill rate

use super::Refill;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// Constant refill rate
pub(crate) struct RateRefill {
    quantity: u64,
    interval: Duration,
    max: u64,

    refill_at: AtomicU64,
}

impl RateRefill {
    pub(crate) fn new(config: RateConfig) -> Self {
        Self {
            quantity: config.quantity,
            interval: config.interval,
            max: config.max,
            refill_at: AtomicU64::new(config.interval.as_millis() as u64),
        }
    }
}

impl Refill for RateRefill {
    fn refill(&self, elapsed: Duration, tokens: &AtomicU64) {
        let mut intervals;

        loop {
            let refill_at = Duration::from_millis(self.refill_at.load(Ordering::Relaxed));

            // Next refill is not due yet, return early
            if elapsed < refill_at {
                return;
            }

            // Number of intervals
            // 1 for the time until `refill_at`, then 1 for every time intervals since then
            intervals = (1 + (elapsed - refill_at).as_nanos() / self.interval.as_nanos()) as u64;

            // Update the `refill_at` time
            let next_refill_at = refill_at + (self.interval * intervals as u32);
            if self
                .refill_at
                .compare_exchange(
                    refill_at.as_millis() as u64,
                    next_refill_at.as_millis() as u64,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                break;
            }

            let amount = intervals * self.quantity;
            let available = tokens.load(Ordering::Acquire);

            if available + amount >= self.max {
                tokens.fetch_add(self.max - available, Ordering::Release);
            } else {
                tokens.fetch_add(amount, Ordering::Release);
            }
        }
    }

    /// Calculate the duration until the requested number of tokens will be avilable
    fn wait_for(&self, available: u64, requested: u64) -> Option<Duration> {
        Some(if requested < available {
            Duration::ZERO
        } else {
            let intervals = (requested - available) / self.quantity;
            Duration::from_millis(self.refill_at.load(Ordering::Acquire))
                + (self.interval * intervals as u32)
        })
    }
}

/// Rate refill configuration
#[derive(Clone, Debug)]
pub struct RateConfig {
    /// Tokens to add per interval
    pub quantity: u64,
    /// Interval of time per refill
    pub interval: Duration,
    /// Maximum number of tokens in the bucket
    pub max: u64,
}
