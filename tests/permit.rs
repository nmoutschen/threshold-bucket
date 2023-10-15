use std::time::Duration;

use threshold_bucket::{Bucket, Error};

#[test]
fn valid_permit() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Bucket::builder()
        .refill_rate(10, Duration::from_secs(10))
        .threshold(100)
        .max(200)
        .build()?;

    let permit = bucket.try_permit()?;
    let tokens = bucket.try_acquire(permit, 50)?;
    assert_eq!(tokens, 50);
    assert_eq!(bucket.available(), 60);

    Ok(())
}

#[test]
fn invalid_permit() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Bucket::builder()
        .refill_rate(10, Duration::from_secs(10))
        .threshold(100)
        .max(200)
        .build()?;

    let other_bucket = Bucket::builder()
        .refill_rate(10, Duration::from_secs(10))
        .threshold(100)
        .max(200)
        .build()?;

    let permit = other_bucket.try_permit()?;

    let res = bucket.try_acquire(permit, 50);

    assert!(matches!(res, Err(Error::InvalidPermit)));

    Ok(())
}
