# Ledger Tax Modeling

## Purpose

This document describes how tax-relevant information should be modeled in `ledger/` so that `tax/` can stay small, auditable, and focused on ELSTER-oriented reporting.

It is a target model for the ongoing refactor.

## Core idea

The tax pipeline should start from ordinary ledger postings.

`ledger/` should answer:

- which real account paid or received money
- which business, private, tax, or transfer account the posting belongs to
- which cases are ordinary defaults
- which cases are explicit exceptions

`tax/` should answer:

- how ledger data is transformed into ELSTER-facing rows
- how tax-only values are added
- how special tax treatments are calculated
- how results are verified against prior tax filings

## Source dimensions

Each tax-relevant posting has two important dimensions:

- source asset account: where the money moved
- counter account: what the posting means

Example:

```hledger
2024-11-15 Hetzner
    assets:dkb:girokonto               EUR-47.00
    expenses:business:hosting:hetzner EUR47.00
```

This means:

- the source account is `assets:dkb:girokonto`
- the tax meaning comes from `expenses:business:hosting:hetzner`

The tax exports should first aggregate by source asset account and only later merge into ELSTER-oriented sheets.

## Asset accounts

Every real bank, card, wallet, or payment account should be modeled as an `assets:*` account.

Current examples:

- `assets:kontist:geschaeftskonto`
- `assets:dkb:girokonto`
- `assets:dkb:kreditkarte`
- `assets:dkb:tagesgeld`
- `assets:paypal`

These accounts are the basis for the per-account spreadsheet tabs and CSV files.

## Default account semantics

Some tax behavior should come from account naming conventions directly.

### Business expenses

Anything under:

- `expenses:business:*`

is considered deductible business spending by default.

This means the tax layer should not need a separate rule entry for every normal business expense account.

Examples:

- `expenses:business:hosting:wasabi`
- `expenses:business:hosting:hetzner`
- `expenses:business:software:ai:openai`
- `expenses:business:steuerberater`

### Tax payments

Anything under:

- `expenses:taxes:*`

is relevant for tax payment standard sums.

Examples:

- `expenses:taxes:umsatzsteuer:vorauszahlung`
- `expenses:taxes:umsatzsteuer:abschlusszahlung`
- `expenses:taxes:einkommensteuer:vorauszahlung`
- `expenses:taxes:einkommensteuer:abschlusszahlung`

The account name should be specific enough that `tax/` does not need to reconstruct the meaning from CSV text later.

### Transfers and withdrawals

Transfers to your own non-tax asset accounts should be modeled explicitly as transfer accounts, not as generic expenses.

Examples:

- `transfers:kontist-girokonto`
- future transfer accounts between DKB, PayPal, and other owned accounts

This allows `tax/` to compute per-account payout-style sums without relying on unreliable bank metadata.

## Preferred hierarchy for exception accounts

When an expense or insurance type is stable and meaningful on its own, prefer a clear account tree over a separate tax label.

Examples:

- `expenses:phone:service`
- `expenses:phone:hardware`
- `expenses:insurance:health:aok`
- `expenses:insurance:travel:ukv`
- `expenses:insurance:travel:hansemerkur`
- `expenses:insurance:liability:haftpflicht`

This avoids maintaining a second naming system such as:

- account name in `ledger/`
- different display label in `tax/`

If the account name is already good enough to appear in the spreadsheet, it should usually be used directly.

## When to use account tags

Use hledger account tags when the account name alone is not enough or when an account carries stable metadata that should be inherited by postings.

All ELSTER contract tags should use the `elster_` prefix.

Good tag use cases:

- deduction mode
- VAT rate
- proportional shares
- tax form placement
- stable classification hints that apply to the whole account

Examples:

```hledger
account expenses:phone  ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.20, elster_vat_share:0.20, elster_vat_rate:0.19

account expenses:insurance:travel:hansemerkur  ; elster_form:einkommensteuer
```

This is preferable when the account is an exception to the default rules.

## When not to use tags

Do not rely on chart-of-accounts names as a tool contract. Use tags for all ELSTER semantics.

Examples:

- duplicating an output item that is already supplied by `elster_item`
- restating a bank identifier that already lives in a non-ELSTER tag
- storing ad-hoc transaction notes that do not affect ELSTER output

If the information affects ELSTER behavior, prefer an explicit `elster_*` tag over an account-name convention.

## Transaction tags vs account tags

Prefer account tags for stable rules that apply to all postings in an account.

Use transaction or posting tags only when a single posting is exceptional.

Examples:

- good account-tag case: all `expenses:phone` postings use the same 20% deduction logic
- good posting-tag case: one posting in an otherwise ordinary account needs a one-off override

The default should be:

- chart of accounts first
- account tags second
- posting tags only for true exceptions

## Spreadsheet flow

The reporting flow should be:

1. Read ledger postings.
2. Select tax-relevant postings based on account namespaces and explicit exceptions.
3. Build one tax spreadsheet section per `assets:*` account that touched tax-relevant postings.
4. Compute standard sums and category rows inside each account section.
5. Merge those account sections into the ELSTER-oriented sheets:
   - `Einnahmenüberschussrechnung`
   - `Umsatzsteuer`
   - `Einkommensteuer`
6. Add tax-only manual values at the very end.

This ordering matters.

Manual values such as `Home-Office-Pauschale` should not influence the understanding of ledger-derived account sums. They are a final tax-layer adjustment.

## What should remain in `tax/`

Even with stronger ledger modeling, some logic remains tax-specific:

- ELSTER sheet structure and output formatting
- yearly manual values like `Home-Office-Pauschale`
- depreciation and AfA logic
- partial deductibility calculations
- verification against `abgleich.yml`

In other words:

- `ledger/` should classify and structure
- `tax/` should interpret and report

## Legacy migration from `aggregation.yml` and `anlagen.yml`

The old `aggregation.yml` and `anlagen.yml` split the legacy tax model across two files and mixed multiple concerns:

- account matching
- tax classification
- tax computation rules

Target state:

- ordinary defaults come from account namespaces
- exception metadata moves to hledger account tags
- only genuinely tax-only configuration remains in `tax/`

Likely examples that can move out of `aggregation.yml` / `anlagen.yml`:

- proportional deduction metadata for account families like `expenses:phone:*`
- insurance grouping that is already obvious from account names
- business-vs-private classification that is already encoded in the chart of accounts

Likely examples that should stay in `tax/`:

- annual manual values
- ELSTER row grouping/order
- validation rules

### Hardware / AfA

The current spreadsheet places `expenses:hardware:computer` in `Einnahmenüberschussrechnung`.

That means the hardware/AfA tagging should be treated carefully:

- the asset should carry `tax_` metadata in the ledger
- the resulting depreciation row should stay in EÜR unless the tax model proves otherwise
- do not assume `tax_form:einkommensteuer` for hardware without re-checking the current export behavior

## Example modeling patterns

### Ordinary business expense

```hledger
2024-07-10 Wasabi
    assets:paypal                        EUR-8.01
    expenses:business:hosting:wasabi     EUR8.01
```

No extra tag is required if the default rule for `expenses:business:*` is sufficient.

### Special partial deduction

```hledger
account expenses:phone  ; tax_form:einnahmenueberschussrechnung, tax_deduction:proportional, tax_expense_share:0.20, tax_vat_share:0.20, tax_vat_rate:0.19

2024-03-15 klarmobil GmbH
    assets:dkb:girokonto  EUR-11.90
    expenses:phone:service  EUR11.90
```

```hledger
2024-12-11 SATURN ONLINE
    assets:kontist:geschaeftskonto  EUR-499.00
    expenses:phone:hardware         EUR499.00
```

### Tax payment with semantic account name

```hledger
2024-12-12 STEUERVERWALTUNG NRW  ; Einkommensteuer 2022
    assets:kontist:geschaeftskonto            EUR-558.41
    expenses:taxes:einkommensteuer:abschlusszahlung  EUR558.41
```

### Insurance relevant for income tax

```hledger
account expenses:insurance:health:aok  ; elster_form:einkommensteuer

2024-01-15 AOK Rheinland / Hamburg
    assets:dkb:girokonto             EUR-300.00
    expenses:insurance:health:aok    EUR300.00
```

## Validation expectations

If more tax semantics move into ledger, validation becomes more important.

The system should eventually validate things like:

- tax-relevant exception accounts have required `tax_` tags
- tags use an allowed vocabulary
- `expenses:taxes:*` accounts are specific enough
- ordinary `expenses:business:*` accounts do not need redundant exception config

Without validation, tags become too easy to misspell and the model becomes fragile.
