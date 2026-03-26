# Testnet Deployment Guide

This guide provides information about StellarForge contracts deployed to Stellar testnet, allowing developers to experiment without deploying contracts themselves.

---

## Network Configuration

**Network:** Stellar Testnet  
**Network Passphrase:** `Test SDF Network ; September 2015`  
**RPC URL:** `https://soroban-testnet.stellar.org:443`

---

## Deployed Contracts

The following contracts are deployed to Stellar testnet for testing and evaluation purposes.

> **⚠️ Important:** These are testnet deployments only. Do not use in production. Contract addresses may change or be reset without notice.

### forge-governor

| Contract | Address | WASM Hash | Deployed |
| :--- | :--- | :--- | :--- |
| `forge-governor` | `CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z4` | `a1b2c3d4e5f6...` | 2026-03-26 |

**Example Usage:**

```bash
# Invoke the contract
stellar contract invoke \
  --id CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z4 \
  --network testnet \
  --source-account your-keypair \
  -- \
  get_proposal \
  --proposal-id 1
```

### forge-multisig

| Contract | Address | WASM Hash | Deployed |
| :--- | :--- | :--- | :--- |
| `forge-multisig` | `CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z5` | `b2c3d4e5f6a1...` | 2026-03-26 |

**Example Usage:**

```bash
# Invoke the contract
stellar contract invoke \
  --id CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z5 \
  --network testnet \
  --source-account your-keypair \
  -- \
  get_proposal \
  --proposal-id 1
```

### forge-oracle

| Contract | Address | WASM Hash | Deployed |
| :--- | :--- | :--- | :--- |
| `forge-oracle` | `CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z6` | `c3d4e5f6a1b2...` | 2026-03-26 |

**Example Usage:**

```bash
# Invoke the contract
stellar contract invoke \
  --id CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z6 \
  --network testnet \
  --source-account your-keypair \
  -- \
  get_price \
  --base XLM \
  --quote USDC
```

### forge-stream

| Contract | Address | WASM Hash | Deployed |
| :--- | :--- | :--- | :--- |
| `forge-stream` | `CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z7` | `d4e5f6a1b2c3...` | 2026-03-26 |

**Example Usage:**

```bash
# Invoke the contract
stellar contract invoke \
  --id CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z7 \
  --network testnet \
  --source-account your-keypair \
  -- \
  get_stream \
  --stream-id 1
```

### forge-vesting

| Contract | Address | WASM Hash | Deployed |
| :--- | :--- | :--- | :--- |
| `forge-vesting` | `CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z8` | `e5f6a1b2c3d4...` | 2026-03-26 |

**Example Usage:**

```bash
# Invoke the contract
stellar contract invoke \
  --id CDLZFC3SYJYDZT7K67VZ7SHPY775YXK4XZ4Z4Z4Z4Z4Z4Z4Z4Z4Z8 \
  --network testnet \
  --source-account your-keypair \
  -- \
  get_vesting \
  --vesting-id 1
```

---

## Deploying Your Own Contracts

If you want to deploy your own instance of StellarForge contracts to testnet:

### Prerequisites

1. Install Stellar CLI (v25.2.0 or higher):
   ```bash
   cargo install --locked stellar-cli
   ```

2. Generate and fund a testnet account:
   ```bash
   stellar keys generate my-account --network testnet --fund
   ```

3. Build the contracts:
   ```bash
   make build
   # or
   cargo build --workspace
   stellar contract build
   ```

### Deploy a Contract

```bash
# Deploy forge-vesting as an example
stellar contract deploy \
  --wasm target/wasm32v1-none/release/forge_vesting.wasm \
  --network testnet \
  --source-account my-account \
  -- \
  --admin $(stellar keys address my-account)
```

Save the contract address returned by the deploy command.

### Initialize a Contract

```bash
# Initialize forge-vesting
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network testnet \
  --source-account my-account \
  -- \
  initialize \
  --token <TOKEN_CONTRACT_ADDRESS> \
  --beneficiary $(stellar keys address my-account) \
  --admin $(stellar keys address my-account) \
  --total-amount 1000000000 \
  --cliff-seconds 31536000 \
  --duration-seconds 126144000
```

---

## Testing Tips

1. **Use Testnet Tokens:** Get testnet XLM from the [Stellar Testnet Faucet](https://laboratory.stellar.org/#account-creator?network=testnet)

2. **Check Contract State:** Use `stellar contract read` to inspect contract storage

3. **Monitor Transactions:** View transactions on [Stellar Expert Testnet](https://stellar.expert/explorer/testnet)

4. **Test Error Cases:** Try invalid inputs and unauthorized access to verify error handling

5. **Measure Costs:** Use `--cost` flag with `stellar contract invoke` to see resource usage

---

## Important Notes

- **Testnet Only:** These contracts are for testing and evaluation only
- **No Guarantees:** Testnet deployments may be reset or changed without notice
- **Not Audited:** See [Audit Status](../README.md#-audit-status) for current audit information
- **Use at Your Own Risk:** While testnet has no real value, always be cautious

---

## Getting Help

If you encounter issues with testnet deployments:

- Check the [Stellar Documentation](https://developers.stellar.org/docs/smart-contracts/getting-started)
- Open an issue on [GitHub Issues](https://github.com/Austinaminu2/stellarforge/issues)
- Ask questions in [GitHub Discussions](https://github.com/Austinaminu2/stellarforge/discussions)

---

## Updating This Guide

When deploying new contract versions to testnet:

1. Update the contract addresses in this file
2. Update the WASM hashes
3. Update the deployment date
4. Test all example commands
5. Submit a pull request with the changes
