#![no_std]

//! # forge-vesting
//!
//! Token vesting contract with configurable cliff and linear release schedule.
//!
//! ## Overview
//! - Deploy with a token, beneficiary, total amount, cliff period, and vesting duration
//! - After the cliff, tokens unlock linearly every second
//! - Beneficiary can call `claim()` at any time to withdraw unlocked tokens
//! - Admin can cancel vesting and reclaim unvested tokens

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, token, Address, Env, Symbol};

// ── Storage Keys ──────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Config,
    Claimed,
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct VestingConfig {
    pub token: Address,
    pub beneficiary: Address,
    pub admin: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_seconds: u64,
    pub duration_seconds: u64,
    pub cancelled: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct VestingStatus {
    pub total_amount: i128,
    pub claimed: i128,
    pub vested: i128,
    pub claimable: i128,
    pub cliff_reached: bool,
    pub fully_vested: bool,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VestingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    CliffNotReached = 4,
    NothingToClaim = 5,
    Cancelled = 6,
    InvalidConfig = 7,
    SameAdmin = 8,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ForgeVesting;

#[contractimpl]
impl ForgeVesting {
    pub fn initialize(
        env: Env,
        token: Address,
        beneficiary: Address,
        admin: Address,
        total_amount: i128,
        cliff_seconds: u64,
        duration_seconds: u64,
    ) -> Result<(), VestingError> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(VestingError::AlreadyInitialized);
        }
        if total_amount <= 0 || duration_seconds == 0 || cliff_seconds > duration_seconds {
            return Err(VestingError::InvalidConfig);
        }

        admin.require_auth();

        let config = VestingConfig {
            token,
            beneficiary,
            admin,
            total_amount,
            start_time: env.ledger().timestamp(),
            cliff_seconds,
            duration_seconds,
            cancelled: false,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::Claimed, &0_i128);

        env.events().publish(
            (Symbol::new(&env, "vesting_initialized"),),
            (
                config.total_amount,
                config.cliff_seconds,
                config.duration_seconds,
            ),
        );

        Ok(())
    }

    pub fn claim(env: Env) -> Result<i128, VestingError> {
        let config: VestingConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VestingError::NotInitialized)?;

        if config.cancelled {
            return Err(VestingError::Cancelled);
        }

        config.beneficiary.require_auth();

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(config.start_time);

        if elapsed < config.cliff_seconds {
            return Err(VestingError::CliffNotReached);
        }

        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env.storage().instance().get(&DataKey::Claimed).unwrap_or(0);
        let claimable = vested - claimed;

        if claimable <= 0 {
            return Err(VestingError::NothingToClaim);
        }

        env.storage()
            .instance()
            .set(&DataKey::Claimed, &(claimed + claimable));

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(
            &env.current_contract_address(),
            &config.beneficiary,
            &claimable,
        );

        env.events().publish(
            (Symbol::new(&env, "claimed"),),
            (&config.beneficiary, claimable),
        );

        Ok(claimable)
    }

    pub fn cancel(env: Env) -> Result<(), VestingError> {
        let mut config: VestingConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VestingError::NotInitialized)?;

        config.admin.require_auth();

        if config.cancelled {
            return Err(VestingError::Cancelled);
        }

        let now = env.ledger().timestamp();
        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env.storage().instance().get(&DataKey::Claimed).unwrap_or(0);
        let returnable = config.total_amount - vested.max(claimed);

        config.cancelled = true;
        env.storage().instance().set(&DataKey::Config, &config);

        if returnable > 0 {
            let token_client = token::Client::new(&env, &config.token);
            token_client.transfer(&env.current_contract_address(), &config.admin, &returnable);
        }

        env.events().publish(
            (Symbol::new(&env, "vesting_cancelled"),),
            (&config.admin, returnable),
        );

        Ok(())
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), VestingError> {
        let mut config: VestingConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VestingError::NotInitialized)?;

        config.admin.require_auth();

        if config.admin == new_admin {
            return Err(VestingError::SameAdmin);
        }

        let old_admin = config.admin;
        config.admin = new_admin.clone();
        env.storage().instance().set(&DataKey::Config, &config);

        env.events().publish(
            (Symbol::new(&env, "admin_transferred"),),
            (&old_admin, &new_admin),
        );

        Ok(())
    }

    pub fn get_status(env: Env) -> Result<VestingStatus, VestingError> {
        let config: VestingConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VestingError::NotInitialized)?;

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(config.start_time);
        let cliff_reached = elapsed >= config.cliff_seconds;
        let vested = Self::compute_vested(&config, now);
        let claimed: i128 = env.storage().instance().get(&DataKey::Claimed).unwrap_or(0);
        let claimable = (vested - claimed).max(0);
        let fully_vested = vested >= config.total_amount;

        Ok(VestingStatus {
            total_amount: config.total_amount,
            claimed,
            vested,
            claimable,
            cliff_reached,
            fully_vested,
        })
    }

    pub fn get_config(env: Env) -> Result<VestingConfig, VestingError> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(VestingError::NotInitialized)
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn compute_vested(config: &VestingConfig, now: u64) -> i128 {
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
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
    };

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ForgeVesting);
        let token = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let admin = Address::generate(&env);
        (env, contract_id, token, beneficiary, admin)
    }

    fn setup_with_token() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, ForgeVesting);
        let token_admin = Address::generate(&env);
        let stellar_asset = env.register_stellar_asset_contract_v2(token_admin);
        let token_id = stellar_asset.address();
        let beneficiary = Address::generate(&env);
        let admin = Address::generate(&env);

        {
            let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
            token_client.mint(&contract_id, &1_000_000);
        }

        (env, contract_id, token_id, beneficiary, admin)
    }

    #[test]
    fn test_initialize_success() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        let result = client.try_initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_initialize_fails() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        let result = client.try_initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        assert_eq!(result, Err(Ok(VestingError::AlreadyInitialized)));
    }

    #[test]
    fn test_claim_before_cliff_fails() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &500, &1000);
        env.ledger().with_mut(|l| l.timestamp += 100);
        let result = client.try_claim();
        assert_eq!(result, Err(Ok(VestingError::CliffNotReached)));
    }

    #[test]
    fn test_get_status_before_cliff() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &500, &1000);
        let status = client.get_status();
        assert!(!status.cliff_reached);
        assert_eq!(status.claimable, 0);
        assert_eq!(status.claimed, 0);
    }

    #[test]
    fn test_invalid_config_rejected() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        let result = client.try_initialize(&token, &beneficiary, &admin, &1_000_000, &2000, &1000);
        assert_eq!(result, Err(Ok(VestingError::InvalidConfig)));
    }

    #[test]
    fn test_cancel_by_admin() {
        let (env, contract_id, token, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        let result = client.try_cancel();
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_cancel_fails() {
        let (env, contract_id, token, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        client.cancel();
        let result = client.try_cancel();
        assert_eq!(result, Err(Ok(VestingError::Cancelled)));
    }

    #[test]
    fn test_claim_after_cancel_fails() {
        let (env, contract_id, token, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        client.cancel();
        env.ledger().with_mut(|l| l.timestamp += 200);
        let result = client.try_claim();
        assert_eq!(result, Err(Ok(VestingError::Cancelled)));
    }

    #[test]
    fn test_fully_vested_after_duration() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        env.ledger().with_mut(|l| l.timestamp += 2000);
        let status = client.get_status();
        assert!(status.fully_vested);
        assert_eq!(status.vested, 1_000_000);
    }

    #[test]
    fn test_cancel_before_cliff_beneficiary_gets_zero_admin_gets_all() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &500, &1000);
        env.ledger().with_mut(|l| l.timestamp += 100);
        client.cancel();
        let tc = soroban_sdk::token::Client::new(&env, &token_id);
        assert_eq!(tc.balance(&beneficiary), 0);
        assert_eq!(tc.balance(&admin), 1_000_000);
    }

    #[test]
    fn test_cancel_after_cliff_splits_tokens_correctly() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &100, &1000);
        env.ledger().with_mut(|l| l.timestamp += 400);
        client.claim();
        client.cancel();
        let tc = soroban_sdk::token::Client::new(&env, &token_id);
        assert_eq!(tc.balance(&beneficiary), 400_000);
        assert_eq!(tc.balance(&admin), 600_000);
    }

    #[test]
    fn test_transfer_admin_success() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        let new_admin = Address::generate(&env);
        let result = client.try_transfer_admin(&new_admin);
        assert!(result.is_ok());
        let config = client.get_config();
        assert_eq!(config.admin, new_admin);
    }

    #[test]
    fn test_transfer_admin_same_admin_fails() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token, &beneficiary, &admin, &1_000_000, &100, &1000);
        let result = client.try_transfer_admin(&admin);
        assert_eq!(result, Err(Ok(VestingError::SameAdmin)));
    }

    // ── Issue #80: Zero Cliff Period Tests ────────────────────────────────────

    #[test]
    fn test_zero_cliff_initialize_succeeds() {
        let (env, contract_id, token, beneficiary, admin) = setup();
        let client = ForgeVestingClient::new(&env, &contract_id);
        let result = client.try_initialize(&token, &beneficiary, &admin, &1_000_000, &0, &1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_cliff_claim_succeeds_immediately() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &0, &1000);
        env.ledger().with_mut(|l| l.timestamp += 100);
        let result = client.try_claim();
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_cliff_correct_vested_amount_at_halfway() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &0, &1000);
        env.ledger().with_mut(|l| l.timestamp += 500);
        let status = client.get_status();
        assert!(status.cliff_reached);
        assert_eq!(status.vested, 500_000);
        assert_eq!(status.claimable, 500_000);
    }

    #[test]
    fn test_zero_cliff_fully_vested_after_duration() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &0, &1000);
        env.ledger().with_mut(|l| l.timestamp += 2000);
        let status = client.get_status();
        assert!(status.fully_vested);
        assert_eq!(status.vested, 1_000_000);
    }

    #[test]
    fn test_zero_cliff_claim_immediately_after_initialize() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &0, &1000);
        let result = client.try_claim();
        assert_eq!(result, Err(Ok(VestingError::NothingToClaim)));
    }

    #[test]
    fn test_zero_cliff_vesting_starts_immediately() {
        let (env, contract_id, token_id, beneficiary, admin) = setup_with_token();
        let client = ForgeVestingClient::new(&env, &contract_id);
        client.initialize(&token_id, &beneficiary, &admin, &1_000_000, &0, &1000);
        env.ledger().with_mut(|l| l.timestamp += 1);
        let status = client.get_status();
        assert!(status.cliff_reached);
        assert_eq!(status.vested, 1_000);
        assert_eq!(status.claimable, 1_000);
    }
}