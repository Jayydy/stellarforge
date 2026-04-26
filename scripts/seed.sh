#!/usr/bin/env bash
# =============================================================================
# scripts/seed.sh — StellarForge local testnet seed script
#
# Deploys and initializes all five StellarForge contracts with realistic sample
# data on a local Stellar network (default: standalone via stellar-cli).
#
# What it seeds:
#   forge-vesting  — 1 vesting schedule (1-year cliff, 4-year total)
#   forge-stream   — 1 pay-per-second token stream
#   forge-multisig — 1 2-of-3 multisig treasury with a sample proposal
#   forge-governor — 1 governance config with a sample proposal
#   forge-oracle   — 1 price feed (XLM/USDC)
#
# Prerequisites:
#   stellar-cli >= v25.2.0   (cargo install --locked stellar-cli)
#   A funded identity named "seed-admin" on the target network:
#     stellar keys generate seed-admin --network <NETWORK> --fund
#
# Usage:
#   bash scripts/seed.sh [--network testnet|standalone]
#
# The script is idempotent: re-running it skips steps that already succeeded
# by checking a local state file (.seed-state) for previously deployed IDs.
# =============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
NETWORK="${NETWORK:-standalone}"
IDENTITY="${IDENTITY:-seed-admin}"
STATE_FILE=".seed-state"

# Parse --network flag
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

log()  { echo "[seed] $*"; }
ok()   { echo "[seed] ✅ $*"; }
skip() { echo "[seed] ⏭  $*"; }
err()  { echo "[seed] ❌ $*" >&2; }

# ---------------------------------------------------------------------------
# State helpers — persist deployed contract IDs across runs (idempotency)
# ---------------------------------------------------------------------------
state_get() { grep -m1 "^$1=" "$STATE_FILE" 2>/dev/null | cut -d= -f2 || true; }
state_set() {
  local key="$1" val="$2"
  if grep -q "^${key}=" "$STATE_FILE" 2>/dev/null; then
    sed -i "s|^${key}=.*|${key}=${val}|" "$STATE_FILE"
  else
    echo "${key}=${val}" >> "$STATE_FILE"
  fi
}

touch "$STATE_FILE"

# ---------------------------------------------------------------------------
# Build contracts (produces target/wasm32v1-none/release/*.wasm)
# ---------------------------------------------------------------------------
log "Building contracts..."
stellar contract build 2>&1 | tail -5
ok "Build complete"

WASM_DIR="target/wasm32v1-none/release"

# ---------------------------------------------------------------------------
# Helper: deploy a contract if not already deployed
# ---------------------------------------------------------------------------
deploy_if_needed() {
  local name="$1" wasm="$2"
  local existing
  existing=$(state_get "${name}_id")
  if [[ -n "$existing" ]]; then
    skip "${name} already deployed: ${existing}"
    echo "$existing"
    return
  fi
  log "Deploying ${name}..."
  local id
  id=$(stellar contract deploy \
    --wasm "${WASM_DIR}/${wasm}" \
    --source "$IDENTITY" \
    --network "$NETWORK" 2>&1 | tail -1)
  state_set "${name}_id" "$id"
  ok "Deployed ${name}: ${id}"
  echo "$id"
}

# ---------------------------------------------------------------------------
# Helper: invoke a contract function, skip if state key already set
# ---------------------------------------------------------------------------
invoke_once() {
  local state_key="$1" contract_id="$2" fn="$3"
  shift 3
  local existing
  existing=$(state_get "$state_key")
  if [[ -n "$existing" ]]; then
    skip "${state_key} already done"
    return
  fi
  log "Invoking ${fn} on ${contract_id}..."
  local result
  result=$(stellar contract invoke \
    --id "$contract_id" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- "$fn" "$@" 2>&1 | tail -1) || { err "Failed: ${fn}"; return 1; }
  state_set "$state_key" "${result:-done}"
  ok "${fn}: ${result:-done}"
}

# ---------------------------------------------------------------------------
# Resolve the admin's Stellar address
# ---------------------------------------------------------------------------
ADMIN_ADDR=$(stellar keys address "$IDENTITY" --network "$NETWORK" 2>/dev/null) || {
  err "Identity '${IDENTITY}' not found. Create it with:"
  err "  stellar keys generate ${IDENTITY} --network ${NETWORK} --fund"
  exit 1
}
log "Admin address: ${ADMIN_ADDR}"

# Sample addresses for beneficiary / owners / voters (use admin for simplicity)
BENEFICIARY="$ADMIN_ADDR"
OWNER_A="$ADMIN_ADDR"
# Two additional placeholder addresses (replace with real funded accounts for full testing)
OWNER_B="GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"
OWNER_C="GBSC4HNWHHLKFBTWXYSXZJYS5IXZVLZQNMXPQR6ZUHYQBM7NKMJQZKN"

# ---------------------------------------------------------------------------
# 1. forge-vesting
# ---------------------------------------------------------------------------
VESTING_ID=$(deploy_if_needed "forge_vesting" "forge_vesting.wasm")

invoke_once "vesting_init" "$VESTING_ID" "initialize" \
  --token    "$ADMIN_ADDR" \
  --beneficiary "$BENEFICIARY" \
  --admin    "$ADMIN_ADDR" \
  --total_amount 1000000000 \
  --cliff_seconds 31536000 \
  --duration_seconds 126144000

# ---------------------------------------------------------------------------
# 2. forge-stream
# ---------------------------------------------------------------------------
STREAM_ID=$(deploy_if_needed "forge_stream" "forge_stream.wasm")

invoke_once "stream_create" "$STREAM_ID" "create_stream" \
  --sender   "$ADMIN_ADDR" \
  --token    "$ADMIN_ADDR" \
  --recipient "$BENEFICIARY" \
  --rate_per_second 100 \
  --duration_seconds 2592000

# ---------------------------------------------------------------------------
# 3. forge-multisig
# ---------------------------------------------------------------------------
MULTISIG_ID=$(deploy_if_needed "forge_multisig" "forge_multisig.wasm")

invoke_once "multisig_init" "$MULTISIG_ID" "initialize" \
  --owners   "[\"${OWNER_A}\",\"${OWNER_B}\",\"${OWNER_C}\"]" \
  --threshold 2 \
  --timelock_seconds 86400

invoke_once "multisig_propose" "$MULTISIG_ID" "propose" \
  --proposer "$ADMIN_ADDR" \
  --to       "$BENEFICIARY" \
  --token    "$ADMIN_ADDR" \
  --amount   500000000

# ---------------------------------------------------------------------------
# 4. forge-governor
# ---------------------------------------------------------------------------
GOVERNOR_ID=$(deploy_if_needed "forge_governor" "forge_governor.wasm")

invoke_once "governor_init" "$GOVERNOR_ID" "initialize" \
  --token          "$ADMIN_ADDR" \
  --quorum         1000000 \
  --voting_period  604800 \
  --timelock_delay 172800

invoke_once "governor_propose" "$GOVERNOR_ID" "propose" \
  --proposer    "$ADMIN_ADDR" \
  --title       "Seed Proposal: Enable streaming payroll" \
  --description "Authorize the treasury to fund a forge-stream contract for contributor payroll."

# ---------------------------------------------------------------------------
# 5. forge-oracle
# ---------------------------------------------------------------------------
ORACLE_ID=$(deploy_if_needed "forge_oracle" "forge_oracle.wasm")

invoke_once "oracle_init" "$ORACLE_ID" "initialize" \
  --admin               "$ADMIN_ADDR" \
  --staleness_threshold 3600

invoke_once "oracle_price_xlm_usdc" "$ORACLE_ID" "submit_price" \
  --base  "XLM" \
  --quote "USDC" \
  --price 1100000

invoke_once "oracle_price_btc_usdc" "$ORACLE_ID" "submit_price" \
  --base  "BTC" \
  --quote "USDC" \
  --price 6500000000000

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
ok "Seed complete. Contract IDs saved to ${STATE_FILE}."
echo ""
cat "$STATE_FILE"
