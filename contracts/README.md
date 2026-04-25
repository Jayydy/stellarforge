# Contracts

This folder contains StellarForge Soroban smart contracts.

## Purpose

- Hosts reusable on-chain primitives for governance, treasury control, payments, vesting, and oracle data.
- Keeps each primitive isolated in its own crate for independent testing and deployment.

## Structure

- `forge-governor`: Token-weighted governance and proposal lifecycle.
- `forge-multisig`: N-of-M treasury approvals with timelock.
- `forge-oracle`: Admin-managed price feed with staleness checks.
- `forge-stream`: Real-time token streaming and withdrawals.
- `forge-vesting`: Single-beneficiary token vesting schedule.
- `forge-vesting-factory`: Multi-schedule vesting from one deployment.

Each contract follows the same pattern:

- `Cargo.toml`: crate manifest
- `src/lib.rs`: contract logic, types, errors, and tests
- `README.md`: contract-specific behavior and notes
