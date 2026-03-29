# Forge Multisig

## Resource Usage

> **Note:** Resource usage estimates are approximate and may vary based on contract state and input sizes. Run `stellar contract invoke` with `--cost` flag to measure actual usage for your specific use case.

### Function Resource Estimates

| Function | CPU Instructions | Memory (bytes) | Ledger Reads | Ledger Writes | Notes |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `initialize` | ~60,000 | ~2,500 | 0 | 3 | Stores owners, threshold, and timelock |
| `propose` | ~70,000 | ~3,000 | 2 | 2 | Validates proposer, creates proposal |
| `approve` | ~50,000 | ~2,000 | 3 | 1 | Records approval, checks threshold |
| `execute` | ~90,000 | ~3,500 | 4 | 2 | Most expensive - validates threshold, checks timelock, transfers funds |
| `get_proposal` | ~20,000 | ~1,500 | 2 | 0 | Read-only query |
| `is_approved` | ~15,000 | ~1,000 | 1 | 0 | Read-only query |

### Most Expensive Functions

1. **`execute`** (~90,000 CPU instructions)
   - Why: Validates approval threshold, checks timelock expiration, and executes token transfer
   - Optimization tip: Ensure all required approvals are collected before attempting execution

2. **`propose`** (~70,000 CPU instructions)
   - Why: Validates proposer eligibility and creates new proposal storage
   - Optimization tip: Batch similar proposals when possible

### Cost Estimation

Soroban charges fees based on:
- **CPU Instructions:** ~0.0001 XLM per 10,000 instructions
- **Memory:** ~0.00001 XLM per byte
- **Ledger Entries:** ~0.001 XLM per read/write

**Example:** Executing a multisig proposal costs approximately:
- CPU: 90,000 instructions × 0.0001 XLM / 10,000 = 0.0009 XLM
- Memory: 3,500 bytes × 0.00001 XLM = 0.035 XLM
- Ledger: 6 operations × 0.001 XLM = 0.006 XLM
- **Total:** ~0.042 XLM per execution

---

## Known Limitations

- Owner list is fixed after initialization
- **Threshold is immutable after initialization.** If a future version adds `update_threshold()`, the `approved_at` logic in `approve()` must be reviewed carefully. Lowering the threshold could cause a proposal to retroactively meet the new threshold at an earlier `approved_at` timestamp, potentially allowing premature execution. Any threshold mutation must preserve the original `approved_at` timestamp or implement additional safeguards.