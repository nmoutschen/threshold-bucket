#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::{sync::Arc, time::Duration};

mod builder;
mod inner;
pub mod permit;
pub mod refill;

pub use builder::Builder;
use permit::{Permit, Permitter};

/// # Leaky bucket with permitter
#[derive(Clone)]
pub struct Bucket {
    permitter: Arc<dyn Permitter>,
    inner: Arc<inner::Inner>,
}

impl Bucket {
    /// Create a new [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Number of tokens available in the [`Bucket`].
    pub fn available(&self) -> u64 {
        self.inner.available()
    }

    /// Try to acquire a [`Permit`].
    pub fn get_permit(&self) -> Option<Permit> {
        self.permitter.get_permit()
    }

    /// Try to acquire one token.
    ///
    /// Shorthand for `try_acquire(permit, 1)`.
    pub fn try_acquire_one(&self, permit: Permit) -> Result<(), Error> {
        if !self.permitter.belongs(&permit) {
            return Err(Error::InvalidPermit);
        }
        permit.notify(1);
        self.inner.try_acquire(1).map(|_| ())
    }

    /// Try to acquire `num` number of tokens.
    ///
    /// This will return an [`Error`] if the permit is invalid, this tries to acquire more token
    /// than available, or it fails to swap the available number of tokens.
    pub fn try_acquire(&self, permit: Permit, num: u64) -> Result<u64, Error> {
        if !self.permitter.belongs(&permit) {
            return Err(Error::InvalidPermit);
        }
        permit.notify(num);
        self.inner.try_acquire(num)
    }
}

/// Bucket build errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuildError {
    /// Missing refill for the bucket [`Builder`]
    #[error("missing refill")]
    MissingRefill,

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
    NotEnoughTokens(Option<Duration>),
}
