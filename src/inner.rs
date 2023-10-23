use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use crate::{refill::Refill, Error};

pub(crate) struct Inner {
    refill: Box<dyn Refill + Send + Sync>,

    /// Currently available number of tokens
    available: AtomicU64,
    start: Instant,
}

impl Inner {
    /// Create a new inner bucket
    pub(crate) fn new<R>(refill: R, initial: u64) -> Self
    where
        R: Refill + Send + Sync + 'static,
    {
        Self {
            refill: Box::new(refill),
            available: AtomicU64::new(initial),
            start: Instant::now(),
        }
    }

    pub fn available(&self) -> u64 {
        self.available.load(Ordering::Acquire)
    }

    pub fn try_acquire(&self, num: u64) -> Result<u64, Error> {
        self.refill(self.start.elapsed());

        // Compare-and-swap loop
        //
        // If there aren't enough tokens available, this will break early.
        // If there are enough tokens, but the number of available tokens is updated before this
        // call can, it will loop until it can, or it tried 65536 times.
        for _ in 0..0x10000 {
            let available = self.available.load(Ordering::Acquire);
            if available < num {
                return Err(Error::NotEnoughTokens(self.refill.wait_for(available, num)));
            }

            let new = available.saturating_sub(num);

            if self
                .available
                .compare_exchange(available, new, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(num);
            }
        }

        // Could not update the number of available tokens after 65536 attempts. Contention is too,
        // return an error instead of trying ad infinitum.
        Err(Error::HighContention)
    }

    /// Refill tokens if necessary
    fn refill(&self, elapsed: Duration) {
        self.refill.refill(elapsed, &self.available)
    }
}
