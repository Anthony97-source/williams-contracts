#![cfg(test)]

//! Regression tests for issue #107: `payout_retried` event now carries
//! resolution context so indexers can track failed-payout queue completion
//! from the event stream.

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _, Ledger},
    token::Client as TokenClient,
    vec,
    xdr::ScVal,
    Address, Env,
};

use crate::{
    event_test_support as events,
    issue_18_test::{FailableToken, FailableTokenClient},
    storage, Allocation, Beneficiary, WillContract, WillContractClient,
};

const DAY: u64 = 86_400;

fn advance(env: &Env, days: u64) {
    env.ledger().with_mut(|l| l.timestamp += days * DAY);
}

/// Creates a will funded with `amount` of a failable token, triggers it, and
/// releases so `distribute` records one failed payout for `beneficiary`.
fn make_failed_payout(
    env: &Env,
    owner: &Address,
    beneficiary: &Address,
    amount: i128,
    client: &WillContractClient<'_>,
) -> (u64, Address) {
    let token_id = env.register(FailableToken, ());
    let failable = FailableTokenClient::new(env, &token_id);
    failable.mint(owner, amount);

    let will_id = client.create_will(
        owner,
        &vec![env, (token_id.clone(), amount)],
        &vec![
            env,
            Beneficiary {
                address: beneficiary.clone(),
                allocation: Allocation::Percentage(10_000),
            },
        ],
        &90,
        &7,
        &vec![env],
        &0,
        &None,
        &0,
    );

    advance(env, 91);
    client.trigger_will(&will_id);
    advance(env, 8);
    failable.set_fail_all(&true);
    client.release_inheritance(&will_id, &None);
    failable.set_fail_all(&false);

    (will_id, token_id)
}

#[test]
fn retry_failed_payout_clears_record_and_delivers_funds() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_700_000_000);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let contract_id = env.register(WillContract, ());
    let client = WillContractClient::new(&env, &contract_id);

    let (will_id, token_id) = make_failed_payout(&env, &owner, &beneficiary, 500_000, &client);

    // Precondition: the failed payout is recorded.
    assert_eq!(
        env.as_contract(&contract_id, || storage::get_failed_payout(
            &env,
            will_id,
            &token_id,
            &beneficiary
        )),
        Some(500_000),
        "precondition: distribute must record the failed payout"
    );

    client.retry_failed_payout(&will_id, &token_id, &beneficiary);

    // The failed payout is cleared from storage.
    assert_eq!(
        env.as_contract(&contract_id, || storage::get_failed_payout(
            &env,
            will_id,
            &token_id,
            &beneficiary
        )),
        None,
        "successful retry must clear the failed-payout record"
    );

    // The beneficiary received the payout.
    let token_client = TokenClient::new(&env, &token_id);
    assert_eq!(
        token_client.balance(&beneficiary),
        500_000,
        "beneficiary must receive the retried payout"
    );
}

#[test]
fn retry_failed_payout_emits_event_with_resolution_context() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_700_000_000);

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let contract_id = env.register(WillContract, ());
    let client = WillContractClient::new(&env, &contract_id);

    let (will_id, token_id) = make_failed_payout(&env, &owner, &beneficiary, 500_000, &client);

    client.retry_failed_payout(&will_id, &token_id, &beneficiary);

    let emitted = env.events().all();
    let event = emitted
        .events()
        .iter()
        .find(|event| {
            events::topic_symbol(&env, event) == Some(symbol_short!("payretry"))
                && events::topic_u64(event, 1) == Some(will_id)
        })
        .expect("retry event for this will");
    let payload = events::data_vec(event).expect("retry event payload");
    assert_eq!(payload.len(), 4);
    assert_eq!(events::address_from(&payload[0], &env), Some(token_id));
    assert_eq!(events::address_from(&payload[1], &env), Some(beneficiary));
    assert_eq!(events::i128_from(&payload[2]), Some(500_000));
    assert_eq!(payload[3], ScVal::Bool(true));
}
