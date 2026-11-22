#!/usr/bin/env bash
# Integration smoke test for the will contract against a local standalone node.
#
# Exercises the full create -> check-in -> cancel lifecycle using the compiled
# WASM bindings and the installed Stellar CLI. Requires a running standalone
# node and a pinned Stellar CLI version.
set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
NETWORK="${NETWORK:-standalone}"
RPC_URL="${RPC_URL:-http://localhost:8000/soroban/rpc}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Standalone Network ; February 2017}"
SOURCE="${SOURCE:-will-owner}"
WILL_WASM="${WILL_WASM:-target/wasm32-unknown-unknown/release/will_contract.wasm}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() { printf '\n==> %s\n' "$*"; }

invoke() {
  stellar contract invoke \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --source "$SOURCE" \
    --id "$WILL_ID" \
    -- "$@"
}

# ---------------------------------------------------------------------------
# Deploy
# ---------------------------------------------------------------------------
log "Deploying will contract"
WILL_ID=$(stellar contract deploy \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --source "$SOURCE" \
  --wasm "$WILL_WASM")
echo "WILL_ID=$WILL_ID"

# ---------------------------------------------------------------------------
# Create will
#
# Beneficiary uses the current shape: { address, allocation } where allocation
# is the Allocation enum. Percentage shares are encoded as
# {"percentage": <bps>} (1 bp = 0.01 %); all percentages must sum to 10000.
# ---------------------------------------------------------------------------
log "Creating will"
BENEFICIARIES='[
  {"address": "'"$BENEFICIARY_1"'", "allocation": {"percentage": 6000}},
  {"address": "'"$BENEFICIARY_2"'", "allocation": {"percentage": 4000}}
]'

invoke create_will \
  --owner "$OWNER" \
  --token "$TOKEN" \
  --tokens "[$TOKEN]" \
  --beneficiaries "$BENEFICIARIES" \
  --checkin_period_days 30 \
  --grace_period_days 7 \
  --confirmation_delay_seconds 0

# ---------------------------------------------------------------------------
# Check in
#
# The current entry point takes `caller` (not the removed `owner` argument).
# ---------------------------------------------------------------------------
log "Checking in"
invoke check_in \
  --caller "$OWNER"

# ---------------------------------------------------------------------------
# Cancel
# ---------------------------------------------------------------------------
log "Cancelling will"
invoke cancel_will \
  --caller "$OWNER"

log "Integration lifecycle complete"
