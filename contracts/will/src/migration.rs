//! Storage schema versioning for persisted `Will` entries (issue #446).
//!
//! Soroban encodes a `#[contracttype]` struct as an XDR map keyed by field
//! name, and decoding is strict: a stored entry that is missing a field the
//! current `Will` declares (or carries one it no longer declares) fails to
//! convert. Adding a field to `Will` would therefore make every will written
//! by an earlier contract version unreadable.
//!
//! This module holds the two pieces that keep old entries loadable:
//!
//! 1. [`decode_will`] — every read of a `DataKey::Will` entry goes through
//!    here. It tries the current layout first and then each known legacy
//!    layout, lifting a legacy entry into the current `Will` shape while
//!    keeping its original `schema_version`.
//! 2. [`upgrade`] — applies the per-version migration steps in order until
//!    the will reaches [`CURRENT_SCHEMA_VERSION`]. Called by
//!    `WillContract::migrate_will`, which persists the result.
//!
//! See the "Storage schema versioning" section of CONTRIBUTING.md for the
//! step-by-step process of introducing a new version.
//!
//! ## Schema v2 field ordering (issue #117)
//!
//! Soroban XDR serializes struct fields in declaration order, so the order of
//! `Will`'s fields directly affects the encoded byte layout. In v2 the fields
//! are grouped by kind to keep fixed-size scalars and addresses adjacent:
//!
//! 1. Identity and token metadata: `id`, `owner`, `token`, `is_native`,
//!    `balance`, `balances`.
//! 2. Beneficiaries: `beneficiaries`, `hashed_beneficiaries`.
//! 3. Timing: `checkin_period_days`, `grace_period_days`, `last_checkin`,
//!    `trigger_time`, `confirmation_deadline`.
//! 4. Status and guardians: `status`, `guardians`, `guardian_vote_weight`,
//!    `guardian_votes`, `guardian`
//! ...(remaining file content unchanged)
+
+// Use the authoritative schema version defined in `storage.rs`.
+use crate::storage::STORAGE_SCHEMA_VERSION;
+
+/// The current storage schema version. All migration logic should reference
+/// this constant to avoid divergence.
+pub const CURRENT_SCHEMA_VERSION: u32 = STORAGE_SCHEMA_VERSION;
+
+...(rest of the original file unchanged)...
