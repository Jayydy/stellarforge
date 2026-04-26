# Forge Vesting Factory

A single deployment that manages many vesting schedules.

## Purpose

- Create multiple vesting schedules without deploying one contract per beneficiary.
- Reduce deployment and operational overhead for team/investor token distributions.

## Core Functions

- `create_schedule(token, beneficiary, admin, total_amount, cliff_seconds, duration_seconds) -> schedule_id`
- `claim(schedule_id)`
- `cancel(schedule_id)`
- `get_status(schedule_id)`
- `get_schedule_count()`

## Notes

- Each schedule has its own token, beneficiary, admin, amount, cliff, and duration.
- On creation, tokens are transferred from admin into the factory contract.
- On cancel, vested amounts remain claimable while unvested amounts return to admin.
# forge-vesting-factory

A single-deployment factory that manages multiple vesting schedules without requiring a separate contract per beneficiary.

## Usage

```rust
// Create a schedule — returns a schedule_id
let id = client.create_schedule(&token, &beneficiary, &admin, &total_amount, &cliff_seconds, &duration_seconds);

// Beneficiary claims unlocked tokens
client.claim(&id);

// Admin cancels early — vested tokens go to beneficiary, unvested return to admin
client.cancel(&id);

// Read current status (vested, claimed, claimable, cancelled)
let status = client.get_status(&id);
```

## Storage Strategy

All persistent data uses `env.storage().persistent()`. There is no instance storage.

| Key | Storage | TTL extended in |
| :--- | :--- | :--- |
| `DataKey::ScheduleCount` | persistent | `create_schedule` |
| `DataKey::Schedule(id)` | persistent | `create_schedule`, `claim`, `cancel` |
| `DataKey::Claimed(id)` | persistent | `claim`, `cancel` |
| `DataKey::VestedAtCancel(id)` | persistent | `cancel` |

### Why `ScheduleCount` must be persistent

`ScheduleCount` is the monotonically increasing counter that assigns IDs to new schedules. If it were stored in instance storage and that entry expired, the counter would silently reset to 0. The next `create_schedule` call would reuse ID 0, overwriting the existing schedule in persistent storage and permanently destroying a beneficiary's vesting data.

Storing `ScheduleCount` in persistent storage and bumping its TTL on every `create_schedule` call ensures the counter survives as long as the individual schedule entries do.

### TTL parameters

All `extend_ttl` calls use `(17280, 34560)` — a minimum of 17 280 ledgers (~1 day) and a target of 34 560 ledgers (~2 days). These match the values used across other StellarForge contracts. Integrators running long-lived schedules should call `create_schedule`, `claim`, or `cancel` at least once every ~30 days, or submit a dedicated TTL-bump transaction, to keep entries alive on mainnet.
# Forge Vesting Factory

A factory contract that manages multiple vesting schedules in a single deployment.

## Overview

The Forge Vesting Factory allows you to create multiple vesting schedules for different beneficiaries without deploying separate contracts for each one. This dramatically reduces deployment costs for multi-beneficiary vesting scenarios such as employee grants, token distributions, or phased releases.

Each vesting schedule has its own:
- Token address
- Beneficiary address  
- Admin address
- Total amount
- Start time
- Cliff period
- Duration

## When to Use Factory vs Standalone

**Use Forge Vesting Factory when:**
- You need to create vesting schedules for multiple beneficiaries
- You want to minimize deployment costs
- You prefer centralized management of all schedules
- You have similar vesting parameters across multiple grants

**Use Forge Vesting (standalone) when:**
- You only need a single vesting schedule
- You need per-contract isolation and security boundaries
- You require contract-specific features like pause or transfer_admin
- You want maximum flexibility for individual schedule management

## Interface Summary

### Core Functions

- **`create_schedule(token, beneficiary, admin, total_amount, cliff_seconds, duration_seconds)`**
  - Creates a new vesting schedule and returns its schedule_id
  - Transfers total_amount tokens from admin into the contract immediately
  - Requires authorization from admin

- **`claim(schedule_id)`**
  - Claims all currently vested and unclaimed tokens for a schedule
  - Only the beneficiary may call this
  - Tokens are transferred directly to the beneficiary

- **`cancel(schedule_id)`**
  - Cancels a vesting schedule
  - Vested tokens go to the beneficiary; remainder to admin
  - Only the admin may call this

- **`get_status(schedule_id)`**
  - Returns the current vesting status for a schedule
  - Read-only function that shows vested, claimed, and claimable amounts

- **`get_schedule_count()`**
  - Returns the total number of schedules ever created
  - Useful for tracking schedule IDs

## Usage Example

```rust
use soroban_sdk::{Address, Env};
use forge_vesting_factory::{ForgeVestingFactoryClient, VestingStatus};

// Setup
let env = Env::default();
let factory_client = ForgeVestingFactoryClient::new(&env, &factory_address);

// Create addresses
let admin = Address::generate(&env);
let beneficiary1 = Address::generate(&env);
let beneficiary2 = Address::generate(&env);
let token_address = Address::generate(&env); // Your token contract

// Fund admin with tokens
// ... (token minting/transfer logic)

// Create multiple schedules for the same beneficiary
let schedule_a = factory_client.create_schedule(
    &token_address,
    &beneficiary1,
    &admin,
    &1000_000000,  // 1000 tokens
    &0,           // No cliff
    &31536000     // 1 year duration
);

let schedule_b = factory_client.create_schedule(
    &token_address,
    &beneficiary1,
    &admin,
    &500_000000,   // 500 tokens
    &2592000,      // 30 day cliff
    &63072000      // 2 year duration
);

// Advance time (in real scenario, this would be actual ledger time)
env.ledger().with_mut(|l| l.timestamp = 15780000); // ~6 months

// Claim from schedule_a only
let claimed_amount = factory_client.claim(&schedule_a);

// Check status of both schedules
let status_a = factory_client.get_status(&schedule_a);
let status_b = factory_client.get_status(&schedule_b);

assert!(status_a.claimed > 0);  // Schedule A has claims
assert_eq!(status_b.claimed, 0); // Schedule B unaffected

// Cancel schedule_b (doesn't affect schedule_a)
factory_client.cancel(&schedule_b);

// Schedule A is still active and claimable
let final_claim = factory_client.claim(&schedule_a);
```

## Known Limitations

- **No pause support** - Cannot pause individual schedules
- **No change_beneficiary** - Cannot modify beneficiary after creation
- **No transfer_admin per schedule** - Admin rights are fixed per schedule
- **No partial cancellation** - Can only cancel entire schedule
- **Single beneficiary per schedule** - Each schedule has exactly one beneficiary

## Resource Usage

> **Note:** Resource usage estimates are approximate and may vary based on contract state and input sizes. Run `stellar contract invoke` with `--cost` flag to measure actual usage for your specific use case.

### Function Resource Estimates

| Function | CPU Instructions | Memory (bytes) | Ledger Reads | Ledger Writes | Notes |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `create_schedule` | ~85,000 | ~3,500 | 2 | 3 | Most expensive - validates inputs, creates schedule, transfers tokens |
| `claim` | ~65,000 | ~2,800 | 3 | 2 | Calculates vested amount, updates claimed, transfers tokens |
| `cancel` | ~70,000 | ~3,000 | 3 | 2 | Calculates split, transfers to both parties |
| `get_status` | ~25,000 | ~1,800 | 3 | 0 | Read-only query with calculations |
| `get_schedule_count` | ~15,000 | ~1,200 | 1 | 0 | Simple counter read |

### Cost Estimation

Soroban charges fees based on:
- **CPU Instructions:** ~0.0001 XLM per 10,000 instructions
- **Memory:** ~0.00001 XLM per byte
- **Ledger Entries:** ~0.001 XLM per read/write

**Example:** Creating a vesting schedule costs approximately:
- CPU: 85,000 instructions × 0.0001 XLM / 10,000 = 0.00085 XLM
- Memory: 3,500 bytes × 0.00001 XLM = 0.035 XLM
- Ledger: 5 operations × 0.001 XLM = 0.005 XLM
- **Total:** ~0.041 XLM per schedule creation

## Error Handling

The contract returns specific error types for different failure scenarios:

- `ScheduleNotFound` - The provided schedule_id doesn't exist
- `CliffNotReached` - Cliff period hasn't elapsed yet
- `NothingToClaim` - No vested tokens available to claim
- `Cancelled` - Schedule has already been cancelled
- `InvalidConfig` - Invalid parameters (amount, duration, cliff)

## Events

The contract emits events for important state changes:

- `schedule_created` - Emitted when a new schedule is created
- `claimed` - Emitted when tokens are claimed from a schedule
- `schedule_cancelled` - Emitted when a schedule is cancelled

Events include relevant data such as schedule_id and amounts for easy tracking and indexing.
