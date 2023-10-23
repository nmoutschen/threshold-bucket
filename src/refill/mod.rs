//! Refill algoritms

use self::rate::RateRefill;
use crate::inner::Inner;
use std::{sync::atomic::AtomicU64, time::Duration};

pub(crate) mod rate;
pub use rate::RateConfig;

pub(crate) trait Refill {
    fn refill(&self, elapsed: Duration, tokens: &AtomicU64);
    fn wait_for(&self, available: u64, requested: u64) -> Option<Duration>;
}

pub(crate) enum RefillConfig {
    Rate(RateConfig),
}

impl RefillConfig {
    pub(crate) fn into_inner_bucket(self, initial: u64) -> Inner {
        match self {
            RefillConfig::Rate(rate_config) => Inner::new(RateRefill::new(rate_config), initial),
        }
    }
}
