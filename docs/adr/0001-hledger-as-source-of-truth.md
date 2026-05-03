# ADR 0001: Use hledger as the source of truth

## Status

Accepted

## Context

The first implementation handled bank CSV import, normalization, classification, and tax aggregation in one custom Python tool.

During development, it became clear that:

- CSV import is a bookkeeping concern
- transaction aggregation is a bookkeeping concern
- multiple accounts must be supported:
  - business current account
  - private current account
  - savings accounts
  - credit card accounts
- transfer handling and account relationships become easier in a dedicated bookkeeping system

## Decision

Use `hledger` as the source of truth for financial postings.

`ledger/` is responsible for:

- importing bank data
- maintaining journals
- reconciling accounts
- representing transfers and balances
- providing stable exports for downstream processing

`tax/` is responsible for:

- tax-specific reporting
- ELSTER-oriented views
- annual manual values
- depreciation and other tax-only adjustments
- verification against previous tax calculations

## Consequences

- custom bank CSV import logic in `tax/` should shrink or disappear
- `tax/` should stop treating raw bank exports as primary inputs
- the boundary between bookkeeping and tax reporting becomes explicit
- future accounts can be added in `ledger/` without redesigning tax logic
