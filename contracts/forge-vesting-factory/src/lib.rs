#![no_std]

//! # forge-vesting-factory
//!
//! A factory contract that manages multiple vesting schedules in a single deployment.
//!
//! ## Overview
//! - Create vesting schedules for multiple beneficiaries without deploying separate contracts
//! - Each schedule has its own token, beneficiary, admin, cliff, and duration
//! - Beneficiaries call `claim(schedule_id)` to withdraw unlocked tokens
//! - Admins call `cancel(schedule_id)` to cancel a schedule and reclaim unvested tokens
//! - Reduces deployment costs dramatically for multi-beneficiary vesting (e.g. employee grants)

use forge_errors::CommonError;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Env, Symbol,
};

// ── Storage Keys ──────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    /// Per-schedule configuration, keyed by schedule_id.
    Schedule(u64),
    /// Cumulative claimed amount per schedule, keyed by schedule_id.
    Claimed(u64),
    /// Monotonically increasing schedule counter.
    ScheduleCount,
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct ScheduleConfig {
    pub token: Address,
    pub beneficiary: Address,
    pub admin: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_seconds: u64,
    pub duration_seconds: u64,
    pub cancelled: bool,
}

/// Status snapshot for a vesting schedule.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VestingStatus {
    pub schedule_id: u64,
    pub total_amount: i128,
    pub claimed: i128,
    pub vested: i128,
    pub claimable: i128,
    pub cliff_reached: bool,
    pub fully_vested: bool,
    pub cancelled: bool,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FactoryError {
    #[from(CommonError)]
    Common(CommonError),
    ScheduleNotFound = 1,
    CliffNotReached = 3,
    NothingToClaim = 4,
    Cancelled = 5,
    InvalidConfig = 6,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ForgeVestingFactory;

#[contractimpl]
impl ForgeVestingFactory {
    /// Create a new vesting schedule and return its `schedule_id`.
    ///
    /// Transfers `total_amount` tokens from `admin` into the contract immediately.
    /// Requires authorization from `admin`.
    ///
    /// # Parameters
    /// - `token` — Soroban token contract address.
    /// - `beneficiary` — Address that will receive vested tokens.
    /// - `admin` — Address authorized to cancel this schedule.
    /// - `total_amount` — Total tokens to vest. Must be > 0.
    /// - `cliff_seconds` — Seconds before any tokens unlock. Must be ≤ `duration_seconds`.
    /// - `duration_seconds` — Total vesting duration in seconds. Must be > 0.
    ///
    /// # Returns
    /// `Ok(u64)` — the new schedule's ID.
    ///
    /// # Errors
    /// - [`FactoryError::InvalidConfig`] — invalid amounts or durations.
    pub fn create_schedule(
        env: Env,
        token: Address,
        beneficiary: Address,
        admin: Address,
        total_amount: i128,
        cliff_seconds: u64,
        duration_seconds: u64,
    ) -> Result<u64, FactoryError> {
        admin.require_auth();

        if total_amount <= 0 || duration_seconds == 0 || cliff_seconds > duration_seconds {
            return Err(FactoryError::Common(CommonError::InvalidConfig));
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ScheduleCount)
            .unwrap_or(0);

        let config = ScheduleConfig {
            token: token.clone(),
            beneficiary,
            admin,
            total_amount,
            start_time: env.ledger().timestamp(),
            cliff_seconds,
            duration_seconds,
            cancelled: false,
        };

        // Pull tokens from admin into the contract
        token::Client::new(&env, &token).transfer(
            &config.admin,
            &env.current_contract_address(),
            &total_amount,
        );

        env.storage()
            .persistent()
            .set(&DataKey::Schedule(id), &config);
        env.storage()
            .instance()
            .set(&DataKey::ScheduleCount, &(id + 1));

        env.events()
            .publish((Symbol::new(&env, "schedule_created"),), (id, total_amount));

        Ok(id)
    }

    /// Claim all currently vested and unclaimed tokens for a schedule.
    ///
    /// Only the beneficiary may call this. Tokens are transferred directly to the beneficiary.
    ///
    /// # Parameters
    /// - `schedule_id` — ID of the schedule to claim from.
    ///
    /// # Returns
    /// `Ok(i128)` — amount of tokens transferred.
    ///
    /// # Errors
    /// - [`FactoryError::ScheduleNotFound`]
    /// - [`FactoryError::Cancelled`]
    /// - [`FactoryError::CliffNotReached`]
    /// - [`FactoryError::NothingToClaim`]
    pub fn claim(env: Env, schedule_id: u64) -> Result<i128, FactoryError> {
        let config: ScheduleConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .ok_or(FactoryError::ScheduleNotFound)?;

        config.beneficiary.require_auth();

        if config.cancelled {
            return Err(FactoryError::Cancelled);
        }

        let now = env.ledger().timestamp();
        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(schedule_id))
            .unwrap_or(0);

        let elapsed = now.saturating_sub(config.start_time);
        if elapsed < config.cliff_seconds {
            return Err(FactoryError::CliffNotReached);
        }

        let claimable = (vested - claimed).max(0);
        if claimable == 0 {
            return Err(FactoryError::NothingToClaim);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Claimed(schedule_id), &(claimed + claimable));

        token::Client::new(&env, &config.token).transfer(
            &env.current_contract_address(),
            &config.beneficiary,
            &claimable,
        );

        env.events()
            .publish((Symbol::new(&env, "claimed"),), (schedule_id, claimable));

        Ok(claimable)
    }

    /// Cancel a vesting schedule. Vested tokens go to the beneficiary; remainder to admin.
    ///
    /// Only the admin may call this.
    ///
    /// # Parameters
    /// - `schedule_id` — ID of the schedule to cancel.
    ///
    /// # Errors
    /// - [`FactoryError::ScheduleNotFound`]
    /// - [`FactoryError::Cancelled`]
    /// - [`FactoryError::Unauthorized`]
    pub fn cancel(env: Env, schedule_id: u64) -> Result<(), FactoryError> {
        let mut config: ScheduleConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .ok_or(FactoryError::ScheduleNotFound)?;

        config.admin.require_auth();

        if config.cancelled {
            return Err(FactoryError::Cancelled);
        }

        let now = env.ledger().timestamp();
        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(schedule_id))
            .unwrap_or(0);

        let beneficiary_amount = (vested - claimed).max(0);
        let admin_amount = (config.total_amount - vested).max(0);

        // Write state before any transfers to prevent double-pay on retry.
        config.cancelled = true;
        env.storage()
            .persistent()
            .set(&DataKey::Schedule(schedule_id), &config);
        // Record the beneficiary payout as claimed so a retry cannot re-pay it.
        if beneficiary_amount > 0 {
            env.storage()
                .persistent()
                .set(&DataKey::Claimed(schedule_id), &(claimed + beneficiary_amount));
        }

        let token = token::Client::new(&env, &config.token);

        // Send unclaimed vested tokens to beneficiary
        if beneficiary_amount > 0 {
            token.transfer(
                &env.current_contract_address(),
                &config.beneficiary,
                &beneficiary_amount,
            );
        }

        // Return unvested tokens to admin
        if admin_amount > 0 {
            token.transfer(
                &env.current_contract_address(),
                &config.admin,
                &admin_amount,
            );
        }

        env.events()
            .publish((Symbol::new(&env, "schedule_cancelled"),), (schedule_id,));

        Ok(())
    }

    /// Return the current vesting status for a schedule.
    ///
    /// Read-only; does not modify state.
    ///
    /// # Parameters
    /// - `schedule_id` — ID of the schedule to query.
    ///
    /// # Returns
    /// `Ok(VestingStatus)` with current vested, claimed, and claimable amounts.
    ///
    /// # Errors
    /// - [`FactoryError::ScheduleNotFound`]
    pub fn get_status(env: Env, schedule_id: u64) -> Result<VestingStatus, FactoryError> {
        let config: ScheduleConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .ok_or(FactoryError::ScheduleNotFound)?;

        let now = env.ledger().timestamp();
        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(schedule_id))
            .unwrap_or(0);

        let elapsed = now.saturating_sub(config.start_time);
        let claimable = if elapsed >= config.cliff_seconds {
            (vested - claimed).max(0)
        } else {
            0
        };

        Ok(VestingStatus {
            schedule_id,
            total_amount: config.total_amount,
            claimed,
            vested,
            claimable,
            cliff_reached: elapsed >= config.cliff_seconds,
            fully_vested: vested >= config.total_amount,
            cancelled: config.cancelled,
        })
    }

    /// Return the total number of schedules ever created.
    pub fn get_schedule_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ScheduleCount)
            .unwrap_or(0)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn compute_vested(config: &ScheduleConfig, now: u64) -> i128 {
        if config.cancelled {
            return 0;
        }
        let elapsed = now.saturating_sub(config.start_time);
        if elapsed < config.cliff_seconds {
            return 0;
        }
        if elapsed >= config.duration_seconds {
            return config.total_amount;
        }
        (config.total_amount * elapsed as i128) / config.duration_seconds as i128
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
    };

    fn setup_token(env: &Env, admin: &Address, amount: i128) -> Address {
        let token_admin = Address::generate(env);
        let token = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();
        token::Client::new(env, &token).mint(admin, &amount);
        token
    }

    fn make_client(env: &Env) -> ForgeVestingFactoryClient {
        let id = env.register_contract(None, ForgeVestingFactory);
        ForgeVestingFactoryClient::new(env, &id)
    }

    #[test]
    fn test_create_schedule_success() {
        let env = Env::default();
        env.mock_all_auths();
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &100, &1_000);
        assert_eq!(id, 0);
        assert_eq!(client.get_schedule_count(), 1);
    }

    #[test]
    fn test_create_multiple_schedules_sequential_ids() {
        let env = Env::default();
        env.mock_all_auths();
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let token = setup_token(&env, &admin, 3_000);

        for expected_id in 0u64..3 {
            let b = Address::generate(&env);
            let id = client.create_schedule(&token, &b, &admin, &1_000, &0, &1_000);
            assert_eq!(id, expected_id);
        }
        assert_eq!(client.get_schedule_count(), 3);
    }

    #[test]
    fn test_claim_after_cliff() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &100, &1_000);

        // Before cliff — claim must fail
        env.ledger().with_mut(|l| l.timestamp = 50);
        let err = client.try_claim(&id).unwrap_err();
        assert_eq!(err, Ok(FactoryError::CliffNotReached));

        // After cliff — partial claim
        env.ledger().with_mut(|l| l.timestamp = 500);
        let claimed = client.claim(&id);
        assert_eq!(claimed, 500); // 500/1000 * 1000 = 500

        let status = client.get_status(&id);
        assert_eq!(status.claimed, 500);
        assert_eq!(status.claimable, 0);
    }

    #[test]
    fn test_claim_nothing_to_claim_after_full_claim() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 500);
        client.claim(&id);

        // Second claim at same timestamp — nothing new
        let err = client.try_claim(&id).unwrap_err();
        assert_eq!(err, Ok(FactoryError::NothingToClaim));
    }

    #[test]
    fn test_cancel_splits_tokens_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 1_000);
        let tok = token::Client::new(&env, &token_addr);

        let id = client.create_schedule(&token_addr, &beneficiary, &admin, &1_000, &0, &1_000);

        // 300s elapsed — 300 tokens vested
        env.ledger().with_mut(|l| l.timestamp = 300);
        client.cancel(&id);

        assert_eq!(tok.balance(&beneficiary), 300);
        assert_eq!(tok.balance(&admin), 700);
    }

    #[test]
    fn test_cancel_already_cancelled_fails() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &0, &1_000);
        client.cancel(&id);

        let err = client.try_cancel(&id).unwrap_err();
        assert_eq!(err, Ok(FactoryError::Cancelled));
    }

    #[test]
    fn test_get_status_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let client = make_client(&env);

        let err = client.try_get_status(&999).unwrap_err();
        assert_eq!(err, Ok(FactoryError::ScheduleNotFound));
    }

    #[test]
    fn test_multiple_concurrent_schedules_independent() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let b1 = Address::generate(&env);
        let b2 = Address::generate(&env);
        let token = setup_token(&env, &admin, 2_000);

        let id1 = client.create_schedule(&token, &b1, &admin, &1_000, &0, &1_000);
        let id2 = client.create_schedule(&token, &b2, &admin, &1_000, &0, &500);

        env.ledger().with_mut(|l| l.timestamp = 500);

        // id1: 500/1000 * 1000 = 500 vested
        // id2: fully vested (500 >= 500)
        let s1 = client.get_status(&id1);
        let s2 = client.get_status(&id2);

        assert_eq!(s1.vested, 500);
        assert!(!s1.fully_vested);

        assert_eq!(s2.vested, 1_000);
        assert!(s2.fully_vested);

        // Claiming id2 does not affect id1
        client.claim(&id2);
        let s1_after = client.get_status(&id1);
        assert_eq!(s1_after.claimed, 0);
    }

    #[test]
    fn test_fully_vested_claim_returns_total() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 1_000);
        let tok = token::Client::new(&env, &token_addr);

        let id = client.create_schedule(&token_addr, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 1_000);
        let claimed = client.claim(&id);
        assert_eq!(claimed, 1_000);
        assert_eq!(tok.balance(&beneficiary), 1_000);

        let status = client.get_status(&id);
        assert!(status.fully_vested);
        assert_eq!(status.claimable, 0);
    }

    #[test]
    fn test_invalid_config_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let b = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        // zero total_amount
        assert_eq!(
            client
                .try_create_schedule(&token, &b, &admin, &0, &0, &1_000)
                .unwrap_err(),
            Ok(FactoryError::InvalidConfig)
        );
        // zero duration
        assert_eq!(
            client
                .try_create_schedule(&token, &b, &admin, &1_000, &0, &0)
                .unwrap_err(),
            Ok(FactoryError::InvalidConfig)
        );
        // cliff > duration
        assert_eq!(
            client
                .try_create_schedule(&token, &b, &admin, &1_000, &500, &100)
                .unwrap_err(),
            Ok(FactoryError::InvalidConfig)
        );
    }

    // ── #436 comprehensive tests ──────────────────────────────────────────────

    /// claim() at 25%, 50%, 75%, and 100% of vesting duration returns correct amounts.
    #[test]
    fn test_claim_at_quarter_intervals() {
    /// Test multiple schedules for same beneficiary are fully independent.
    /// 
    /// Creates two schedules for the same beneficiary with different admins and amounts,
    /// then verifies that claiming from one doesn't affect the other, and cancelling
    /// one doesn't affect the other.
    #[test]
    fn test_multiple_schedules_same_beneficiary_independent() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 1_000);
        let tok = token::Client::new(&env, &token_addr);

        // No cliff, 1000s duration, 1000 tokens
        let id = client.create_schedule(&token_addr, &beneficiary, &admin, &1_000, &0, &1_000);

        // 25% — claim 250
        env.ledger().with_mut(|l| l.timestamp = 250);
        assert_eq!(client.claim(&id), 250);
        assert_eq!(tok.balance(&beneficiary), 250);

        // 50% — claim another 250
        env.ledger().with_mut(|l| l.timestamp = 500);
        assert_eq!(client.claim(&id), 250);
        assert_eq!(tok.balance(&beneficiary), 500);

        // 75% — claim another 250
        env.ledger().with_mut(|l| l.timestamp = 750);
        assert_eq!(client.claim(&id), 250);
        assert_eq!(tok.balance(&beneficiary), 750);

        // 100% — claim final 250
        env.ledger().with_mut(|l| l.timestamp = 1_000);
        assert_eq!(client.claim(&id), 250);
        assert_eq!(tok.balance(&beneficiary), 1_000);
    }

    /// claim() before cliff reverts with CliffNotReached.
    #[test]
    fn test_claim_before_cliff_reverts() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        // 500s cliff, 1000s duration
        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &500, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 499);
        assert_eq!(
            client.try_claim(&id).unwrap_err(),
            Ok(FactoryError::CliffNotReached)
        );
    }

    /// cancel() at halfway: beneficiary gets vested tokens, admin gets remainder.
    #[test]
    fn test_cancel_at_halfway_splits_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 1_000);
        let tok = token::Client::new(&env, &token_addr);

        let id = client.create_schedule(&token_addr, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 500);
        client.cancel(&id);

        assert_eq!(tok.balance(&beneficiary), 500);
        assert_eq!(tok.balance(&admin), 500);

        let status = client.get_status(&id);
        assert!(status.cancelled);
    }

    /// cancel() on an already-cancelled schedule reverts with Cancelled.
    #[test]
    fn test_cancel_already_cancelled_reverts() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &0, &1_000);
        client.cancel(&id);

        assert_eq!(
            client.try_cancel(&id).unwrap_err(),
            Ok(FactoryError::Cancelled)
        );
    }

    /// get_status() after a partial claim reflects correct claimed and claimable values.
    #[test]
    fn test_get_status_after_partial_claim() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 300);
        client.claim(&id);

        env.ledger().with_mut(|l| l.timestamp = 600);
        let status = client.get_status(&id);
        assert_eq!(status.claimed, 300);
        assert_eq!(status.vested, 600);
        assert_eq!(status.claimable, 300);
        assert!(!status.fully_vested);
        assert!(!status.cancelled);
    }

    /// Two concurrent schedules for different beneficiaries do not interfere.
    #[test]
    fn test_two_concurrent_schedules_no_interference() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let b1 = Address::generate(&env);
        let b2 = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 2_000);
        let tok = token::Client::new(&env, &token_addr);

        let id1 = client.create_schedule(&token_addr, &b1, &admin, &1_000, &0, &1_000);
        let id2 = client.create_schedule(&token_addr, &b2, &admin, &1_000, &0, &500);

        env.ledger().with_mut(|l| l.timestamp = 500);

        // id2 is fully vested; id1 is 50% vested
        client.claim(&id2);
        assert_eq!(tok.balance(&b2), 1_000);

        // id1 unaffected
        let s1 = client.get_status(&id1);
        assert_eq!(s1.claimed, 0);
        assert_eq!(s1.claimable, 500);

        client.claim(&id1);
        assert_eq!(tok.balance(&b1), 500);
    }

    /// get_schedule_count() increments correctly with each create_schedule() call.
    #[test]
    fn test_get_schedule_count_increments() {
        let env = Env::default();
        env.mock_all_auths();
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let token = setup_token(&env, &admin, 3_000);

        assert_eq!(client.get_schedule_count(), 0);
        for expected in 1u64..=3 {
            let b = Address::generate(&env);
            client.create_schedule(&token, &b, &admin, &1_000, &0, &1_000);
            assert_eq!(client.get_schedule_count(), expected);
        }
    }

    /// Non-beneficiary calling claim() reverts.
    #[test]
    fn test_non_beneficiary_cannot_claim() {
        let env = Env::default();
        // Do NOT mock_all_auths — we need real auth checking
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let attacker = Address::generate(&env);
        let token = setup_token(&env, &admin, 1_000);

        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "create_schedule",
                args: soroban_sdk::vec![
                    &env,
                    token.clone().into(),
                    beneficiary.clone().into(),
                    admin.clone().into(),
                    1_000i128.into(),
                    0u64.into(),
                    1_000u64.into(),
                ],
                sub_invokes: &[],
            },
        }]);
        let id = client.create_schedule(&token, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 500);

        // attacker tries to claim — auth will fail
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "claim",
                args: soroban_sdk::vec![&env, id.into()],
                sub_invokes: &[],
            },
        }]);
        // The contract calls beneficiary.require_auth() which will fail for attacker
        let result = client.try_claim(&id);
        assert!(result.is_err(), "non-beneficiary must not be able to claim");
    }

    /// Second cancel() call after a partial failure cannot double-pay the beneficiary.
    /// Simulates the bug scenario from #433: state is written before transfers,
    /// so a retry sees cancelled=true and returns Cancelled immediately.
    #[test]
    fn test_cancel_idempotent_no_double_pay() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 0);
        let client = make_client(&env);
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token_addr = setup_token(&env, &admin, 1_000);
        let tok = token::Client::new(&env, &token_addr);

        let id = client.create_schedule(&token_addr, &beneficiary, &admin, &1_000, &0, &1_000);

        env.ledger().with_mut(|l| l.timestamp = 400);
        client.cancel(&id);

        // Balances after first cancel: beneficiary=400, admin=600
        assert_eq!(tok.balance(&beneficiary), 400);
        assert_eq!(tok.balance(&admin), 600);

        // Second cancel must revert — no double-pay
        assert_eq!(
            client.try_cancel(&id).unwrap_err(),
            Ok(FactoryError::Cancelled)
        );

        // Balances unchanged
        assert_eq!(tok.balance(&beneficiary), 400);
        assert_eq!(tok.balance(&admin), 600);
        
        // Create different admins for each schedule
        let admin_a = Address::generate(&env);
        let admin_b = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let token = setup_token(&env, &admin_a, 3_000); // Fund admin_a initially
        
        // Fund admin_b separately
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&admin_a, &admin_b, &1_000);

        // Create schedule_a: 1000 tokens, 1000 second duration
        let schedule_a = client.create_schedule(&token, &beneficiary, &admin_a, &1_000, &0, &1_000);
        
        // Create schedule_b: 500 tokens, 1000 second duration  
        let schedule_b = client.create_schedule(&token, &beneficiary, &admin_b, &500, &0, &1_000);

        // Verify schedule count is 2
        assert_eq!(client.get_schedule_count(), 2);

        // Advance time to 50% (500 seconds)
        env.ledger().with_mut(|l| l.timestamp = 500);

        // Claim from schedule_a only
        let claimed_amount = client.claim(&schedule_a);
        assert_eq!(claimed_amount, 500); // 50% of 1000 = 500

        // Verify schedule_a status reflects the claim
        let status_a = client.get_status(&schedule_a);
        assert_eq!(status_a.claimed, 500);
        assert_eq!(status_a.claimable, 0);

        // Verify schedule_b is unaffected (still 0 claimed)
        let status_b = client.get_status(&schedule_b);
        assert_eq!(status_b.claimed, 0);
        assert_eq!(status_b.claimable, 250); // 50% of 500 = 250

        // Cancel schedule_b
        client.cancel(&schedule_b);

        // Verify schedule_b is cancelled
        let status_b_cancelled = client.get_status(&schedule_b);
        assert!(status_b_cancelled.cancelled);

        // Verify schedule_a is still active and claimable
        let status_a_after_cancel = client.get_status(&schedule_a);
        assert!(!status_a_after_cancel.cancelled);
        assert_eq!(status_a_after_cancel.claimed, 500); // Still has previous claim
        
        // Advance to full vesting for schedule_a
        env.ledger().with_mut(|l| l.timestamp = 1_000);
        
        // Should be able to claim remaining amount from schedule_a
        let final_claim = client.claim(&schedule_a);
        assert_eq!(final_claim, 500); // Remaining 50% = 500
        
        // Verify schedule_a is fully claimed
        let status_a_final = client.get_status(&schedule_a);
        assert_eq!(status_a_final.claimed, 1_000); // Total claimed = 1_000
        assert!(status_a_final.fully_vested);

        // Schedule count should remain 2 throughout
        assert_eq!(client.get_schedule_count(), 2);
    }
}
