# AGENTS.md

## Scope

`hledger-elster/` is a Rust CLI that generates German tax exports for
[ELSTER.de](https://www.elster.de) from an `hledger` journal.

- `src/main.rs`: clap CLI entrypoint for the default `generate` command and `init-config`
- `src/paths.rs`: env-var/CLI path resolution (`FINANCES_LEDGER_JOURNAL`, `FINANCES_TAX_DATA_DIR`, `HLEDGER_ELSTER_CONFIG`)
- `src/config.rs`: `hledger-elster` TOML config loading/writing, Home-Office-Pauschale policy
- `src/posting.rs`: `TaxPosting`, the normalized enriched-posting model
- `src/dataset.rs`: `TaxDataset`, filter/group combinators over postings
- `src/journal.rs`: `hledger ... print --output-format json` shelling and JSON model
- `src/enrich.rs`: `elster_*` tag validation and enrichment, GWG/AfA override, drawing/contribution fallback
- `src/afa.rs`: straight-line depreciation
- `src/aggregates.rs`: net/gross/VAT aggregation over a `TaxDataset`
- `src/drawing.rs`: `is_drawing`/`is_contribution` predicates
- `src/classification.rs`: EÜR income/expense classification
- `src/periods.rs`: period-label generation and per-period aggregation
- `src/euer.rs` / `src/est.rs` / `src/ust.rs`: the three ELSTER form builders
- `src/herleitung.rs`: per-form audit-trail ("Herleitung") sheet builders
- `src/report_writer.rs`: xlsx/CSV writing, sheet-name/filename sanitization, deterministic zip output, export-hygiene tracking
- `specs/`: Markdown specifications; fenced `gherkin` blocks are compiled into cucumber features at build time by `build.rs` (see `tests/cucumber.rs` for step definitions)
- `examples/`: sanitized example journals
- `docs/`: ELSTER-specific design and migration notes

Private data and private tests stay outside this repository.

## Commands

- `just build`
- `just test`
- `cargo run -- -f <journal>`

## Boundaries

- Do not move private journals, private tax artifacts, or private verification tests into this directory.
- Keep ELSTER-facing output labels in German.
- Keep user-specific verification tests outside this repository.
- Keep workbook export structure consistent: `name.xlsx` has a sibling `name/` directory, and each workbook tab is also exported there as a CSV.

## Tag contract

The `elster_*` tag reference in `README.md` is the authoritative contract between
the journal author and the tool. Whenever a tag is added, removed, or its semantics
change, `README.md` must be updated in the same change. The README table is the
source of truth — the code is the implementation.

## Traceability invariant

Every numeric line in the main output forms (EÜR, ESt, USt) must be backed by a
corresponding Herleitung sheet that lists the individual transactions contributing
to that figure. A reader must be able to verify any form total by summing the rows
of the matching Herleitung sheet — no form value may be opaque.

Consequence: whenever a calculation is added or changed in `src/euer.rs`, `src/est.rs`,
or `src/ust.rs`, the matching sheet-builder in `src/herleitung.rs` must be updated in
the same change so that EÜR ↔ Herleitung, ESt ↔ Herleitung, and USt ↔ Herleitung remain
consistent. Entnahmen and Einlagen follow the same rule: both the EÜR totals and the
Herleitung sheets use the shared `is_drawing` / `is_contribution` predicates from
`src/drawing.rs` so they can never diverge.
