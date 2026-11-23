#!/usr/bin/env bash
#
# run-deploy-sandbox.sh — smoke-test scripts/deploy-testnet.sh against a
# local Stellar standalone (Quickstart) container (#78).
#
# This script starts a throwaway Stellar standalone network inside Docker,
# generates a funded identity, builds the contract, deploys it, and verifies
# that get_contract_version returns the expected value.  It mirrors exactly
# what deploy-testnet.sh does against the live testnet, so any regression in
# the deployment logic is caught before it reaches CI or testnet.
#
# Prerequisites (all are available in the GitHub Actions ubuntu-latest runner):
#   - Docker
#   - stellar-cli >= 22.0.0  (`cargo install --locked stellar-cli`)
#   - rustup target wasm32v1-none
#
# Usage (local):
#   ./scripts/run-deploy-sandbox.sh
#
# Usage (CI): this script is invoked by .github/workflows/scripts-check.yml
# in the deploy-sandbox job.  It exits 0 on success and non-zero on any
# failure, causing the CI job to fail.
#
# Environment variables (all optional):
#   SANDBOX_IMAGE   Docker image for the Stellar quickstart container.
#                   Default: stellar/quickstart:latest
#   SANDBOX_PORT    Host port to expose the RPC on. Default: 8000
#   KEEP_CONTAINER  If set to "true", the container is not removed on exit.
#                   Useful for debugging failed runs locally.
#   KEEP_LOGS       If set to "true", the container's daemon logs are saved to
#                   "sandbox-logs-<container-id>.txt" before the container is
#                   removed.  This provides diagnostic information when a run
#                   fails.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SANDBOX_IMAGE="${SANDBOX_IMAGE:-stellar/quickstart:latest}"
SANDBOX_PORT="${SANDBOX_PORT:-8000}"
KEEP_CONTAINER="${KEEP_CONTAINER:-false}"
KEEP_LOGS="${KEEP_LOGS:-false}"
CONTAINER_NAME="williams-sandbox-$$"
RPC_URL="http://localhost:${SANDBOX_PORT}/soroban/rpc"
NETWORK_PASSPHRASE="Standalone Network ; February 2017"
DEPLOY_IDENTITY="sandbox-deployer"

log() { echo "[sandbox] $*"; }

cleanup() {
    EXIT_CODE=$?
    if [ "$KEEP_CONTAINER" = "true" ]; then
        log "KEEP_CONTAINER is true; leaving container $CONTAINER_NAME running."
    else
        if [ "$KEEP_LOGS" = "true" ]; then
            LOG_FILE="sandbox-logs-${CONTAINER_NAME}.txt"
            log "Saving container logs to $LOG_FILE"
            # Capture both stdout and stderr of the container daemon
            docker logs "$CONTAINER_NAME" > "$LOG_FILE" 2>&1 || log "Failed to capture logs for $CONTAINER_NAME"
        fi
        log "Removing container $CONTAINER_NAME"
        docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || log "Failed to remove container $CONTAINER_NAME"
    fi
    exit $EXIT_CODE
}

trap cleanup EXIT

# ... rest of the original script unchanged ...
