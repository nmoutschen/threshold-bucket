//! # Threshold-based permits
//!
//! Grant permits if the total number of available tokens is greater than a specified threshold.

use super::{InnerPermit, Permitter};
use crate::{inner::Inner, Permit};
use std::sync::Arc;

pub(crate) struct ThresholdPermitter {
    config: ThresholdConfig,
    inner: Arc<Inner>,
}

impl Permitter for ThresholdPermitter {
    fn get_permit(&self) -> Option<Permit> {
        let available = self.inner.available();
        if available >= self.config.threshold {
            Some(Permit::new(ThresholdPermit, Arc::downgrade(&self.inner)))
        } else {
            None
        }
    }

    fn belongs(&self, permit: &Permit) -> bool {
        permit
            .addr()
            .map(|inner| std::ptr::eq(Arc::as_ptr(&self.inner).cast(), inner))
            .unwrap_or(false)
    }
}

pub(crate) struct ThresholdPermit;

impl InnerPermit for ThresholdPermit {
    fn notify(&self, _num: u64) {}
}

/// Threshold configuration
#[derive(Clone, Debug)]
pub struct ThresholdConfig {
    /// Threshold under which the bucket will refuse to grant new permits.
    pub threshold: u64,
}

impl ThresholdConfig {
    pub(crate) fn into_permitter(self, inner: Arc<Inner>) -> ThresholdPermitter {
        ThresholdPermitter {
            config: self,
            inner,
        }
    }
}
