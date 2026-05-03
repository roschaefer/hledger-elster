# ELSTER special cases

These cases are not fully represented by ordinary bookkeeping postings and therefore remain part of the tax layer.

## Manual annual values

- `Home-Office-Pauschale`

This is a yearly tax-specific value. It does not come from ordinary bank transactions and should remain a per-year manual input.

## Depreciation

- `AfA` for hardware such as a laptop

Depreciation may be represented in bookkeeping, but the tax layer must remain able to produce the ELSTER-oriented view and yearly verification of these values.

## Partially deductible expenses

- mobile phone expenses with a 20% deductible share

These require explicit tax treatment logic:

- deductible net amount
- deductible VAT amount
- correct mapping into ELSTER-facing outputs

## Rule of thumb

If a value is:

- not directly present in ordinary postings, or
- only partially derivable from bookkeeping,

it belongs in the tax layer, not in raw import handling.
