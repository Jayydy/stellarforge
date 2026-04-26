#![cfg(test)]

use crate::CommonError;
use soroban_sdk::Env;

#[test]
fn test_common_error_values() {
    // Verify that the error codes match the expected values defined in the documentation
    assert_eq!(CommonError::AlreadyInitialized as u32, 1);
    assert_eq!(CommonError::NotInitialized as u32, 2);
    assert_eq!(CommonError::Unauthorized as u32, 3);
}

#[test]
fn test_common_error_debug() {
    let err = CommonError::Unauthorized;
    // Simple check to ensure Debug is implemented and works as expected
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("Unauthorized"));
}
