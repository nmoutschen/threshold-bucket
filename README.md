# Leaky bucket with threshold

This crate implements a leaky bucket algorithm with threshold. This can be used in situations where
one wants to limit operations based on cost metrics, but the cost can only be calculated after the
operation has run.

To do so, the [`Bucket`] consists of three main parameters:

* A **refill rate**, adding a number of new tokens in the bucket at regular interval.
* A **maximum capacity**, above which tokens will leak out of the bucket.
* A **threshold**, at which the bucket will stop granting new [`Permit`]s.

To take tokens from the [`Bucket`], a user must present a valid [`Permit`]. The bucket will only
grant a [`Permit`] if the number of tokens in the bucket is higher than the threshold.

```rust
# use std::time::Duration;
use threshold_bucket::{
    refill::RateConfig,
    permit::ThresholdConfig,
    Bucket,
};

# || {
// Creating a new Bucket with a threshold of 100, max of 200, and a refill of 10 tokens every
// second.
// This bucket starts with 110 tokens.
let refill_rate = RateConfig {
    quantity: 10,
    interval: Duration::from_secs(1),
    max: 200,
};
let threshold = ThresholdConfig {
    threshold: 100,
};

let bucket = Bucket::builder()
    .rate(refill_rate)
    .threshold(threshold)
    .initial(110)
    .build()?;

// We can acquire a permit since there are more than 100 tokens.
let permit = bucket.get_permit().unwrap();

// We now remove 50 tokens, leaving 60 tokens behind.
let tokens = bucket.try_acquire(permit, 50);

// This will return `None`, as there are only 60 tokens left in the bucket, less than the threshold
// value.
let permit = bucket.get_permit();

# Ok::<_, Box<dyn std::error::Error>>(())
# };
```