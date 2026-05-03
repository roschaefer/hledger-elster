# Target architecture

## Goal

Split responsibilities cleanly:

- `ledger/` handles bookkeeping
- `tax/` handles tax reporting

## Desired boundary

`ledger/`:

- imports bank data
- stores journals
- models accounts, postings, transfers, balances
- exports stable machine-readable data

`tax/`:

- consumes stable exports from `ledger/`
- maps postings to ELSTER-oriented outputs
- adds tax-only logic
- generates ELSTER-oriented outputs
- verifies results against prior tax filings

## Intended result

`tax/` should eventually stop depending on raw bank CSV imports.

## Modeling note

The intended ledger-side modeling for tax-relevant postings, account names, and account tags is documented in:

- [../domain/ledger-tax-modeling.md](../domain/ledger-tax-modeling.md)
