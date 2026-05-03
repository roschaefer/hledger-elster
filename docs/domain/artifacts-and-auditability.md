# Artifacts and auditability

The tax layer must always produce artifacts that are easy to inspect and compare.

## Required artifact forms

- CSV folder
- `.xlsx` workbook

## Invariants

- all artifact forms must represent the same values
- CSV remains the easiest source for `git diff`
- spreadsheets must remain suitable for manual review and tax audit workflows

## Why CSV matters

CSV is the fastest way to see exactly which values changed between runs.

This is especially important when:

- refactoring tax logic
- validating yearly calculations
- comparing current results with previous tax filings

## Why spreadsheets still matter

Spreadsheet outputs are still required because they are practical for:

- ELSTER preparation
- visual inspection
- sharing during a tax audit
