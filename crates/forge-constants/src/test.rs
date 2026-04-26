#![cfg(test)]

use crate::{ttl, time, test};

#[test]
fn test_ttl_constants() {
    assert_eq!(ttl::INSTANCE_TTL_THRESHOLD, 17_280);
    assert_eq!(ttl::INSTANCE_TTL_EXTEND, 34_560);
}

#[test]
fn test_time_constants() {
    assert_eq!(time::SECONDS_PER_MINUTE, 60);
    assert_eq!(time::SECONDS_PER_HOUR, 3_600);
    assert_eq!(time::SECONDS_PER_DAY, 86_400);
}

#[test]
fn test_test_constants() {
    assert_eq!(test::SMALL_AMOUNT, 1_000);
    assert_eq!(test::MEDIUM_AMOUNT, 10_000);
    assert_eq!(test::LARGE_AMOUNT, 1_000_000);
}
