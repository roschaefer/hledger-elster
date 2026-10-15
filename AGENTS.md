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
- `src/csv_import.rs`: reads the CSV files this tool writes back into `ReportRow`/`TrailSheet` — the read-back half of the [CSV/xlsx equivalence invariant](./tests/specifications/csv-xlsx-equivalence/)
- `src/report_writer.rs`: xlsx/CSV writing, sheet-name/filename sanitization, deterministic zip output, export-hygiene tracking
- `tests/specifications/`: executable Scrut specifications; each chapter
  directory uses `README.md` for its document, with scenario inputs written
  inline by the helpers in `readme-setup.sh`; a chapter whose scenarios share
  a starting state keeps it in its `setup.bash` and splits further scenarios
  into sibling documents
- `examples/`: sanitized example journals; `README.md` is a Scrut document
  that runs its examples against them

Private data and private tests stay outside this repository.

## Commands

- `just build`
- `just test`
- `just test-scrut`
- `cargo run -- -f <journal>`

## Boundaries

- Do not move private journals, private tax artifacts, or private verification tests into this directory.
- Keep ELSTER-facing output labels in German.
- Keep user-specific verification tests outside this repository.
- Keep workbook export structure consistent: `name.xlsx` has a sibling `name/` directory, and each workbook tab is also exported there as a CSV.
