# Postings export schema

This document defines the intended interface from `ledger/` to `tax/`.

## Principle

`ledger/` should carry as much responsibility as possible.

That means:

- bookkeeping truth lives in `ledger/`
- `tax/` should not reconstruct bookkeeping from raw bank data
- `tax/` should consume a stable export that is already useful for tax calculation

## Export level

The intended interface is a **year-filtered posting export**, not raw bank CSVs and not balance-only output.

Why:

- raw bank CSVs contain source-specific quirks
- `hledger bal` is useful for review, but too aggregated for tax reporting
- posting-level data preserves enough structure for monthly and yearly tax views

## Canonical export

For each year, `ledger/` should provide a canonical CSV, for example:

```text
ledger/exports/2024/postings.csv
```

Each row represents one posting.

## Initial concrete implementation

The first implementation uses:

```bash
hledger register \
  -f hledger.journal \
  -b 2024-01-01 \
  -e 2025-01-01 \
  expenses income \
  --output-format csv

In the refactored repository layout, the resulting posting exports are written to
`ledger/exports/<year>/postings.csv`.
```

This is then normalized into a canonical CSV by `ledger/export_postings.py`.

## Initial exported columns

The current normalized export schema is:

- `transaction_id`
- `date`
- `code`
- `description`
- `transaction_comment`
- `posting_comment`
- `account`
- `amount`
- `currency`
- `running_total`
- `running_total_currency`
- `source_file`
- `source_line`

## Minimum columns needed by `tax/`

The initial target schema is:

- `date`
- `description`
- `account`
- `amount`
- `currency`
- `payee`
- `note`
- `tags`

Useful optional columns:

- `transaction_id`
- `status`
- `source_file`
- `source_account`

The current implementation already covers the essential columns needed for the first migration step.

## Responsibilities by layer

### `ledger/` owns

- CSV import
- bank-specific quirks
- account mapping
- transfers between own accounts
- business vs private bookkeeping classification
- tax payment bookkeeping classification
- unknown/unassigned bookkeeping items
- correct year/month attribution

### `tax/` owns

- ELSTER-specific report shapes
- annual manual values
- verification against previous tax filings
- spreadsheet and CSV artifact generation
- genuinely tax-specific adjustments

## Current practical migration rule

When possible, tax outputs should be derived directly from ledger account structure.

Examples already visible in the current ledger:

- business income is separated under `income:business:*`
- VAT advance payments are separated under `expenses:taxes:umsatzsteuer`
- many business expenses already live under `expenses:business:*`

This means the migration should prefer:

- using account structure directly
- fixing missing classification in `ledger/`

instead of rebuilding bookkeeping logic in `tax/`.

## Tax-specific adjustments that remain in `tax/`

The following are currently expected to remain outside ordinary bookkeeping and stay in the tax layer:

- `Home-Office-Pauschale`
- `AfA` for hardware
- partially deductible expenses such as mobile phone costs

## Important boundary: partial deductibility

Partially deductible expenses should remain modeled in `tax/`, not in `ledger/`.

Reason:

- the ledger should reflect the real-world booking
- the deductible share is tax logic, not bookkeeping truth

Example:

- a mobile phone payment is recorded in full in `ledger/`
- `tax/` applies the deductible share, such as 20%
- `tax/` also carries any fallback VAT knowledge required for this treatment

This includes:

- deductible net amount calculation
- deductible VAT amount calculation
- fallback VAT percentages where bookkeeping data does not fully encode them

## Consequence for the migration

When moving from V1 to the hledger-based design:

- remove bookkeeping reconstruction from `tax/`
- preserve tax-specific logic in `tax/`
- prefer fixing missing or wrong bookkeeping classification in `ledger/`
- only add logic to `tax/` when it is genuinely tax-specific

## Current implementation status

The current implementation already uses the canonical postings export for:

- `Einnahmenüberschussrechnung`
- `Umsatzsteuer`
- `Einkommensteuer`

The current VAT migration uses these ledger-derived inputs:

- `income:business:*` for gross business income, split into net revenue and VAT in `tax/`
- `expenses:taxes:umsatzsteuer` for VAT prepayment outflows
- mapped expense accounts for deductible input VAT

The current income-tax migration uses these ledger-derived inputs:

- `expenses:taxes:einkommensteuer` together with exported `transaction_comment`
  to distinguish `Einkommensteuervorauszahlung` from
  `Einkommensteuer-Abschlusszahlung`
- `expenses:health:insurance` and selected `expenses:personal:*` accounts
  for the insurance block
