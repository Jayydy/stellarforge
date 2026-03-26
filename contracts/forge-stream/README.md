# Forge Stream

## Resource Usage

> **Note:** Resource usage estimates are approximate and may vary based on contract state and input sizes. Run `stellar contract invoke` with `--cost` flag to measure actual usage for your specific use case.

### Function Resource Estimates

| Function | CPU Instructions | Memory (bytes) | Ledger Reads | Ledger Writes | Notes |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `create_stream` | ~85,000 | ~3,500 | 2 | 3 | Most expensive - validates inputs, creates stream |
| `withdraw` | ~65,000 | ~2,500 | 3 | 2 | Calculates accrued amount, transfers tokens |
| `cancel_stream` | ~70,000 | ~3,000 | 3 | 2 | Calculates final amounts, refunds/pays out |
| `pause_stream` | ~45,000 | ~2,000 | 2 | 1 | Marks stream as paused |
| `resume_stream` | ~50,000 | ~2,200 | 2 | 1 | Adjusts for paused time, resumes stream |
| `get_stream` | ~20,000 | ~1,500 | 2 | 0 | Read-only query |
| `get_withdrawable` | ~25,000 | ~1,800 | 2 | 0 | Calculates current accrued amount |

### Most Expensive Functions

1. **`create_stream`** (~85,000 CPU instructions)
   - Why: Validates sender/recipient, encodes stream parameters, creates multiple storage entries
   - Optimization tip: Reuse stream configurations for recurring payments

2. **`cancel_stream`** (~70,000 CPU instructions)
   - Why: Calculates final amounts, handles partial refunds, updates multiple storage entries
   - Optimization tip: Use pause/resume instead of cancel/recreate for temporary stops

### Cost Estimation

Soroban charges fees based on:
- **CPU Instructions:** ~0.0001 XLM per 10,000 instructions
- **Memory:** ~0.00001 XLM per byte
- **Ledger Entries:** ~0.001 XLM per read/write

**Example:** Creating a stream costs approximately:
- CPU: 85,000 instructions × 0.0001 XLM / 10,000 = 0.00085 XLM
- Memory: 3,500 bytes × 0.00001 XLM = 0.035 XLM
- Ledger: 5 operations × 0.001 XLM = 0.005 XLM
- **Total:** ~0.041 XLM per stream creation

---

## Known Limitations

- No mid-stream rate changes