#![cfg(test)]

//! Regression tests for issue #18: `reveal_and_claim` executed token
//! transfers directly, so a single failing transfer panicked the whole
//! claim — no failed-payout record was created and a multi-token hashed
//! beneficiary could not receive *any* of their assets. Failures now flow
//! through the same `pay_or_record_failure` ledger `distribute` uses (#459),
//! so the claim commits and each token's share is retried individually.

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    vec,
    xdr::ToXdr,
    Address, Bytes, Env, Vec as SorobanVec,
};

use crate::{storage, Allocation, Beneficiary, WillContract, WillContractClient, PREIMAGE_LENGTH};

const DAY: u64 = 86_400;

#[contracttype]
#[derive(Clone)]
enum TokenDataKey {
    Balance(Address),
    FailAll,
}

/// A minimal SEP-41-shaped token whose transfers can be switched to fail
/// wholesale, simulating a paused token contract or a destination the token
/// refuses to pay.
#[contract]
pub struct FailableToken;

#[contractimpl]
impl FailableToken {
    pub fn mint(env: Env, to: Address, amount: i128) {
        let key = TokenDataKey::Balance(to.clone());
        let balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(balance + amount));
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .get(&TokenDataKey::Balance(id))
            .unwrap_or(0)
    }

    /// `create_will` and `batch_create_wills` probe every token with a
    /// read-only `decimals()` call, so this mock must answer it like a real
    /// SEP-41 token to be usable as a will's locked asset.
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    /// Transfer tokens while optionally rejecting impossible movements.
    ///
    /// * Rejects negative amounts.
    /// * Rejects transfers that would cause the sender's balance to go below zero.
    /// * Honors the `FailAll` switch used in other tests to simulate a fully
    ///   paused token contract.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), soroban_sdk::Error> {
        // Disallow nonsensical transfer amounts.
        if amount <= 0 {
            return Err(soroban_sdk::Error::from_contract_error(1)); // Invalid amount
        }

        // If the token is globally set to fail, abort.
        if env.storage().instance().has(&TokenDataKey::FailAll) {
            return Err(soroban_sdk::Error::from_contract_error(2)); // Forced failure
        }

        // Ensure the sender has sufficient balance.
        let sender_key = TokenDataKey::Balance(from.clone());
        let sender_balance: i128 = env.storage().instance().get(&sender_key).unwrap_or(0);
        if sender_balance < amount {
            return Err(soroban_sdk::Error::from_contract_error(3)); // Insufficient balance
        }

        // Perform the transfer.
        env.storage().instance().set(&sender_key, &(sender_balance - amount));
        let receiver_key = TokenDataKey::Balance(to.clone());
        let receiver_balance: i128 = env.storage().instance().get(&receiver_key).unwrap_or(0);
        env.storage().instance().set(&receiver_key, &(receiver_balance + amount));
        Ok(())
    }
}

// The remainder of the file is unchanged.
