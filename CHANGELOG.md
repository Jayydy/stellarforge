# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### forge-vesting

- `initialize(token, beneficiary, admin, total_amount, cliff_seconds, duration_seconds)` — deploy a linear vesting schedule with an optional cliff period
- `claim()` — withdraw all currently unlocked tokens (beneficiary only)
- `cancel()` — cancel vesting and return unvested tokens to admin (admin only)
- `get_status()` — returns `VestingStatus` with total, claimed, vested, claimable amounts, cliff status, and fully-vested flag
- `get_config()` — returns the full `VestingConfig`
- Events: `vesting_initialized`, `claimed`, `vesting_cancelled`

### forge-stream

- `create_stream(sender, token, recipient, rate_per_second, duration_seconds)` — start a pay-per-second token stream; pulls total tokens from sender upfront
- `withdraw(stream_id)` — withdraw all accrued tokens (recipient only)
- `cancel_stream(stream_id)` — cancel stream, pay out accrued tokens to recipient, return remainder to sender (sender only)
- `get_stream(stream_id)` — returns full `Stream` data
- `get_stream_status(stream_id)` — returns `StreamStatus` with streamed, withdrawn, withdrawable, remaining amounts, and active/finished flags
- Events: `stream_created`, `withdrawn`, `stream_cancelled`

### forge-multisig

- `initialize(owners, threshold, timelock_delay)` — set up an N-of-M multisig treasury
- `propose(proposer, to, token, amount)` — propose a token transfer (owners only)
- `approve(owner, proposal_id)` — approve a proposal; starts timelock when threshold is reached
- `reject(owner, proposal_id)` — reject a proposal (owners only)
- `execute(executor, proposal_id)` — execute an approved proposal after the timelock delay
- `get_proposal(proposal_id)` — returns `Proposal` data
- `get_owners()` — returns the list of owner addresses
- `get_threshold()` — returns the approval threshold

### forge-governor

- `initialize(config)` — configure the governor with vote token, voting period, quorum, and timelock delay
- `propose(proposer, title, description)` — create a governance proposal; returns proposal ID
- `vote(voter, proposal_id, support, weight)` — cast a token-weighted vote
- `finalize(proposal_id)` — finalize a proposal after voting ends; sets state to `Passed` or `Failed`
- `execute(executor, proposal_id)` — mark a passed proposal as executed after the timelock
- `get_proposal(proposal_id)` — returns `Proposal` data
- `get_config()` — returns `GovernorConfig`
- `has_voted(proposal_id, voter)` — returns whether an address has voted on a proposal

### forge-oracle

- `initialize(admin, staleness_threshold)` — set up the oracle with an admin and staleness window
- `submit_price(base, quote, price)` — submit a price for a trading pair (admin only); price scaled to 7 decimals
- `get_price(base, quote)` — returns `PriceData`; reverts with `PriceStale` if data exceeds the staleness threshold
- `get_price_unsafe(base, quote)` — returns `PriceData` without staleness check
- `set_staleness_threshold(new_threshold)` — update the staleness window (admin only)
- `transfer_admin(new_admin)` — transfer admin role to a new address
- `get_admin()` — returns the current admin address
- Events: `price_updated`
