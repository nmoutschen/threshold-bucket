# Leaky bucket with threshold

This crate implements a leaky bucket algorithm with threshold.

A user must present a valid [`Permit`] to acquire tokens in this bucket. The bucket will grant
a [`Permit`] if the number of tokens is higher than the specified threshold.

```rust
# use std::time::Duration;
use threshold_bucket::Bucket;

# || {
// Creating a new Bucket with a threshold of 100, max of 200, and a refill of 10 tokens every
// second.
// This bucket starts with 110 tokens.
let bucket = Bucket::builder()
    .refill_rate(10, Duration::from_secs(1))
    .threshold(100)
    .max(200)
    .build()?;

// We can acquire a permit since there are more than 100 tokens.
let permit = bucket.try_permit()?;

// We now remove 50 tokens, leaving 60 tokens behind.
let tokens = bucket.try_acquire(permit, 50);

// This will fail, as there are only 60 tokens left in the bucket, less than the threshold
// value.
let permit = bucket.try_permit()?;

# Ok::<_, Box<dyn std::error::Error>>(())
# };
```