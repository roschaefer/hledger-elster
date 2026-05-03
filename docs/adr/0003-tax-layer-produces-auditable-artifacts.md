# ADR 0003: The tax layer produces auditable artifacts

## Status

Accepted

## Context

The tax workflow depends on outputs that are:

- human-readable
- versionable
- easy to compare with `git diff`
- suitable for communication during a tax audit (`Steuerprüfung`)

The first implementation showed that spreadsheets are useful, but CSV remains essential for deterministic inspection.

## Decision

`tax/` must continue to produce two synchronized artifact forms:

- a folder of CSV files
- an `.xlsx` workbook

These artifacts must represent the same values.

CSV is the primary diffable format.

Spreadsheet outputs exist for:

- manual inspection
- copy/paste into ELSTER workflows
- external communication

## Consequences

- CSV/XLSX parity must be preserved
- deterministic output matters
- spreadsheet presentation is important, but must not diverge from CSV values
