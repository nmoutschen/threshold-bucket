#![warn(missing_docs)]
//! # Leaky bucket with threshold
//!
//! This crate implements a leaky bucket algorithm with threshold.
//!
//! A user must present a valid [`Permit`] to acquire tokens in this bucket. The bucket will grant
//! a [`Permit`] if the number of tokens is higher than the specified threshold.
//!
//! ```rust
//! # use std::time::Duration;
//! use threshold_bucket::Bucket;
//!
//! # || {
//!
//! // Creating a new Bucket with a threshold of 100, max of 200, and a refill of 10 tokens every
//! // second.
//! // This bucket starts with 110 tokens.
//! let bucket = Bucket::builder()
//!     .refill_rate(10, Duration::from_secs(1))
//!     .threshold(100)
//!     .max(200)
//!     .build()?;
//!
//! // We can acquire a permit since there are more than 100 tokens.
//! let permit = bucket.try_permit()?;
//!
//! // We now remove 50 tokens, leaving 60 tokens behind.
//! let tokens = bucket.try_acquire(permit, 50);
//!
//! // This will fail, as there are only 60 tokens left in the bucket, less than the threshold
//! // value.
//! let permit = bucket.try_permit()?;
//!
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! # };
//! ```

use std::{sync::Arc, time::Duration};

mod builder;
pub use builder::Builder;
mod inner;
mod permit;
pub use permit::Permit;

/// # Leaky bucket with threshold
#[derive(Clone)]
pub struct Bucket(Arc<inner::Inner>);

impl Bucket {
    /// Create a new [`Bucket`] with `max` capacity and that refills `refill` tokens every `interval`.
    ///
    /// This starts with `refill` + `threshold` tokens in the bucket.
    pub(crate) fn new(refill: u64, interval: Duration, threshold: u64, max: u64) -> Self {
        Self(Arc::new(inner::Inner::new(
            refill, interval, threshold, max,
        )))
    }

    /// Create a new bucket [`Builder`].
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Number of tokens available in the [`Bucket`].
    pub fn available(&self) -> u64 {
        self.0.available()
    }

    /// Threshold before this [`Bucket`] will reject permit grant requests.
    pub fn threshold(&self) -> u64 {
        self.0.threshold()
    }

    /// Maximum number of tokens for this bucket
    pub fn max(&self) -> u64 {
        self.0.max()
    }

    /// Try to acquire a [`Permit`].
    ///
    /// This will return an [`Error`] when there are no tokens available in the bucket.
    pub fn try_permit(&self) -> Result<Permit, Error> {
        self.0.try_permit()
    }

    /// Try to acquire one token.
    ///
    /// Shorthand for `try_acquire(permit, 1)`.
    pub fn try_acquire_one(&self, permit: Permit) -> Result<(), Error> {
        self.0.try_acquire_one(permit)
    }

    /// Try to acquire `num` number of tokens.
    ///
    /// This will return an [`Error`] if the permit is invalid, this tries to acquire more token
    /// than available, or it fails to swap the available number of tokens.
    pub fn try_acquire(&self, permit: Permit, num: u64) -> Result<u64, Error> {
        self.0.try_acquire(permit, num)
    }
}

/// Bucket build errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    /// Missing refill_rate for the bucket [`Builder`]
    #[error("missing refill rate")]
    MissingRefillRate,

    /// Missing max number of tokens for the bucket [`Builder`]
    #[error("missing max")]
    MissingMax,
}

/// Bucket errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// The requested number of tokens exceeds the maximum possible number of tokens
    #[error("requested number exceeds the maximum number of tokens")]
    ExceedMaxTokens,

    /// High contention to update the number of available tokens
    #[error("high contention to update available token")]
    HighContention,

    /// The permit passed does not correspond to this bucket
    #[error("invalid permit")]
    InvalidPermit,

    /// Not enough tokens available
    #[error("not enough tokens available")]
    NotEnoughTokens(Duration),
}
