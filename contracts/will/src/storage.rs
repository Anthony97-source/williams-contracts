//! Persistent storage helpers for the Williams contract.
//!
//! All will state lives in persistent storage keyed by a `DataKey`. Owner and
//! beneficiary indexes are maintained as `Vec<u64>` of will ids so the
//! contract can answer `get_wills_by_owner` / `get_wills_by_beneficiary`
//! without an off-chain indexer. Guardian votes are tracked per
//! `(will_id, guardian)` pair with a timestamp so they can expire over time,
//! and cleared independently when a guardian-release cycle resets.
+
+// Authoritative storage schema version constant. This should be the single
+// source of truth for the persisted schema version. All migration logic must
+// reference this constant instead of duplicating the literal.
+pub const STORAGE_SCHEMA_VERSION: u32 = 2;
+
+use soroban_sdk::{contracttype, panic_with_error, Address, Env, Map, Val, Vec};
+
+use crate::errors::WillError;
+use crate::migration;
+use crate::types::{
+    Beneficiary, GuardianVoteReason, ProtocolStats, TokenLockedBalance, Will, WillStatus,
+    WillStatusTransition,
+};
+
 // Number of ledgers in one calendar day, assuming a **5-second average ledger
 // close time** on the Stellar network.
@@
 // A redeployment is required to activate the updated value, because this
 // constant is compiled into the `.wasm` binary and is not configurable at
 // runtime.
@@
 // `LIFETIME_THRESHOLD` and `BUMP_AMOUNT` are expressed as multiples of
 // `DAY_IN_LEDGER
 ...(remaining file content unchanged)
