//! # forge-constants
//!
//! Shared constants used across StellarForge contracts.
//!
//! This module provides centralized constants to avoid magic numbers
//! and ensure consistency across the entire StellarForge ecosystem.

/// Time-to-Live (TTL) constants for Stellar storage.
///
/// Persistent storage entries on Stellar expire unless their TTL is extended.
/// All TTLs are expressed in ledgers (1 ledger ≈ 5 seconds).
pub mod ttl {
    /// Instance TTL threshold: 17,280 ledgers ≈ 1 day
    pub const INSTANCE_TTL_THRESHOLD: u32 = 17_280;
    
    /// Instance TTL extension: 34,560 ledgers ≈ 2 days
    pub const INSTANCE_TTL_EXTEND: u32 = 34_560;
    
    /// Proposal TTL extension: 1,036,800 ledgers ≈ 60 days
    /// Applied to DataKey::Proposal entries. A proposal must survive its full
    /// lifecycle: voting_period + timelock_delay + a generous buffer.
    pub const PROPOSAL_TTL_EXTEND: u32 = 1_036_800;
    
    /// Vote TTL extension: 1,036,800 ledgers ≈ 60 days
    /// Applied to DataKey::Vote entries. A vote record must outlive the proposal
    /// it belongs to so that has_voted() remains reliable throughout the entire lifecycle.
    pub const VOTE_TTL_EXTEND: u32 = 1_036_800;
}

/// Error code constants to ensure consistent error numbering across contracts.
pub mod error_codes {
    /// Common error codes from forge-errors crate
    pub const ALREADY_INITIALIZED: u32 = 1;
    pub const NOT_INITIALIZED: u32 = 2;
    pub const UNAUTHORIZED: u32 = 3;
    
    /// Contract-specific error codes
    pub mod vesting {
        pub const CLIFF_NOT_REACHED: u32 = 4;
        pub const NOTHING_TO_CLAIM: u32 = 5;
        pub const CANCELLED: u32 = 6;
        pub const PAUSED: u32 = 9;
        pub const NOT_PAUSED: u32 = 10;
        pub const SAME_ADMIN: u32 = 8;
        pub const SAME_BENEFICIARY: u32 = 11;
        pub const BENEFICIARY_AS_ADMIN: u32 = 12;
        pub const VESTING_COMPLETE: u32 = 13;
    }
    
    pub mod factory {
        pub const SCHEDULE_NOT_FOUND: u32 = 1;
        pub const CLIFF_NOT_REACHED: u32 = 3;
        pub const NOTHING_TO_CLAIM: u32 = 4;
        pub const CANCELLED: u32 = 5;
        pub const INVALID_CONFIG: u32 = 6;
    }
    
    pub mod governor {
        pub const PROPOSAL_NOT_FOUND: u32 = 4;
        pub const VOTING_CLOSED: u32 = 5;
        pub const VOTING_STILL_OPEN: u32 = 6;
        pub const ALREADY_VOTED: u32 = 7;
        pub const QUORUM_NOT_REACHED: u32 = 8;
        pub const PROPOSAL_NOT_PASSED: u32 = 9;
        pub const TIMELOCK_NOT_ELAPSED: u32 = 10;
        pub const ALREADY_EXECUTED: u32 = 11;
        pub const ALREADY_CANCELLED: u32 = 12;
        pub const INVALID_WEIGHT: u32 = 13;
        pub const ALREADY_FINALIZED: u32 = 15;
        pub const VOTE_NOT_FOUND: u32 = 16;
    }
    
    pub mod stream {
        pub const STREAM_NOT_FOUND: u32 = 2;
        pub const UNAUTHORIZED: u32 = 2;
        pub const NOTHING_TO_WITHDRAW: u32 = 3;
        pub const ALREADY_CANCELLED: u32 = 4;
    }
    
    pub mod multisig {
        pub const PROPOSAL_NOT_FOUND: u32 = 1;
        pub const UNAUTHORIZED: u32 = 2;
        pub const ALREADY_EXECUTED: u32 = 3;
        pub const TIMED_OUT: u32 = 4;
        pub const ALREADY_APPROVED: u32 = 5;
        pub const INSUFFICIENT_APPROVALS: u32 = 6;
        pub const INVALID_THRESHOLD: u32 = 7;
    }
    
    pub mod oracle {
        pub const PRICE_NOT_FOUND: u32 = 1;
        pub const STALE_DATA: u32 = 2;
        pub const INVALID_PRICE: u32 = 3;
        pub const UNAUTHORIZED: u32 = 4;
    }
}

/// Time constants commonly used across contracts.
pub mod time {
    /// Seconds in various time units
    pub const SECONDS_PER_MINUTE: u64 = 60;
    pub const SECONDS_PER_HOUR: u64 = 3_600;
    pub const SECONDS_PER_DAY: u64 = 86_400;
    pub const SECONDS_PER_WEEK: u64 = 604_800;
    pub const SECONDS_PER_MONTH: u64 = 2_592_000; // 30 days
    pub const SECONDS_PER_YEAR: u64 = 31_536_000; // 365 days
    
    /// Common time periods used in tests and examples
    pub const ONE_HOUR: u64 = 3_600;
    pub const ONE_DAY: u64 = 86_400;
    pub const ONE_WEEK: u64 = 604_800;
    pub const THIRTY_DAYS: u64 = 2_592_000;
    pub const ONE_YEAR: u64 = 31_536_000;
}

/// Test constants for consistent test scenarios.
pub mod test {
    /// Common test amounts
    pub const SMALL_AMOUNT: i128 = 1_000;
    pub const MEDIUM_AMOUNT: i128 = 10_000;
    pub const LARGE_AMOUNT: i128 = 1_000_000;
    
    /// Common test durations
    pub const SHORT_DURATION: u64 = 100;
    pub const MEDIUM_DURATION: u64 = 1_000;
    pub const LONG_DURATION: u64 = 10_000;
    
    /// Common test cliff periods
    pub const NO_CLIFF: u64 = 0;
    pub const SHORT_CLIFF: u64 = 100;
    pub const MEDIUM_CLIFF: u64 = 500;
    pub const LONG_CLIFF: u64 = 1_000;
}
