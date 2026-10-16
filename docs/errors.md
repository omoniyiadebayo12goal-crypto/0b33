# Error Codes and Handling

 This document defines the canonical error taxonomy for `subscription_vault` and the stable numeric codes clients should use for UX, retry policy, alerting, and backend integration.

> [!IMPORTANT]
> Breaking changes
> Legacy mixed codes such as `400`, `401`, `404`, `429`, and `10xx` have been consolidated into stable category ranges. Clients must match against the new canonical codes from [`contracts/subscription_vault/src/types.rs`](../contracts/subscription_vault/src/types.rs).

## Taxonomy

- `1000-1099` Auth: caller identity or permission failure.
- `2000-2099` Not found: missing resource or missing initialization.
- `3000-3099` Invalid args: caller supplied invalid input.
- `4000-4099` State transition: lifecycle, replay, emergency-stop, or other state conflict.
- `5000-5099` Accounting: balance, arithmetic, and pricing failures.
- `6000-6099` Limits: caps, quotas, pagination limits, and throttles.
- `7000-7099` Merchant config: fee, operations, and merchant configuration.
- `8000-8099` Token: token acceptance and decimals.
- `9000-9099` Subscription update: usage mode changes.
- `9100-9199` Schema migration: version compatibility.
- `10000-10099` Dispute/chargeback: dispute lifecycle errors.

## Canonical Table

| Code | Variant | Category | Meaning | Recommended handling |
|---|---|---|---|---|
| 1001 | `Unauthorized` | Auth | Required signer or admin identity mismatch. | Do not retry unchanged. Rebuild request with the correct signer. |
| 1002 | `Forbidden` | Auth | Caller is authenticated but not allowed for this resource. | Do not retry unchanged. Surface permission error. |
| 1003 | `SubscriberBlocklisted` | Auth | Subscriber is blocklisted from protected flows. | Stop retrying. Escalate to support/admin flow. |
| 1004 | `SelfRotation` | Auth | Admin rotation target equals current admin. | Fix request payload. |
| 2001 | `NotFound` | Not found | Requested subscription, token metadata, blocklist entry, or similar record is missing. | Verify identifiers before retrying. |
| 2002 | `NotInitialized` | Not found | Contract or config has not been initialized. | Admin setup required before retrying. |
| 3001 | `InvalidAmount` | Invalid args | Amount is zero, negative, or otherwise structurally invalid. | Fix input. No automatic retry. |
| 3002 | `InvalidInput` | Invalid args | Generic caller input validation failure. | Fix request parameters. |
| 3003 | `InvalidRecoveryAmount` | Invalid args | Recovery amount is zero or negative. | Fix input. |
| 3004 | `InvalidNewAdmin` | Invalid args | Proposed admin address is invalid for rotation. | Fix request payload. |
| 3005 | `MetadataKeyTooLong` | Invalid args | Metadata key exceeds length limit. | Trim key and retry. |
| 3006 | `MetadataValueTooLong` | Invalid args | Metadata value exceeds length limit. | Trim value and retry. |
| 3007 | `OraclePriceInvalid` | Invalid args | Oracle returned a non-positive price. | Treat as terminal for this request; investigate oracle data. |
| 4001 | `InvalidStatusTransition` | State transition | Requested lifecycle transition is not legal from current status. | Refresh state before presenting next action. |
| 4002 | `NotActive` | State transition | Operation requires an active subscription state. | Refresh state; do not blindly retry. |
| 4003 | `SubscriptionExpired` | State transition | Subscription has expired. | Stop retrying mutating operations on this subscription. |
| 4004 | `IntervalNotElapsed` | State transition | Interval charge attempted too early. | Safe to retry after the next eligible timestamp only. |
| 4005 | `Replay` | State transition | Duplicate charge/recovery/reference was detected. | Treat as idempotent duplicate. Do not retry with a new key for the same action. |
| 4006 | `RecoveryNotAllowed` | State transition | Recovery flow is not allowed in the current context. | Stop and inspect state/policy. |
| 4007 | `EmergencyStopActive` | State transition | Emergency stop blocks critical mutations. | Pause writes until admin clears emergency stop. |
| 4008 | `AlreadyInitialized` | State transition | Contract init was called more than once. | Do not retry. |
| 4009 | `MerchantPaused` | State transition | Merchant-wide pause blocks this action. | Retry only after merchant pause is removed. |
| 4010 | `Reentrancy` | State transition | Reentrancy guard detected a nested call. | Treat as security failure and investigate immediately. |
| 5001 | `InsufficientBalance` | Accounting | Vault, merchant, or refundable balance is insufficient. | Safe to retry only after balances change. |
| 5002 | `InsufficientPrepaidBalance` | Accounting | Usage charge exceeds prepaid balance. | Top up first, then retry. |
| 5003 | `BelowMinimumTopup` | Accounting | Deposit/top-up is below configured threshold. | Increase amount and retry. |
| 5004 | `Underflow` | Accounting | Arithmetic underflow or negative-balance invariant violation. | Treat as terminal and investigate; not user-retriable. |
| 5005 | `Overflow` | Accounting | Arithmetic overflow or counter overflow. | Treat as terminal and investigate; not user-retriable. |
| 5006 | `OracleNotConfigured` | Accounting | Oracle pricing is enabled but no oracle address is configured. | Admin/configuration fix required. |
| 5007 | `OraclePriceUnavailable` | Accounting | Oracle payload is missing or malformed. | Retry only after oracle data recovers. |
| 5008 | `OraclePriceStale` | Accounting | Oracle quote is older than allowed max age. | Retry only after a fresh quote exists. |
| 6001 | `SubscriptionLimitReached` | Limits | Subscription ID space has been exhausted. | Treat as terminal capacity failure. |
| 6002 | `LifetimeCapReached` | Limits | Lifetime charge cap is exhausted or would be exceeded. | Stop charging; surface terminal state to user. |
| 6003 | `UsageNotEnabled` | Limits | Usage charge attempted on a non-usage subscription. | Fix request or subscription type. |
| 6004 | `InvalidExportLimit` | Limits | Export/list limit is outside allowed bounds. | Fix pagination limit. |
| 6005 | `MetadataKeyLimitReached` | Limits | Metadata key quota is exhausted. | Delete/update keys before retrying. |
| 6006 | `MaxConcurrentSubscriptionsReached` | Limits | Subscriber already has maximum active subscriptions for the plan. | Stop and surface quota state. |
| 6007 | `CreditLimitExceeded` | Limits | Requested liability exceeds subscriber credit limit. | Reduce exposure or raise limit before retrying. |
| 6008 | `RateLimitExceeded` | Limits | Usage rate limit exceeded in current window. | Retry after the rate window resets. |
| 6009 | `UsageCapExceeded` | Limits | Usage cap would be exceeded for the billing period. | Retry only after a new billing period or cap change. |
| 6010 | `BurstLimitExceeded` | Limits | Usage call arrived too soon after prior call. | Retry after the minimum interval elapses. |
| 6021 | `MerchantTagLimitExceeded` | Limits | Requested merchant tag set exceeds `MAX_MERCHANT_TAGS`. | Reduce the tag list and retry. |
| 7001 | `InvalidFeeBips` | Merchant config | Fee basis nots exceed the maximum allowed value (10 000). | Fix `fee_bips` to be in range [0, 10000] and retry. |
| 7002 | `InvalidOperations` | Merchant config | Allowed-operations bitmask contains invalid `OP_*` bits. | Fix `allowed_operations` to use only valid `OP_*` bits and retry. |
| 7003 | `MustAllowChargeOperation` | Merchant config | The `OP_CHARGE` bit is not set; merchants must accept charges. | Set the `OP_CHARGE` bit in `allowed_operations` and retry. |
| 7004 | `MerchantNotApproved` | Merchant config | Merchant is not approved under whitelist mode. | Request approval from the admin or disable whitelist mode. |
| 7005 | `UnknownMerchantTag` | Merchant config | Tag is not present in the admin-controlled tag allowlist. | Fix input; use only tags returned by `get_tag_allowlist`. |
| 7006 | `DuplicateMerchantTag` | Merchant config | The same tag appears more than once in one `set_merchant_tags`/`set_tag_allowlist` call. | Remove the repeated tag and retry. |
| 9001 | `CannotChangeUsageMode` | Subscription update | Attempting to toggle usage_enabled on an existing subscription. | Cannot toggle usage_enabled on an existing subscription; create a new one. |
| 9101 | `SchemaMigrationDowngrade` | Schema migration | Stored schema version is newer than the binary. | Downgrade rejected; deploy the correct binary version. |
| 9102 | `SchemaVersionMismatch` | Schema migration | Stored schema version does not match expected version. | Migration rejected; deploy compatible binary. |
| 9103 | `SchemaMigrationVerificationFailed` | Schema migration | Post-migration storage invariants did not pass verification. | Treat as terminal; investigate migrated storage before retrying. |
| 10001 | `DisputeNotFound` | Not found | No dispute for the given ID. | Verify dispute ID. |
| 10002 | `DisputeAlreadyResolved` | State transition | Dispute has already been resolved. | Do not retry; inspect resolution. |
| 10003 | `DisputeNotResponded` | State transition | Cannot resolve an unresponded dispute before window elapses. | Retry after admin responds or window elapses. |
| 10004 | `DisputeWindowElapsed` | State transition | Dispute window has elapsed. | Check resolution rules. |
| 10005 | `DisputeAlreadyOpen` | State transition | A dispute is already open for this subscription. | Wait for resolution or inspect existing dispute. |
| 10006 | `DisputeAlreadyResponded` | State transition | Dispute is not in `Open` status. | Cannot respond twice. |
| 12001 | `RenewalWindowClosed` | State transition | Re-enabling auto-renewal after the one-interval renewal window has elapsed. | Cancel and recreate the subscription. Do not retry `set_auto_renew(true)`. |

## Handling `Error::Replay` (4005) in Production Clients

`Error::Replay` is returned when the contract detects a duplicate charge, recovery, or reference. It is **not** a generic failure and must not be treated as a retryable transient error. In production clients the correct behavior depends on *why* the duplicate was detected:

| Scenario | What happened | Correct client behavior |
|---|---|---|
| Already succeeded | The charge (identified by the same idempotency key) already committed successfully. | Safe to ignore. Record the charge as successful and do not create a new key. |
| Duplicate key collision | The client reused an idempotency key for a different logical action (different amount, subscription, or operation). | Client bug. Do not retry. Alert and fix key generation. |

The contract cannot always tell these apart from the error alone, so the client must use its own record of the idempotency key and the original request parameters.

### Recommended client algorithm

1. Before sending a charge, derive a deterministic idempotency key from `tx_hash` and the logical action (e.g. `subscription_id`, amount, period).
2. Persist the key and the request fingerprint before submitting.
3. On `Error::Replay` (4005):
   - If the key matches a previously committed charge for the same fingerprint, treat it as success (idmpotent no-op).
   - If the key matches a different fingerprint, raise a client-bug alert and do not retry.
4. Never generate a fresh key for the same logical action in response to `Replay`.

### Example: TypeScript

```ts
import { Error } from "./generated/subscription_vault";

interface ChargeRequest {
  subscriptionId: bigint;
  amount: bigint;
  period: number;
}

interface CommittedCharge {
  key: string;
  fingerprint: string;
}

const committedCharges = new Map<string, CommittedCharge>();

function fingerprint(req: ChargeRequest): string {
  return `${req.subscriptionId}:${req.amount}:${req.period}`;
}

async function charge(req: ChargeRequest, key: string): Promise<void> {
  const fp = fingerprint(req);
  try {
    await contract.charge(req.subscriptionId, req.amount, req.period, key);
    committedCharges.set(key, { key, fingerprint: fp });
  } catch (err) {
    if (err === Error.Replay) {
      const prior = committedCharges.get(key);
      if (prior && prior.fingerprint === fp) {
        // Already succeeded: safe to ignore.
        return;
      }
      // Duplicate key collision: client bug.
      throw new Error(
        `Replay for key ${key} but fingerprint mismatch (prior=${prior.fingerprint}, new=${fp})`,
      );
    }
    throw err;
  }
}
```

### Example: Rust

```rust
use std::collections::HashMap;

use subscription_vault::Error;

#[derive(Clone, Eq::PartialEq, Hash)]
struct ChargeRequest {
    subscription_id: u64,
    amount: i128,
    period: u32,
}

fn fingerprint(req: &ChargeRequest) -> String {
    format!("{}:{}:{}", req.subscription_id, req.amount, req.period)
}

async fn charge(
    req: ChargeRequest,
    key: &str,
    committed: &mut HashMap<String, String>,
) > Result<void, Error> {
    match contract.charge(req.subscription_id, req.amount, req.period, key) {
        Ok() => {
            committed.insert(key.to_string(), fingerprint(&req));
            Ok(()
        }
        Err(Error::Replay) => {
            match committed.get(key) {
                // Already succeeded: safe to ignore.
                Some(prior) if prior == &fingerprint(&req) => Ok(),
                // Duplicate key collision: client bug.
                Some(prior) => panic!(
                    "Replay for key {key} but fingerprint mismatch (prior={prior}, new={})",
                    fingerprint(&req)
                ),
                None => panic!("Replay for key {key} without a prior committed charge"),
            }
        }
        Err(e) => Err(e),
    }
}
```

## Retry Guidance

- Safe to retry later: `IntervalNotElapsed`, `EmergencyStopActive`, `OraclePriceStale`, `OraclePriceUnavailable`, `RateLimitExceeded`, `BurstLimitExceeded`, `InsufficientBalance`, `InsufficientPrepaidBalance`.
- Safe only after request changes: `Unauthorized`, `Forbidden`, `InvalidAmount`, `InvalidInput`, `InvalidExportLimit`, `UsageNotEnabled`, `CreditLimitExceeded`, `MaxConcurrentSubscriptionsReached`, `MerchantTagLimitExceeded`, `UnknownMerchantTag`, `DuplicateMerchantTag`.
- Treat as idempotent duplicate, not a fresh retry: `Replay`.
- Treat as terminal and operator-visible: `Overflow`, `Underflow`, `Reentrancy`, `SubscriptionLimitReached`, `LifetimeCapReached`.

## Security Notes

- Errors are intentionally coarse and must not leak sensitive internal balances beyond already-public business state.
- Charging paths avoid ambiguous reverted errors when a lifetime-cap overrun must persist a cancellation. In those cases the contract may return a semantic success/result while batch interfaces still map the condition to stable code `6002`.
- Never auto-retry a charge after `Replay`, `LifetimeCapReached`, `NotActive`, or `SubscriptionExpired`.
- Client payment UX should distinguish “insufficient balance” from “request rejected” to avoid duplicate funding or duplicate charge attempts.

## Source of Truth

- Enum and numeric assignments: [`contracts/subscription_vault/src/types.rs`](../contracts/subscription_vault/src/types.rs)
- Batch charge error-code mapping: [`contracts/subscription_vault/src/admin.rs`](../contracts/subscription_vault/src/admin.rs)
- Core charge semantics: [`contracts/subscription_vault/src/charge_core.rs`](../contracts/subscription_vault/src/charge_core.rs)

<!-- GENERATED:entrypoint-table:start -->
## Entrypoint Cross-Reference

This table is **generated** by `scripts/generate_error_table.py` and kept in sync
by CI (see `.github/workflows/docs.yml`). Do not edit the block between the
sentinel comments manually — run the script instead.

Column definitions:
- `**Emitting entrypoints**: source modules that contain `Error::<Variant>`.
  The public entrypoint name as exposed in `lib.rs` is listed where it differs
  from the internal module name.
- `**Recovery action**`: recommended remediation for integrators.
- `**Related event**`: Soroban event type emitted alongside this error, where applicable.

| Code | Variant | Category | Emitting entrypoints (modules) | Recovery action | Related event |
|---|---|---|---|---|---|
