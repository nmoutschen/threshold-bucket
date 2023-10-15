use std::time::Duration;

use crate::{Bucket, BuildError};

/// Builder for [`Bucket`]
#[derive(Default)]
#[must_use]
pub struct Builder {
    max: Option<u64>,
    threshold: Option<u64>,
    refill_rate: Option<(u64, Duration)>,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of tokens
    pub fn max(self, max: u64) -> Self {
        Self {
            max: Some(max),
            ..self
        }
    }

    /// Set the threshold under which the bucket will not grant new permits
    pub fn threshold(self, threshold: u64) -> Self {
        Self {
            threshold: Some(threshold),
            ..self
        }
    }

    /// Set the refill rate
    pub fn refill_rate(self, qty: u64, interval: Duration) -> Self {
        Self {
            refill_rate: Some((qty, interval)),
            ..self
        }
    }

    /// Build the [`Bucket`]
    pub fn build(self) -> Result<Bucket, BuildError> {
        let (refill, interval) = self.refill_rate.ok_or(BuildError::MissingRefillRate)?;
        Ok(Bucket::new(
            refill,
            interval,
            self.threshold.unwrap_or(0),
            self.max.ok_or(BuildError::MissingMax)?,
        ))
    }
}
