# Forge Governor

## Resource Usage

> **Note:** Resource usage estimates are approximate and may vary based on contract state and input sizes. Run `stellar contract invoke` with `--cost` flag to measure actual usage for your specific use case.

### Function Resource Estimates

| Function       | CPU Instructions | Memory (bytes) | Ledger Reads | Ledger Writes | Notes                                                                 |
| :------------- | :--------------: | :------------: | :----------: | :-----------: | :-------------------------------------------------------------------- |
| `initialize`   |     ~50,000      |     ~2,000     |      0       |       2       | Stores admin and config                                               |
| `propose`      |     ~80,000      |     ~3,000     |      2       |       3       | Validates proposer, creates proposal                                  |
| `vote`         |     ~60,000      |     ~2,500     |      3       |       2       | Updates vote tally, records voter                                     |
| `execute`      |     ~100,000     |     ~3,500     |      4       |       2       | Most expensive - validates quorum, checks timelock, executes proposal |
| `get_proposal` |     ~20,000      |     ~1,500     |      2       |       0       | Read-only query                                                       |
| `get_vote`     |     ~15,000      |     ~1,000     |      1       |       0       | Read-only query                                                       |

### Most Expensive Functions

1. **`execute`** (~100,000 CPU instructions)
    - Why: Validates quorum requirements, checks timelock expiration, and executes the proposal action
    - Optimization tip: Ensure proposals are well-formed before submission to avoid failed executions

2. **`propose`** (~80,000 CPU instructions)
    - Why: Validates proposer eligibility, creates new proposal storage, and initializes vote tracking
    - Optimization tip: Reuse proposal templates for similar governance actions

### Cost Estimation

Soroban charges fees based on:

- **CPU Instructions:** ~0.0001 XLM per 10,000 instructions
- **Memory:** ~0.00001 XLM per byte
- **Ledger Entries:** ~0.001 XLM per read/write

**Example:** Executing a proposal costs approximately:

- CPU: 100,000 instructions × 0.0001 XLM / 10,000 = 0.001 XLM
- Memory: 3,500 bytes × 0.00001 XLM = 0.035 XLM
- Ledger: 6 operations × 0.001 XLM = 0.006 XLM
- **Total:** ~0.042 XLM per execution

---

## Known Limitations

- No vote delegation

---

## Tie-Breaking Behaviour

When `finalize()` is called after the voting period ends, the contract requires a **strict majority** to pass a proposal:

```
votes_for > votes_against
```

If `votes_for == votes_against` (a tie), the proposal resolves to **`Failed`**. There is no mechanism that breaks a tie in favour of the proposer, the quorum, or any other party. This behaviour is deterministic and unconditional.

### Summary table

| Condition                                                | Outcome  |
| :------------------------------------------------------- | :------- |
| `total_votes < quorum`                                   | `Failed` |
| `total_votes >= quorum` and `votes_for > votes_against`  | `Passed` |
| `total_votes >= quorum` and `votes_for == votes_against` | `Failed` |
| `total_votes >= quorum` and `votes_for < votes_against`  | `Failed` |

### Rationale

Requiring a strict majority means the status quo is preserved on a tie. A proposal must actively win — not merely draw — to advance to execution. This is the safest default for an on-chain treasury governor where an ambiguous outcome should never result in a token transfer.
