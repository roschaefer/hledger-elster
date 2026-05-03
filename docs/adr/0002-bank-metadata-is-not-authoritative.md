# ADR 0002: Bank-provided metadata is not authoritative

## Status

Accepted

## Context

The first implementation relied in part on bank-provided metadata, especially Kontist's `Umsatzkategorie`.

During development, we learned:

- the Kontist UI and exported CSV can diverge
- `Umsatzkategorie` may be missing in CSV exports even when the UI shows a category
- `Privat` is not a reliable signal for owner drawings or non-business transactions
- tax classification must not depend on opaque bank heuristics

## Decision

Treat bank-provided classification metadata only as an optional hint.

It may be used:

- for convenience
- for import assistance
- for review support

It must not be treated as the authoritative source for:

- tax classification
- bookkeeping classification
- owner drawings
- deductible VAT handling

The authoritative classification must come from:

- our own bookkeeping structure in `ledger/`
- our own mapping rules
- explicit reviewable logic in `tax/`

## Consequences

- the system must be bank-agnostic
- classification logic should move away from bank-specific fields
- future changes in bank export behavior should have limited architectural impact
