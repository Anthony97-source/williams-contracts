# Contract event specification

Every state-changing entry point of the `will` contract publishes an event so
off-chain indexers can reconstruct a will's lifecycle without re-simulating
transactions. The source of truth is
[`contracts/will/src/events.rs`](../contracts/will/src/events.rs).

Each event has two topics: a short symbol naming the event and, unless noted,
the `will_id` (`u64`). The data column lists the tuple published as event
data, in order.

| Topic | Emitted by | Data |
| ----- | ---------- | ---- |
| `created` | `create_will`, `clone_will`, `batch_create_wills`, `split_will` | `(owner: Address, token_count: u32, beneficiaries: Vec<Beneficiary>, checkin_deadline: u64)` |
| `confirmed` | `confirm_will` | `owner: Address` |
| `checkin` | `check_in` | `(owner: Address, next_deadline: u64)` |
| `batchchk` | `batch_check_in` (second topic is `owner`) | `(will_ids: Vec<u64>, count: u32)` |
| `triggered` | `trigger_will` | `grace_period_ends: u64` |
| `emerg` | `emergency_checkin` | `(owner: Address, next_deadline: u64)` |
| `grext` | `request_grace_extension` | `(owner: Address, extra_days: u64, cumulative_extension_days: u64)` |
| `released` | `release_inheritance`, `guardian_trigger` (on quorum) | `(token_count: u32, beneficiaries_count: u32)` |
| `bounty` | `release_inheritance` (keeper bounty paid) | `(keeper: Address, amount: i128)` — published once per token the bounty was paid from; with the default proportional selection (#109) that is one event per held token whose share rounds above zero. Guardian-triggered releases do not pay a keeper bounty. |
| `bountytok` | `set_keeper_bounty_token` | `token: Option<Address>` — `Some(t)` targets the whole bounty at `t`, `None` restores the default proportional split (#109). |
| `cancelled` | `cancel_will` | `(owner: Address, token_count: u32)` |
| `closed` | `close_will` | `owner: Address` |
| `archived` | `archive_will` | `(owner: Address, timestamp: u64, reason: Symbol)` |
| `benefup` | `update_beneficiaries` | `(owner: Address, beneficiary_count: u32, beneficiaries: Vec<Beneficiary>)` |
| `renounce` | `renounce_beneficiary` | `(beneficiary: Address, owner: Address, beneficiaries: Vec<Beneficiary>, trigger_time: Option<u64>)` — `trigger_time` is `Some(t)` when renounced during the grace period opened at `t`, `None` while `Active`. Renunciation is irreversible once emitted (#487). |
| `guardup` | `update_guardians`, `update_will_settings` (when guardian list changes) | `(owner: Address, guardians: Vec<Guardian>)` |
| `gaccept` | `accept_guardian_role` | `guardian: Address` |
| `greject` | `reject_guardian_role` | `guardian: Address` |
| `periodu` | `update_periods` | `(owner: Address, checkin_period_days: u64, grace_period_days: u64, next_deadline: u64)` |
| `setupd` | `update_will_settings` | `(owner: Address, updated_fields: Vec<Symbol>)` |
| `topup` | `top_up` | `(owner: Address, token: Address, amount: i128, new_balance: i128)` |
| `delegset` | `set_delegate` | `(owner: Address, delegate: Address)` |
| `delegclr` | `set_delegate` (clearing) | `owner: Address` |
| `gvote` | `guardian_trigger` | `(guardian: Address, weight: u32, total_weight: u32)` |
| `gcvote` | `guardian_cancel_trigger` | `(guardian: Address, weight: u32, total_weight: u32)` |
| `gcancel` | `guardian_cancel_trigger` (on quorum) | `(guardian: Address, next_deadline: u64)` |
| `merged` | `merge_wills` (topic is the surviving id) | `(owner: Address, consumed_will_id: u64, new_balance: i128, beneficiaries: Vec<Beneficiary>)` |
| `migrated` | `migrate_will` | `(owner: Address, from_version: u32, to_version: u32)` |
| `cloned` | `clone_will` (topic is the new id) | `(source_id: u64, owner: Address)` |
| `batch` | `batch_create_wills` (second topic is `owner`) | `will_ids: Vec<u64>` |
| `split` | `split_will` (topic is the original id) | `(new_id: u64, owner: Address, split_amount: i128)` |
| `hadd` | `add_hashed_beneficiary` | `(owner: Address, commitment: Bytes, percentage: u32)` |
| `hclaim` | `reveal_and_claim` | `(claimant: Address, amount: i128)` |
| `lftback` | `distribute` (leftover refunded to owner) | `(token: Address, owner: Address, amount: i128)` |
| `payfail` | `distribute`, `reveal_and_claim` (failed transfer) | `(token: Address, recipient: Address, amount: i128)` |
| `payretry` | `retry_failed_payout` | `(token: Address, recipient: Address, amount: i128, fully_resolved: bool)` |
| `payredir` | `redirect_failed_payout` | `(token: Address, old_recipient: Address, new_recipient: Address, amount: i128)` |
| `portmig` | `migrate_owner_portfolio` (second topic is `old_owner`) | `new_owner: Address` |

## `guardup`

`update_guardians` and `update_will_settings` (when the guardian list changes) both publish this event. The data is a tuple, not a scalar:

- `owner`: the will's owner.
- `guardians`: the full new guardian list (`Vec<Guardian>`, address + weight pairs), already applied. Indexers can reconstruct the current eligible voter set without a follow-up `get_will` call.

When `update_will_settings` is the caller, `guardup` is published alongside `setupd`. Indexers subscribed only to `guardup` will catch guardian changes regardless of which entry point made them.

## `gaccept` and `greject`

These events gate guardian vote eligibility. A guardian named in the will cannot cast a `guardian_trigger` vote until `gaccept` is emitted for them.

- `gaccept` — published by `accept_guardian_role`: `guardian: Address` (the consenting guardian).
- `greject` — published by `reject_guardian_role`: `guardian: Address` (the rejecting guardian).

Indexers reconstructing the eligible-voter set from events must track both. A guardian who appears in `guardup` but not yet in `gaccept` is named but not yet eligible.

## `grext`

Published by `request_grace_extension` when the owner stacks additional grace days on a `Triggered` will (#102).

- `owner`: the requesting owner.
- `extra_days`: days added in this call.
- `cumulative_extension_days`: total extension days accumulated across all calls for this grace cycle.

## `merged` (payload change)

The `merged` event carries a fourth field, `beneficiaries`, added after `new_balance`. Indexers that previously decoded only three fields must add the fourth:

`(owner: Address, consumed_will_id: u64, new_balance: i128, beneficiaries: Vec<Beneficiary>)`

`beneficiaries` is the surviving will's full post-merge beneficiary list (already proportionally recalculated). An indexer can reconstruct the new split without a follow-up `get_will` call.

## `hadd`

Published by `add_hashed_beneficiary` when the owner registers a hashed-commitment beneficiary (#185). Without this event, indexers reconstructing will state from events alone would miss every hashed beneficiary silently.

- `owner`: the will's owner.
- `commitment`: the commitment `Bytes` (the hash pre-image is not revealed until `reveal_and_claim`).
- `percentage`: basis-point allocation reserved for this commitment.

## `lftback`

Published by `distribute` when a `FixedAmount`-only will has a leftover balance that no beneficiary can receive (#383). The fixed amounts are paid and the remainder is returned to the owner.

- `token`: the token whose leftover balance was refunded.
- `owner`: the address that received the refund.
- `amount`: the refunded amount.

This only fires on under-allocated `FixedAmount` wills. It does not fire on `Percentage`-only wills.

## `portmig`

Published by `migrate_owner_portfolio` after the full owner index (all `OwnerWills` and `OwnerStatusWills` buckets) has been transferred to a new address and the old keys have been pruned.

- **Second topic**: `old_owner: Address` (not `will_id`; this event is portfolio-scoped, not will-scoped).
- **Data**: `new_owner: Address`.

Indexers maintaining identity mappings can update them from this single event without re-scanning individual wills.

## `archived`

`archive_will` moves a `Released`, `Cancelled` or `Settled` will out of active storage
and out of the owner and beneficiary indexes. After this event, `get_will`
returns `WillNotFound` and the will no longer shows up in `get_wills_by_owner`,
`get_wills_by_owner_and_status` or `get_wills_by_beneficiary`.

- `owner`: the will's owner.
- `timestamp`: ledger timestamp (Unix seconds) when the will was archived.
- `reason`: the terminal status the will was archived from, one of
  `released`, `cancelled` or `settled`.

Indexers should treat this as the will's final lifecycle event.

## `periodu`

`update_periods` publishes this event whenever the owner updates a will's check-in or
grace periods. Because `update_periods` resets `last_checkin` to the current transaction
timestamp (`now`), `next_deadline` is always a valid future deadline computed as
`now + checkin_period_days * 86_400`. Off-chain indexers and user interfaces are guaranteed
never to receive a stale or historical timestamp.

- `owner`: the will's owner.
- `checkin_period_days`: updated check-in period in days.
- `grace_period_days`: updated grace period in days.
- `next_deadline`: ledger timestamp (Unix seconds) for the next check-in deadline (`now + checkin_period_days * 86_400`), guaranteed to be strictly in the future.

## Failed operations

A failed invocation rolls back all of its storage writes and events, so no
event is ever published for a failed operation. The same applies to
`get_protocol_stats`, which counts successful transitions only.

## Events with non-`will_id` second topics

Most events use `(symbol, will_id)` as their two topics. The following use a different second topic; indexers that filter by topic 2 for a `will_id` will miss them:

| Topic | Second topic |
| ----- | ------------ |
| `batchchk` | `owner: Address` |
| `batch` | `owner: Address` |
| `portmig` | `old_owner: Address` |

## Compatibility changelog

These corrections reflect the state of the contract at commit `3210249c79d425b56842a555e77edbf86cdd638f`. Indexers that implemented an earlier version of this table should update their decoders as follows.

**Breaking payload changes** (wrong shapes in the previous table):

- `guardup` data was documented as a scalar `owner: Address`; it is a tuple `(owner: Address, guardians: Vec<Guardian>)`. Decoders expecting a scalar will fail to deserialise.
- `merged` data was documented as a 3-tuple `(owner, consumed_will_id, new_balance)`; it is a 4-tuple with `beneficiaries: Vec<Beneficiary>` appended. Decoders will silently drop the beneficiary list or fail.
- `payretry` data was documented as a 3-tuple `(token, recipient, amount)`; it is a 4-tuple with `fully_resolved: bool` appended (#107). Decoders will silently drop the resolution flag.

**Emitter correction:**

- `bounty` previously listed `guardian_trigger` as a co-emitter. Guardian-triggered releases do not pay a keeper bounty. Only `release_inheritance` emits `bounty`.

**New events** (not present in the previous table):

| Topic | First published |
| ----- | --------------- |
| `grext` | `request_grace_extension` (#102) |
| `gaccept` | `accept_guardian_role` |
| `greject` | `reject_guardian_role` |
| `hadd` | `add_hashed_beneficiary` (#185) |
| `lftback` | `distribute` leftover refund (#383) |
| `portmig` | `migrate_owner_portfolio` |

Topic-redesign work (#119) and the bounty token-identity proposal are out of scope for this document and tracked separately.
