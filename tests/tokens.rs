use std::time::Duration;

use threshold_bucket::{Bucket, Error};

#[test]
fn available_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Bucket::builder()
        .refill_rate(100, Duration::from_secs(10))
        .max(200)
        .build()?;

    let tokens = bucket.try_acquire(bucket.try_permit()?, 50)?;
    assert_eq!(tokens, 50);
    assert_eq!(bucket.available(), 50);

    let tokens = bucket.try_acquire(bucket.try_permit()?, 50)?;
    assert_eq!(tokens, 50);
    assert_eq!(bucket.available(), 0);

    Ok(())
}

#[test]
fn exceed_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Bucket::builder()
        .refill_rate(100, Duration::from_secs(10))
        .max(200)
        .build()?;

    let tokens = bucket.try_acquire(bucket.try_permit()?, 50)?;
    assert_eq!(tokens, 50);
    assert_eq!(bucket.available(), 50);

    let res = bucket.try_acquire(bucket.try_permit()?, 60);

    // Waiting time should be approx. 10 seconds (40 tokens missing, with 100 tokens/10s)
    if let Err(Error::NotEnoughTokens(wait_for)) = res {
        assert!(wait_for > Duration::from_secs(9));
        assert!(wait_for < Duration::from_secs(11));
    } else {
        assert!(false, "invalid response");
    }

    Ok(())
}

#[test]
fn exceed_threshold() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Bucket::builder()
        .refill_rate(10, Duration::from_secs(10))
        .threshold(100)
        .max(200)
        .build()?;

    let tokens = bucket.try_acquire(bucket.try_permit()?, 55)?;
    assert_eq!(tokens, 55);
    assert_eq!(bucket.available(), 55);

    let res = bucket.try_permit();
    // Waiting time should be approx. 50 seconds (45 tokens missing, with 10 tokens/10s)
    if let Err(Error::NotEnoughTokens(wait_for)) = res {
        assert!(dbg!(wait_for) > Duration::from_secs(49));
        assert!(wait_for < Duration::from_secs(51));
    } else {
        assert!(false, "invalid response");
    }

    Ok(())
}
