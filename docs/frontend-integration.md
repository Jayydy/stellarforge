
# Frontend Integration Guide

This guide covers local development setup, constants/config management, empty state handling, and loading state patterns for frontends integrating with StellarForge contracts.

---

## Local Development Workflow (#463)

### Prerequisites
- Node.js v18+
- Stellar CLI v25.2.0+: `cargo install --locked stellar-cli`
- Rust target: `rustup target add wasm32v1-none`

### Setup

```bash
# 1. Build contracts
cargo build --workspace

# 2. Fund a testnet account
stellar keys generate dev-account --network testnet --fund

# 3. Deploy a contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/forge_vesting.wasm \
  --source dev-account \
  --network testnet
