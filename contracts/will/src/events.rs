//! Contract events published by Williams.
//!
//! Every state-changing entry point publishes exactly one event so that
//! off-chain indexers (such as the Williams SDK/app) can reconstruct will
//! history without re-simulating transactions. That includes the guardian
//! consent transitions: [`guardian_accepted_role`] and
//! [`guardian_rejected_role`] report when a named guardian becomes eligible to
//! vote in `guardian_trigger`, or loses that eligibility.

use soroban_sdk::{symbol_short, Address, Bytes, Env, Symbol, Vec};

use crate::types::{Beneficiary, Guardian};

// Existing event functions ... (unchanged)

/// Published when a guardian that was previously rejected is re‑appointed
/// (i.e. its consent status moves from `Rejected` back to `Pending`).
///
/// This allows off‑chain indexers to differentiate a simple status reset from a
/// fresh acceptance, which is required for correct consent‑dashboard behavior.
pub fn guardian_reappointed_role(env: &Env, will_id: u64, guardian: &Address) {
    env.events().publish(
        (symbol_short!("guardian_reappointed"), will_id),
        (guardian,),
    );
}
