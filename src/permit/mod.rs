//! # [`Permit`] types
//!
//! This module contains the different types of [`Permit`]s supported by this crate:
//!
//! * [`threshold::ThresholdPermit`]

use crate::inner::Inner;
use std::sync::{Arc, Weak};

pub(crate) mod always;
pub(crate) mod threshold;
pub use threshold::ThresholdConfig;

/// Trait that grants a [`Permit`] if conditions are met.
pub(crate) trait Permitter {
    /// Get a new [`Permit`] for this [`Permitter`]
    ///
    /// This should return [`None`] if it cannot allocate a permit at the moment.
    fn get_permit(&self) -> Option<Permit>;

    /// Check if the [`Permit`] belongs to this [`Permitter`]
    fn belongs(&self, permit: &Permit) -> bool;
}

pub(crate) trait InnerPermit {
    fn notify(&self, num: u64);
}

/// Permit used to get tokens from a bucket
pub struct Permit {
    inner: Box<dyn InnerPermit>,
    bucket: Weak<Inner>,
}

impl Permit {
    pub(crate) fn new<I>(inner: I, bucket: Weak<Inner>) -> Self
    where
        I: InnerPermit + 'static,
    {
        Self {
            inner: Box::new(inner),
            bucket,
        }
    }
    pub(crate) fn addr(&self) -> Option<*const Inner> {
        Weak::upgrade(&self.bucket).map(|b| Arc::as_ptr(&b))
    }

    pub(crate) fn notify(self, num: u64) {
        self.inner.notify(num);
    }
}

/// Configuration for permits
pub(crate) enum PermitConfig {
    /// Threshold-based permit allocation
    Threshold(ThresholdConfig),
}
