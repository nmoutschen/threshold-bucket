use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use crate::{Error, Permit};

pub(crate) struct Inner {
    refill: u64,
    interval: Duration,
    max: u64,
    threshold: u64,

    /// Currently available number of tokens
    available: AtomicU64,
    start: Instant,
    refill_at: AtomicU64,
}

impl Inner {
    pub fn new(refill: u64, interval: Duration, threshold: u64, max: u64) -> Self {
        Self {
            refill,
            interval,
            max,
            threshold,
            available: AtomicU64::new(threshold + refill),
            start: Instant::now(),
            refill_at: AtomicU64::new(interval.as_millis() as u64),
        }
    }

    pub fn available(&self) -> u64 {
        self.available.load(Ordering::Acquire)
    }

    pub fn threshold(&self) -> u64 {
        self.threshold
    }

    pub fn max(&self) -> u64 {
        self.max
    }

    pub fn try_permit(self: &Arc<Self>) -> Result<Permit, Error> {
        if self.available() >= self.threshold {
            Ok(Permit::new(Arc::downgrade(self)))
        } else {
            Err(Error::ExceedMaxTokens)
        }
    }

    /// Check if a [`Permit`] is valid for this bucket
    fn check_permit(&self, permit: Permit) -> Result<(), Error> {
        permit
            .0
            .upgrade()
            .and_then(|b| {
                if !std::ptr::eq(Arc::as_ptr(&b), self) {
                    Some(())
                } else {
                    None
                }
            })
            .ok_or(Error::InvalidPermit)
    }

    pub fn try_acquire_one(&self, permit: Permit) -> Result<(), Error> {
        self.try_acquire(permit, 1).map(|_| ())
    }

    pub fn try_acquire(&self, permit: Permit, num: u64) -> Result<u64, Error> {
        self.check_permit(permit)?;
        let refill_res = self.refill(self.start.elapsed());

        // Compare-and-swap loop
        //
        // If there aren't enough tokens available, this will break early.
        // If there are enough tokens, but the number of available tokens is updated before this
        // call can, it will loop until it can, or it tried 65536 times.
        for _ in 0..0x10000 {
            let available = self.available.load(Ordering::Acquire);
            if available < num {
                return Err(Error::NotEnoughTokens(match refill_res {
                    Ok(_) => self.start.elapsed(),
                    Err(err) => err,
                }));
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

    fn refill(&self, elapsed: Duration) -> Result<(), Duration> {
        let mut intervals;

        loop {
            let refill_at = Duration::from_millis(self.refill_at.load(Ordering::Relaxed));

            // Next refill is not due yet, retry later
            if elapsed < refill_at {
                return Err(refill_at - elapsed);
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
        }

        let amount = intervals * self.refill;
        let available = self.available.load(Ordering::Acquire);

        if available + amount >= self.max {
            self.available
                .fetch_add(self.max - available, Ordering::Release);
        } else {
            self.available.fetch_add(amount, Ordering::Release);
        }

        Ok(())
    }
}
