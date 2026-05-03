# Ledger Tagging Rollout

## Goal

Introduce stable account-level metadata in `ledger/` without breaking the current import workflow.

## First step

Add `account` directives in `ledger/accounts.journal` for:

- real `assets:*` accounts
- transfer accounts
- tax-relevant exception accounts

This gives us a stable place for account-level semantics while keeping ordinary business-expense defaults in account names.

## Why not enable strict tag validation yet

Current imported journal comments contain many accidental `name:value` fragments such as:

- invoice references
- URLs
- exchange-rate comments
- payment processor payload fragments

hledger interprets these as tags in comments.

That means:

- `hledger tags --used` currently reports many accidental tags
- `tag` directives plus `hledger check tags` would fail immediately

So the rollout should be:

1. start using account tags intentionally
2. let `tax/` consume account tags where useful
3. only later decide whether comment cleanup or stricter tag validation is worth it

## Current intentional account tags

The initial account declarations model:

- payment account identity for `assets:*`
- transfer-account identity for `transfers:*`
- exception metadata for:
  - `expenses:phone`
  - `expenses:hardware:computer`
  - AOK insurance
  - UKV / Hansemerkur travel insurance
  - private liability insurance

All of these should use `elster_`-prefixed tags in `ledger/accounts.journal`.

## Out of scope for this step

- rewriting imported transaction comments
- enforcing a global allowed-tag vocabulary
- replacing `aggregation.yml` and `anlagen.yml` with account tags in `ledger/accounts.journal`
- changing current tax calculation behavior
