# ADR 0004: Tax layer rewrite

## Status

Accepted

## Context

The existing tax layer accumulated several structural problems over time:

- The `Transaction` model carried legacy bank-CSV fields (`quelle_bank`, `konto_name`,
  `konto_typ`, `vorstatus`, `raw`) from before hledger became the source of truth.
  Code in `account_tags.py` synthesized fake `Transaction` objects with hardcoded
  strings to feed this model.

- Classifications and tax treatments were stored in `dict[int, ...]` keyed by Python
  object identity (`id(transaction)`). Any code path that rebuilt or reordered
  transactions invalidated these dicts silently. This was the root cause of unrelated
  spreadsheet tabs affecting each other when new features were added.

- AfA depreciation was injected as synthetic `Transaction` objects into the main
  transaction list, making the bookkeeping view and the tax computation view
  indistinguishable.

- Verification was encoded in `abgleich.yml` files with a custom parser and status
  vocabulary. The parser, the status enum, and the reporting logic were bespoke
  infrastructure that could be replaced by standard tooling.

- Code and generated artifact labels mixed English and German without a clear rule.

## Decisions

### 1. Replace `Transaction` with `TaxPosting`

The new domain model is `TaxPosting`. It contains only fields derivable from hledger
postings and account directives:

- bookkeeping dimensions: `posting_date`, `source_account`, `counter_account`, `amount`,
  `description`, `transaction_comment`, `posting_comment`
- tax enrichment resolved at ingest time: `tax_form`, `tax_deduction`, `vat_rate`,
  `expense_share`, `vat_share`
- provenance: `derived_kind`, `source_file`, `source_line`

No legacy bank-CSV fields. No `raw` dict. No `id()`-keyed classification dicts.

### 2. Queryable dataset

`TaxDataset` wraps a list of `TaxPosting` and exposes filter and group methods:
`for_form()`, `for_account_prefix()`, `for_source_account()`, `for_year()`,
`for_quarter()`, `group_by_counter_account()`, and so on.

All tax enrichment is embedded in each `TaxPosting` at load time. Reports query the
dataset; they do not classify or tag postings.

### 3. Reports are isolated functions

Each report (`euer.py`, `ust.py`, `est.py`, `per_account.py`) is a function that
receives the full `TaxDataset` and returns a `WorksheetSpec`. Reports call pure
functions from `calculate/` to aggregate values.

No shared mutable state exists between reports. Changing the shape of one report
cannot affect another.

### 4. AfA is computed at report time only

`TaxPosting` stores the original purchase posting with `tax_deduction="afa"` and
the depreciation parameters (`tax_afa_years`, `tax_vat_rate`).

`calculate/afa.py` is a pure function: `(TaxPosting, report_year) → Decimal`.

The EÜR report calls this function at report time to derive the depreciation amount
for Line 27. The `TaxDataset` always reflects the bookkeeping reality — the initial
purchase — and contains no synthetic depreciation rows.

Per-account bookkeeping views show the original payment. ELSTER-facing views show the
depreciation schedule. These are two different projections of the same source data.

### 5. Verification uses pytest

`abgleich.yml` is replaced by a pytest test suite under `tests/`.

Tests are grouped by ELSTER form and section. The four verification states are:

- `expected`: plain assertion
- `approximate`: `pytest.approx` with an explicit tolerance
- `confirmed`: plain assertion on the accepted new value, with a comment documenting
  the prior filed value and the reason for the difference
- `under_review`: `@pytest.mark.needs_review(reason)` with an assertion on the
  currently observed deviation value — does not fail the build, appears in a summary
  section, fails hard if the value drifts further

See `docs/domain/verification.md` for full detail.

### 6. Naming convention

All source code, field names, function names, and internal identifiers use English,
following hledger conventions.

German is permitted only in generated output artifacts (spreadsheet column headers,
CSV file names, ELSTER-facing labels). The three output reports are named after their
ELSTER forms: Einnahmenüberschussrechnung, Umsatzsteuer, Einkommensteuer.

### 7. Source account list moves to the tax layer

`EXCLUDED_ASSET_PREFIXES` in `ledger/export_postings.py` is replaced by
`ASSET_ACCOUNTS` in `tax/src/ingest/asset_accounts.py`. The ledger export script
becomes a dumb extract tool. The tax layer owns the decision about which accounts
are source asset accounts.

## Consequences

- The `Transaction` dataclass and all `id()`-keyed classification dicts are deleted.
- `aggregation.yml` and `anlagen.yml` (or equivalent YAML config) are replaced by
  account tags in `ledger/accounts.journal` for ordinary defaults, and by pure Python
  in report functions for ELSTER-specific grouping.
- `abgleich.yml` files are superseded by test files. Historical `abgleich.yml` values
  are migrated to test assertions during the rewrite.
- Historical ELSTER submission PDFs are private reference material for
  ELSTER line-number-to-account mappings and should be consulted when adding or
  changing report output.
