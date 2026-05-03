# Mapping V1 to hledger

This document tracks how concepts from the first implementation should move into the new architecture.

## Likely moves into `ledger/`

- bank-specific CSV import handling
- account-level aggregation
- transfer handling
- multi-account support
- bookkeeping categorization

## Likely stays in `tax/`

- ELSTER-oriented outputs
- `Home-Office-Pauschale`
- `AfA`
- partially deductible tax logic
- fallback VAT knowledge for tax-only partial-deductibility rules
- `abgleich.yml`
- spreadsheet artifact generation

## Current migration status

The interface from `ledger/` to `tax/` is now defined as:

- exported posting-level CSV data under `ledger/exports/<year>/postings.csv`

instead of direct parsing of journal files from `tax/`.

The first report slices already migrated to that interface are:

- `Einnahmenüberschussrechnung`
- `Umsatzsteuer`
- `Einkommensteuer`

Verification/report cleanup still remain on the older path for now.
