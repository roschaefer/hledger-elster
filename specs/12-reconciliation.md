# Reconciliation library

`hledger_elster::reconciliation` lets you pin specific cells of a generated
ELSTER export to their previously verified values, so `cargo test` fails
loudly the moment one of them drifts from what was actually filed --
something nothing else in this repository's own `cargo test` run checks,
since that only proves the *current* ledger produces a *self-consistent*
export, not that it still matches history.

## Why this exists

Particularly useful when migrating from spreadsheets: pin `Case`s to the
figures you already expect from the exported data, and let failures tell
you where the export -- or the spreadsheet you trusted until now --
disagrees.

Everything else -- `Case`/`SumCase`, `export_dir()`, the stale-manifest
guard, what a drift means and how to resolve it (`Status::ConfirmedDrift`,
`Status::UnderReview`), and a fully worked, verified example for each of
those -- lives in the `reconciliation` module's own documentation rather
than here:

```sh
cargo doc --open -p hledger-elster  # then open the `reconciliation` module
```

or read [`src/reconciliation.rs`](../src/reconciliation.rs) directly.
Every example there is a real doctest, checked by `cargo test --doc` on
every run, so it can't drift from what the library actually does.
