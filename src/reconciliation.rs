//! Helpers for writing your own regression ("Abgleich") tests against a
//! generated ELSTER export: assert that a specific figure in the export CSVs
//! still matches a previously verified value, so a refactor, a config
//! change, or a late-arriving correction to last year's ledger that silently
//! shifts a number gets caught instead of silently re-filed.
//!
//! A `Case` failing doesn't by itself say *what* is wrong -- a drift between
//! the pinned `expected` value and what the export actually produced means
//! one of four different things, and each has a different resolution:
//!
//! 1. **The historical (filed) figure was wrong; the export is now correct.**
//!    An old bug -- in the ledger, in a tag, in the tool -- produced the
//!    number that actually got filed. Once you've confirmed the export is
//!    now right, update `expected` to that value, move `.previous(...)` to
//!    the old (wrong) figure, set `.status(Status::ConfirmedDrift)`, and
//!    `.reason(...)` to explain the old bug. The case still asserts exact
//!    equality (against the corrected `expected`) -- `previous`/`reason` just
//!    keep the old figure and its explanation visible in the test source for
//!    whoever reads it next.
//! 2. **The export is wrong; the historical (filed) figure was correct.**
//!    That's a hledger-elster bug. Please open one at
//!    <https://github.com/roschaefer/hledger-elster/issues> rather than
//!    changing `expected` to match -- a mismatched `Case` panic says as much.
//! 3. **The drift is real, reviewed, and permanent.** Sometimes a bug in the
//!    historical data can't be fixed anymore -- it's already been filed with
//!    ELSTER, and amending it isn't worth it (or isn't possible). This is the
//!    same resolution as (1): `.status(Status::ConfirmedDrift)`, `expected`
//!    pinned to what the export correctly produces, `.previous(...)` holding
//!    what was actually filed, `.reason(...)` explaining why nothing more
//!    will be done about it.
//! 4. **You don't know yet which of the above it is, and can't find out right
//!    now.** Maybe there's no time to dig into it this week; maybe the
//!    discrepancy looks like a hledger-elster bug and you're waiting on a
//!    <https://github.com/roschaefer/hledger-elster/issues> report to be
//!    triaged. Either way, set `.status(Status::UnderReview)`, pin `expected`
//!    to what the export produces *right now*, and `.reason(...)` to explain
//!    why review is postponed. The case still asserts exact equality --
//!    exactly like `ConfirmedDrift` -- so if the ledger changes again and
//!    produces a *further* drift, the case fails again rather than silently
//!    accepting an unrelated second discrepancy under the same postponed
//!    reason.
//!
//! Every `UnderReview` case prints a `"UNDER REVIEW: ..."` line when it runs,
//! even if it passes -- run `cargo test -- --show-output` (libtest normally
//! suppresses output from passing tests) and grep for that marker to list
//! every case still awaiting review.
//!
//! Typical usage in a downstream crate that keeps its own historical ledger
//! is a `#[test]` fn defined with [`case_test!`](crate::case_test):
//!
//! ```
//! use hledger_elster::{case_test, reconciliation::Case};
//! use rust_decimal_macros::dec;
//!
//! case_test!(
//!     krankenversicherung_kv,
//!     Case::new(
//!         "einkommensteuer/final/Expenses:Insurance:Health:AOK:KV",
//!         "2024/steuererklaerung/einkommensteuer.csv",
//!         "Kennzahl",
//!         "Krankenversicherung",
//!         "2024",
//!         dec!(1236.98),
//!     )
//! );
//! ```
//!
//! `case_test!` just wraps [`assert_csv_value`] in a `#[test]` fn, so that
//! call is what's actually verified in the worked examples below (a
//! `#[test]` fn defined by a macro invocation, as above, can't be *executed*
//! from inside a doctest, only compiled -- doctests run their body directly,
//! not through libtest's test collection). Every example shares this
//! journal:
//!
//! ```text
//! account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
//! account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
//!
//! 2024-01-15 Client invoice
//!     income:business       -119.00 EUR
//!     assets:bank:business   119.00 EUR
//! ```
//!
//! which produces a `Umsatzsteuerpflichtige Betriebseinnahmen` of `100.00`
//! -- the 19% VAT split out of the 119.00 gross invoice -- in
//! `2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv`.
//!
//! ### An unreviewed drift fails
//!
//! A `Case` pinned to the wrong figure panics, `assert_eq!`-style, with a
//! pointer to whichever of scenarios (1)-(4) above applies:
//!
//! ```
//! # use hledger_elster::reconciliation::{assert_csv_value, export_dir, Case};
//! # use rust_decimal_macros::dec;
//! # let journal = "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2024-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -119.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   119.00 EUR\n\
//! # ";
//! # let tmp = tempfile::tempdir().unwrap();
//! # let journal_path = tmp.path().join("journal.journal");
//! # std::fs::write(&journal_path, journal).unwrap();
//! # std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal_path);
//! # std::env::set_var("FINANCES_TAX_DATA_DIR", tmp.path().join("export"));
//! # hledger_elster::report_writer::generate_report().unwrap();
//! let result = std::panic::catch_unwind(|| {
//!     assert_csv_value(
//!         &export_dir(),
//!         &Case::new(
//!             "income:business",
//!             "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
//!             "Kennzahl",
//!             "Umsatzsteuerpflichtige Betriebseinnahmen",
//!             "2024",
//!             dec!(999.00), // pinned to the wrong figure on purpose
//!         ),
//!     );
//! });
//! let message = *result.unwrap_err().downcast::<String>().unwrap();
//! assert!(message.contains("expected=999.00, actual=100.00, delta=-899.00"));
//! assert!(message.contains("https://github.com/roschaefer/hledger-elster/issues"));
//! ```
//!
//! The panic this produces reads:
//!
//! ```text
//! assertion `left == right` failed: income:business: value mismatch, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=Expected, expected=999.00, actual=100.00, delta=-899.00
//!
//! If the historical (filed) figure was wrong and the export above is now correct, see scenario (1)/(3) in the `reconciliation` module docs: mark this case `.status(Status::ConfirmedDrift)` with `.previous(...)`/`.reason(...)` explaining why. If the export is wrong and the historical figure was correct, that's a hledger-elster bug -- please open one at https://github.com/roschaefer/hledger-elster/issues
//!   left: 100.00
//!  right: 999.00
//! ```
//!
//! ### Fixing the ledger turns it green
//!
//! Pin `expected` to what the ledger *should* produce, and the same `Case`
//! passes once the export agrees -- no change to the test itself:
//!
//! ```
//! # use hledger_elster::reconciliation::{assert_csv_value, export_dir, Case};
//! # use rust_decimal_macros::dec;
//! # let journal = "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2024-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -119.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   119.00 EUR\n\
//! # ";
//! # let tmp = tempfile::tempdir().unwrap();
//! # let journal_path = tmp.path().join("journal.journal");
//! # std::fs::write(&journal_path, journal).unwrap();
//! # std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal_path);
//! # std::env::set_var("FINANCES_TAX_DATA_DIR", tmp.path().join("export"));
//! # hledger_elster::report_writer::generate_report().unwrap();
//! assert_csv_value(
//!     &export_dir(),
//!     &Case::new(
//!         "income:business",
//!         "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
//!         "Kennzahl",
//!         "Umsatzsteuerpflichtige Betriebseinnahmen",
//!         "2024",
//!         dec!(100.00),
//!     ),
//! );
//! ```
//!
//! ### The historical figure was wrong (`Status::ConfirmedDrift`)
//!
//! Say `90.00` was actually filed for 2024, but reviewing it turns up an old
//! bug -- the correct figure was always `100.00`. Pin `expected` to the
//! corrected figure, move the old one to `.previous(...)`, and explain why
//! in `.reason(...)`; the case still asserts exact equality against
//! `expected`, it just also keeps the old figure and the explanation
//! visible in the test source:
//!
//! ```
//! # use hledger_elster::reconciliation::{assert_csv_value, export_dir, Case, Status};
//! # use rust_decimal_macros::dec;
//! # let journal = "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2024-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -119.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   119.00 EUR\n\
//! # ";
//! # let tmp = tempfile::tempdir().unwrap();
//! # let journal_path = tmp.path().join("journal.journal");
//! # std::fs::write(&journal_path, journal).unwrap();
//! # std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal_path);
//! # std::env::set_var("FINANCES_TAX_DATA_DIR", tmp.path().join("export"));
//! # hledger_elster::report_writer::generate_report().unwrap();
//! assert_csv_value(
//!     &export_dir(),
//!     &Case::new(
//!         "income:business",
//!         "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
//!         "Kennzahl",
//!         "Umsatzsteuerpflichtige Betriebseinnahmen",
//!         "2024",
//!         dec!(100.00),
//!     )
//!     .status(Status::ConfirmedDrift)
//!     .previous(dec!(90.00))
//!     .reason("Historical figure missed a client invoice; corrected once the ledger was fixed"),
//! );
//! ```
//!
//! ### Postponing a review (`Status::UnderReview`)
//!
//! No time to dig into a drift this week, or it looks like a hledger-elster
//! bug awaiting triage? Pin `expected` to what the export produces *right
//! now* and explain why in `.reason(...)`. The case passes, behaving exactly
//! like `ConfirmedDrift` -- so a *further* drift still fails -- except it
//! also prints a marker every time it runs, even when it passes:
//!
//! ```
//! # use hledger_elster::reconciliation::{assert_csv_value, export_dir, Case, Status};
//! # use rust_decimal_macros::dec;
//! # let journal = "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2024-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -119.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   119.00 EUR\n\
//! # ";
//! # let tmp = tempfile::tempdir().unwrap();
//! # let journal_path = tmp.path().join("journal.journal");
//! # std::fs::write(&journal_path, journal).unwrap();
//! # std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal_path);
//! # std::env::set_var("FINANCES_TAX_DATA_DIR", tmp.path().join("export"));
//! # hledger_elster::report_writer::generate_report().unwrap();
//! assert_csv_value(
//!     &export_dir(),
//!     &Case::new(
//!         "income:business",
//!         "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
//!         "Kennzahl",
//!         "Umsatzsteuerpflichtige Betriebseinnahmen",
//!         "2024",
//!         dec!(100.00),
//!     )
//!     .status(Status::UnderReview)
//!     .reason("No time to review this week"),
//! );
//! ```
//!
//! `cargo test -- --show-output` (libtest normally suppresses output from
//! passing tests) prints, for the case above:
//!
//! ```text
//! UNDER REVIEW: income:business: under review, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=UnderReview, expected=100.00, actual=100.00, delta=+0.00, reason=No time to review this week
//! ```
//!
//! Grepping a full test run for `UNDER REVIEW:` lists every case still
//! awaiting review (verified by this crate's own test suite, since a
//! doctest has no way to capture its own stdout).
//!
//! ### A removed year's stale CSV is rejected
//!
//! Every `hledger elster` run writes a manifest of the files it (re)wrote.
//! If last year's journal entries are removed and the export is regenerated,
//! the old CSV for that year may still be sitting on disk -- but it's gone
//! from the manifest, and a `Case` reading it refuses rather than silently
//! trusting stale data:
//!
//! ```
//! # use hledger_elster::reconciliation::{assert_csv_value, export_dir, Case};
//! # use rust_decimal_macros::dec;
//! # let tmp = tempfile::tempdir().unwrap();
//! # let journal_path = tmp.path().join("journal.journal");
//! # let export_dir_path = tmp.path().join("export");
//! # std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal_path);
//! # std::env::set_var("FINANCES_TAX_DATA_DIR", &export_dir_path);
//! #
//! # // First run: a 2024 entry produces the 2024 CSV.
//! # std::fs::write(&journal_path, "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2024-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -119.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   119.00 EUR\n\
//! # ").unwrap();
//! # hledger_elster::report_writer::generate_report().unwrap();
//! #
//! # // Second run: the journal now only covers 2023 -- the 2024 CSV file is
//! # // still on disk, but the fresh manifest no longer lists it.
//! # std::fs::write(&journal_path, "\
//! # account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto\n\
//! # account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen\n\
//! # \n\
//! # 2023-01-15 Client invoice\n\
//! # \x20\x20\x20\x20income:business       -50.00 EUR\n\
//! # \x20\x20\x20\x20assets:bank:business   50.00 EUR\n\
//! # ").unwrap();
//! # hledger_elster::report_writer::generate_report().unwrap();
//! let result = std::panic::catch_unwind(|| {
//!     assert_csv_value(
//!         &export_dir(),
//!         &Case::new(
//!             "income:business",
//!             "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
//!             "Kennzahl",
//!             "Umsatzsteuerpflichtige Betriebseinnahmen",
//!             "2024",
//!             dec!(100.00),
//!         ),
//!     );
//! });
//! let message = *result.unwrap_err().downcast::<String>().unwrap();
//! assert!(message.contains("was not (re)generated by the most recent `hledger elster` run"));
//! ```
//!
//! `export_dir()` resolves the export directory the same way the main tool
//! resolves where to *write* it: `FINANCES_TAX_DATA_DIR`, defaulting to
//! `./data/exports`. Point both at the same value (a Justfile that runs
//! `hledger elster` and then `cargo test` needs no further wiring) and every
//! case reads back the export the tool just produced.

use crate::csv_import::read_report_rows;
use crate::paths;
use crate::periods::{self, ReportRow};
use rust_decimal::Decimal;
use std::path::{Path, PathBuf};

/// Resolves the export directory the same way `hledger elster` resolves
/// where to write it: `FINANCES_TAX_DATA_DIR`, defaulting to
/// `./data/exports`. Panics with a pointer to that env var if nothing has
/// been exported there yet.
pub fn export_dir() -> PathBuf {
    let export_root = paths::tax_data_dir();
    if !export_root.exists() {
        panic!(
            "Tax export directory does not exist: {}. Run `hledger elster` first, or point FINANCES_TAX_DATA_DIR at an existing export.",
            export_root.display()
        );
    }
    export_root
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// The figure is expected to match `expected` exactly. If it doesn't,
    /// see the module docs for how to tell a historical-data bug (1) from a
    /// hledger-elster bug (2).
    #[default]
    Expected,
    /// The figure is allowed to drift from `expected` by up to `tolerance`
    /// (required -- and only meaningful for this status).
    Tolerated,
    /// Scenario (1) or (3) from the module docs: the export used to match
    /// `previous` but was knowingly corrected to `expected`, or `previous` is
    /// what was actually filed and can no longer be changed. Either way,
    /// `previous` and `reason` (both required -- `reason` must be non-blank,
    /// and `previous` must differ from `expected`) record what it used to be
    /// and why they differ, and the case still asserts exact equality
    /// against `expected`.
    ConfirmedDrift,
    /// Scenario (4) from the module docs: review is postponed (no time yet,
    /// or a suspected hledger-elster bug awaiting triage). `expected` is
    /// pinned to whatever the export produced when review was postponed, and
    /// `reason` (required, non-blank) records why. Behaves exactly like
    /// `ConfirmedDrift` -- the case still asserts exact equality, so
    /// a *further* drift fails again -- except every run also prints a
    /// `"UNDER REVIEW: ..."` line (visible with `cargo test -- --show-output`)
    /// so these cases can be listed and revisited.
    UnderReview,
}

/// One reconciliation assertion against a single CSV cell: find the row
/// where `key_field == key_value`, read `value_field` from that row, and
/// compare it to `expected` under `status`.
#[derive(Debug, Clone)]
pub struct Case {
    pub id: &'static str,
    pub path: &'static str,
    pub key_field: &'static str,
    pub key_value: &'static str,
    pub value_field: &'static str,
    pub expected: Decimal,
    pub status: Status,
    pub tolerance: Option<Decimal>,
    pub previous: Option<Decimal>,
    pub reason: Option<&'static str>,
}

impl Case {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &'static str,
        path: &'static str,
        key_field: &'static str,
        key_value: &'static str,
        value_field: &'static str,
        expected: Decimal,
    ) -> Self {
        Self {
            id,
            path,
            key_field,
            key_value,
            value_field,
            expected,
            status: Status::Expected,
            tolerance: None,
            previous: None,
            reason: None,
        }
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn tolerance(mut self, tolerance: Decimal) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    pub fn previous(mut self, previous: Decimal) -> Self {
        self.previous = Some(previous);
        self
    }

    pub fn reason(mut self, reason: &'static str) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// Like `Case`, but `expected` is compared against the sum of `value_field`
/// across every row matching one of `key_values` -- for figures that are
/// only meaningful once several rows (e.g. several bank sub-accounts) are
/// added together.
#[derive(Debug, Clone)]
pub struct SumCase {
    pub id: &'static str,
    pub path: &'static str,
    pub key_field: &'static str,
    pub key_values: &'static [&'static str],
    pub value_field: &'static str,
    pub expected: Decimal,
    pub status: Status,
    pub tolerance: Option<Decimal>,
    pub previous: Option<Decimal>,
    pub reason: Option<&'static str>,
}

impl SumCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &'static str,
        path: &'static str,
        key_field: &'static str,
        key_values: &'static [&'static str],
        value_field: &'static str,
        expected: Decimal,
    ) -> Self {
        Self {
            id,
            path,
            key_field,
            key_values,
            value_field,
            expected,
            status: Status::Expected,
            tolerance: None,
            previous: None,
            reason: None,
        }
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn tolerance(mut self, tolerance: Decimal) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    pub fn previous(mut self, previous: Decimal) -> Self {
        self.previous = Some(previous);
        self
    }

    pub fn reason(mut self, reason: &'static str) -> Self {
        self.reason = Some(reason);
        self
    }
}

fn parse_cell(row: &ReportRow, path: &str, value_field: &str) -> Decimal {
    let cell = row
        .get(value_field)
        .unwrap_or_else(|| panic!("Missing column {value_field:?} in {path}"));
    periods::parse(cell)
        .unwrap_or_else(|err| panic!("failed to parse {cell:?} as Decimal in {path}: {err}"))
}

fn matching_rows<'a>(
    rows: &'a [ReportRow],
    key_field: &str,
    key_value: &str,
) -> Vec<&'a ReportRow> {
    rows.iter()
        .filter(|row| row.get(key_field).map(String::as_str) == Some(key_value))
        .collect()
}

/// Panics unless `path` is listed in `export_root`'s manifest -- i.e. unless
/// it was actually (re)written by the most recent `hledger elster` run. Guards
/// against a year or form that's been removed from the journal but whose old
/// CSV is still sitting in the export directory: without this, a `Case`
/// reading that path would silently compare against stale data instead of
/// the (nonexistent) current export.
fn verify_fresh(export_root: &Path, path: &str) {
    let manifest_path = export_root.join(paths::MANIFEST_FILE_NAME);
    let manifest = std::fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
        panic!(
            "Failed to read {} ({err}) -- this export was not produced by `hledger elster`, \
             or predates its freshness check. Run `hledger elster` to (re)generate it.",
            manifest_path.display()
        )
    });
    if !manifest.lines().any(|line| line == path) {
        panic!(
            "{path} was not (re)generated by the most recent `hledger elster` run in:\n  {}\n\n\
             This is stale data left over from an earlier export (e.g. a year or form that no \
             longer appears in the journal). Re-run `hledger elster` and check your journal \
             includes cover this year/form before trusting this figure.",
            export_root.display()
        );
    }
}

/// Reads `export_root/path`, finds the row where `key_field == key_value`,
/// and parses `value_field` from that row as a `Decimal`. Panics if no row
/// matches, or if more than one does -- a `Case` targets a single row by
/// definition, so more than one match is ambiguous rather than something to
/// silently sum or pick the first of; use a `SumCase` if you mean to add
/// multiple matching rows together. Amounts in the export are
/// German-formatted (`,` decimal, per `periods::fmt`); this parses them back
/// with `periods::parse`.
pub fn read_csv_value(
    export_root: &Path,
    path: &str,
    key_field: &str,
    key_value: &str,
    value_field: &str,
) -> Decimal {
    verify_fresh(export_root, path);
    let full_path = export_root.join(path);
    let rows = read_report_rows(&full_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", full_path.display()));
    match matching_rows(&rows, key_field, key_value).as_slice() {
        [] => panic!("Missing row {key_value:?} in {path} using key {key_field:?}"),
        [row] => parse_cell(row, path, value_field),
        matches => panic!(
            "{} rows match {key_value:?} in {path} using key {key_field:?} -- expected exactly \
             one; use a SumCase if you mean to add them together",
            matches.len()
        ),
    }
}

/// Like `read_csv_value`, but sums `value_field` across *every* row matching
/// `key_field == key_value` instead of requiring exactly one -- used by
/// `assert_csv_sum` so a `SumCase` key that matches several rows (e.g. two
/// sections that both used the same `elster_item`) contributes all of them,
/// not just the first.
fn sum_csv_value(
    export_root: &Path,
    path: &str,
    key_field: &str,
    key_value: &str,
    value_field: &str,
) -> Decimal {
    verify_fresh(export_root, path);
    let full_path = export_root.join(path);
    let rows = read_report_rows(&full_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", full_path.display()));
    let matches = matching_rows(&rows, key_field, key_value);
    if matches.is_empty() {
        panic!("Missing row {key_value:?} in {path} using key {key_field:?}");
    }
    matches
        .into_iter()
        .map(|row| parse_cell(row, path, value_field))
        .sum()
}

#[allow(clippy::too_many_arguments)]
fn check(
    id: &str,
    path: &str,
    row_label: &str,
    value_field: &str,
    status: Status,
    expected: Decimal,
    tolerance: Option<Decimal>,
    previous: Option<Decimal>,
    reason: Option<&str>,
    actual: Decimal,
) {
    check_to(
        &mut std::io::stdout(),
        id,
        path,
        row_label,
        value_field,
        status,
        expected,
        tolerance,
        previous,
        reason,
        actual,
    );
}

/// Same as `check`, but writes the `UnderReview` marker line to `out`
/// instead of unconditionally to stdout -- lets tests capture it into a
/// buffer, since there's no stable way to intercept a real `println!`'s
/// destination from outside.
#[allow(clippy::too_many_arguments)]
fn check_to(
    out: &mut impl std::io::Write,
    id: &str,
    path: &str,
    row_label: &str,
    value_field: &str,
    status: Status,
    expected: Decimal,
    tolerance: Option<Decimal>,
    previous: Option<Decimal>,
    reason: Option<&str>,
    actual: Decimal,
) {
    let message = |summary: &str| -> String {
        let mut parts = vec![
            format!("{id}: {summary}"),
            format!("path={path}"),
            format!("row={row_label}"),
            format!("column={value_field}"),
            format!("status={status:?}"),
            format!("expected={expected}"),
            format!("actual={actual}"),
            format!("delta={:+.2}", actual - expected),
        ];
        if let Some(t) = tolerance {
            parts.push(format!("tolerance={t}"));
        }
        if let Some(p) = previous {
            parts.push(format!("previous={p}"));
        }
        if let Some(r) = reason {
            parts.push(format!("reason={r}"));
        }
        parts.join(", ")
    };
    let reason_is_blank = reason.map(|r| r.trim().is_empty()).unwrap_or(true);

    if status == Status::ConfirmedDrift {
        if reason_is_blank {
            panic!(
                "{}",
                message(
                    "Status::ConfirmedDrift requires a non-blank .reason(...) \
                     explaining the accepted deviation"
                )
            );
        }
        match previous {
            None => panic!(
                "{}",
                message(
                    "Status::ConfirmedDrift requires a .previous(...) \
                     recording the old (wrong) figure it replaced"
                )
            ),
            Some(p) if p == expected => panic!(
                "{}",
                message(
                    "Status::ConfirmedDrift's .previous(...) equals `expected` \
                     -- nothing was actually corrected, so there's no drift to confirm"
                )
            ),
            Some(_) => {}
        }
    }
    if status == Status::UnderReview {
        if reason_is_blank {
            panic!(
                "{}",
                message(
                    "Status::UnderReview requires a non-blank .reason(...) explaining \
                     why review is postponed"
                )
            );
        }
        // Printed even when the assertion below passes, so a case still
        // awaiting review stays discoverable: `cargo test -- --show-output`
        // (or `--nocapture`) and grep for this marker lists every one.
        writeln!(out, "UNDER REVIEW: {}", message("under review")).unwrap();
    }
    if tolerance.is_some() && status != Status::Tolerated {
        panic!(
            "{}",
            message(
                ".tolerance(...) only applies to Status::Tolerated -- did you \
                 forget to `.status(Status::Tolerated)`?"
            )
        );
    }
    if status == Status::Tolerated {
        let t = tolerance.unwrap_or_else(|| {
            panic!(
                "{}",
                message("Status::Tolerated requires a .tolerance(...)")
            )
        });
        assert!(
            (actual - expected).abs() <= t,
            "{}",
            message(&format!("outside tolerance {t}"))
        );
        return;
    }
    assert_eq!(
        actual,
        expected,
        "{}\n\n\
         If the historical (filed) figure was wrong and the export above is \
         now correct, see scenario (1)/(3) in the `reconciliation` module \
         docs: mark this case `.status(Status::ConfirmedDrift)` with \
         `.previous(...)`/`.reason(...)` explaining why. If the export is \
         wrong and the historical figure was correct, that's a \
         hledger-elster bug -- please open one at \
         https://github.com/roschaefer/hledger-elster/issues",
        message("value mismatch")
    );
}

pub fn assert_csv_value(export_root: &Path, case: &Case) {
    let actual = read_csv_value(
        export_root,
        case.path,
        case.key_field,
        case.key_value,
        case.value_field,
    );
    check(
        case.id,
        case.path,
        case.key_value,
        case.value_field,
        case.status,
        case.expected,
        case.tolerance,
        case.previous,
        case.reason,
        actual,
    );
}

pub fn assert_csv_sum(export_root: &Path, case: &SumCase) {
    if case.key_values.is_empty() {
        panic!(
            "{}: SumCase.key_values is empty -- a SumCase with no keys always sums to zero \
             without ever reading {}, so it would pass no matter what the export contains",
            case.id, case.path
        );
    }
    let mut seen = std::collections::HashSet::new();
    for key_value in case.key_values {
        if !seen.insert(*key_value) {
            panic!(
                "{}: SumCase.key_values contains {key_value:?} more than once -- every row \
                 matching it would be counted twice",
                case.id
            );
        }
    }
    let actual = case
        .key_values
        .iter()
        .fold(Decimal::ZERO, |acc, key_value| {
            acc + sum_csv_value(
                export_root,
                case.path,
                case.key_field,
                key_value,
                case.value_field,
            )
        });
    let row_label = case.key_values.join("+");
    check(
        case.id,
        case.path,
        &row_label,
        case.value_field,
        case.status,
        case.expected,
        case.tolerance,
        case.previous,
        case.reason,
        actual,
    );
}

/// Rust analogue of `pytest.mark.parametrize`: expands to one independently
/// runnable/filterable `#[test]` fn per case.
#[macro_export]
macro_rules! case_test {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            $crate::reconciliation::assert_csv_value(&$crate::reconciliation::export_dir(), &$case);
        }
    };
}

#[macro_export]
macro_rules! case_sum_test {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            $crate::reconciliation::assert_csv_sum(&$crate::reconciliation::export_dir(), &$case);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report_writer::generate_report;
    use rust_decimal_macros::dec;
    use std::path::Path;

    fn with_example_export<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = crate::paths::ENV_LOCK.lock().unwrap();

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let journal = manifest_dir.join("examples/ledger/hledger.journal");
        let out_dir = tempfile::tempdir().unwrap();

        std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal);
        std::env::set_var("FINANCES_TAX_DATA_DIR", out_dir.path());
        std::env::remove_var("HLEDGER_ELSTER_CONFIG");

        let exit_code = generate_report().unwrap();

        let result = f(out_dir.path());

        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        std::env::remove_var("FINANCES_TAX_DATA_DIR");

        assert_eq!(exit_code, 0);
        result
    }

    #[test]
    fn export_dir_resolves_via_finances_tax_data_dir_and_matches_what_was_written() {
        with_example_export(|expected_dir| {
            assert_eq!(export_dir(), expected_dir);
        });
    }

    #[test]
    fn export_dir_panics_with_a_pointer_to_the_env_var_when_nothing_was_exported() {
        let _guard = crate::paths::ENV_LOCK.lock().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        std::env::set_var(
            "FINANCES_TAX_DATA_DIR",
            out_dir.path().join("never-written"),
        );
        let result = std::panic::catch_unwind(export_dir);
        std::env::remove_var("FINANCES_TAX_DATA_DIR");

        let err = result.expect_err("export_dir() should panic when nothing was exported yet");
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(
            message.contains("does not exist"),
            "unexpected panic message: {message}"
        );
    }

    #[test]
    fn read_csv_value_parses_german_formatted_amounts_back_into_decimal() {
        with_example_export(|export_root| {
            let actual = read_csv_value(
                export_root,
                "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                "Kennzahl",
                "Steuerpflichtiger Gewinn/Verlust",
                "2024",
            );
            assert_eq!(actual, dec!(-824.22));
        });
    }

    #[test]
    fn assert_csv_value_passes_for_a_matching_case_and_fails_for_a_wrong_one() {
        with_example_export(|export_root| {
            assert_csv_value(
                export_root,
                &Case::new(
                    "test/matching",
                    "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                    "Kennzahl",
                    "Vereinnahmte Umsatzsteuer",
                    "2024",
                    dec!(190.00),
                ),
            );

            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/mismatching",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(1.00),
                    ),
                );
            });
            let err = result.expect_err("mismatching case should panic");
            let message = err
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            assert!(
                message.contains("github.com/roschaefer/hledger-elster/issues"),
                "mismatch panic should point at the bug tracker: {message}"
            );
        });
    }

    #[test]
    fn assert_csv_value_confirmed_drift_matches_corrected_expected_exactly() {
        with_example_export(|export_root| {
            // Scenario (1)/(3) from the module docs: `expected` is pinned to
            // what the export actually (correctly) produces; `previous` is
            // only ever a documentation field and is never compared against.
            assert_csv_value(
                export_root,
                &Case::new(
                    "test/confirmed-drift",
                    "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                    "Kennzahl",
                    "Vereinnahmte Umsatzsteuer",
                    "2024",
                    dec!(190.00),
                )
                .status(Status::ConfirmedDrift)
                .previous(dec!(150.00))
                .reason("test fixture: historical figure was wrong, corrected here"),
            );

            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/confirmed-drift-still-fails-on-real-drift",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(1.00),
                    )
                    .status(Status::ConfirmedDrift)
                    .previous(dec!(150.00))
                    .reason("this reason does not make an unrelated drift pass"),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_confirmed_drift_requires_a_reason() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/confirmed-drift-no-reason",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::ConfirmedDrift),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_confirmed_drift_rejects_a_blank_reason() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/confirmed-drift-blank-reason",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::ConfirmedDrift)
                    .previous(dec!(150.00))
                    .reason("   "),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_confirmed_drift_requires_previous() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/confirmed-drift-no-previous",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::ConfirmedDrift)
                    .reason("no previous set, should panic"),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_confirmed_drift_requires_previous_to_differ_from_expected() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/confirmed-drift-previous-equals-expected",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::ConfirmedDrift)
                    .previous(dec!(190.00))
                    .reason("previous equals expected, should panic"),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_within_tolerance_accepts_small_drift() {
        with_example_export(|export_root| {
            assert_csv_value(
                export_root,
                &Case::new(
                    "test/tolerance",
                    "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                    "Kennzahl",
                    "Vereinnahmte Umsatzsteuer",
                    "2024",
                    dec!(190.01),
                )
                .status(Status::Tolerated)
                .tolerance(dec!(0.05)),
            );
        });
    }

    #[test]
    fn assert_csv_value_tolerance_without_tolerated_status_panics() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/tolerance-without-status",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.01),
                    )
                    .tolerance(dec!(0.05)),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_tolerated_status_without_tolerance_panics() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/tolerated-without-tolerance",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::Tolerated),
                );
            });
            assert!(result.is_err());
        });
    }

    fn write_manifest(dir: &Path, paths: &[&str]) {
        std::fs::write(
            dir.join(crate::paths::MANIFEST_FILE_NAME),
            format!("{}\n", paths.join("\n")),
        )
        .unwrap();
    }

    #[test]
    fn read_csv_value_panics_when_multiple_rows_match_the_same_key() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("dup.csv"),
            "Kennzahl;2024\nFoo;1,00\nFoo;2,00\n",
        )
        .unwrap();
        write_manifest(dir.path(), &["dup.csv"]);

        let result = std::panic::catch_unwind(|| {
            read_csv_value(dir.path(), "dup.csv", "Kennzahl", "Foo", "2024")
        });
        assert!(result.is_err());
    }

    #[test]
    fn assert_csv_sum_adds_every_row_sharing_a_key_not_just_the_first() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("dup.csv"),
            "Kennzahl;2024\nFoo;1,00\nFoo;2,00\n",
        )
        .unwrap();
        write_manifest(dir.path(), &["dup.csv"]);

        assert_csv_sum(
            dir.path(),
            &SumCase::new(
                "test/dup-sum",
                "dup.csv",
                "Kennzahl",
                &["Foo"],
                "2024",
                dec!(3.00),
            ),
        );
    }

    #[test]
    fn assert_csv_sum_rejects_an_empty_key_list() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dup.csv"), "Kennzahl;2024\nFoo;1,00\n").unwrap();
        write_manifest(dir.path(), &["dup.csv"]);

        let result = std::panic::catch_unwind(|| {
            assert_csv_sum(
                dir.path(),
                &SumCase::new(
                    "test/empty-keys",
                    "dup.csv",
                    "Kennzahl",
                    &[],
                    "2024",
                    dec!(0.00),
                ),
            );
        });
        assert!(result.is_err());
    }

    #[test]
    fn assert_csv_sum_rejects_duplicate_keys() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dup.csv"), "Kennzahl;2024\nFoo;50,00\n").unwrap();
        write_manifest(dir.path(), &["dup.csv"]);

        let result = std::panic::catch_unwind(|| {
            assert_csv_sum(
                dir.path(),
                &SumCase::new(
                    "test/duplicate-keys",
                    "dup.csv",
                    "Kennzahl",
                    &["Foo", "Foo"],
                    "2024",
                    dec!(100.00),
                ),
            );
        });
        assert!(result.is_err());
    }

    #[test]
    fn read_csv_value_panics_when_export_has_no_manifest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dup.csv"), "Kennzahl;2024\nFoo;1,00\n").unwrap();

        let result = std::panic::catch_unwind(|| {
            read_csv_value(dir.path(), "dup.csv", "Kennzahl", "Foo", "2024")
        });
        let err = result.expect_err("missing manifest should panic");
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(
            message.contains("was not produced by `hledger elster`"),
            "unexpected panic message: {message}"
        );
    }

    #[test]
    fn read_csv_value_panics_when_the_path_is_stale_and_missing_from_the_manifest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("stale.csv"), "Kennzahl;2024\nFoo;1,00\n").unwrap();
        // Manifest lists a different, unrelated file -- "stale.csv" was left
        // over from an earlier run and never (re)written by this one.
        write_manifest(dir.path(), &["current.csv"]);

        let result = std::panic::catch_unwind(|| {
            read_csv_value(dir.path(), "stale.csv", "Kennzahl", "Foo", "2024")
        });
        let err = result.expect_err("stale path should panic");
        let message = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(
            message.contains("was not (re)generated by the most recent"),
            "unexpected panic message: {message}"
        );
    }

    #[test]
    fn assert_csv_value_under_review_requires_a_reason() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/under-review-no-reason",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::UnderReview),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_under_review_rejects_a_blank_reason() {
        with_example_export(|export_root| {
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/under-review-blank-reason",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(190.00),
                    )
                    .status(Status::UnderReview)
                    .reason("   "),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn assert_csv_value_under_review_pins_the_postponed_value_and_still_catches_further_drift() {
        with_example_export(|export_root| {
            // Pinned to what the export currently produces: passes, same as
            // `ConfirmedDrift` would.
            assert_csv_value(
                export_root,
                &Case::new(
                    "test/under-review",
                    "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                    "Kennzahl",
                    "Vereinnahmte Umsatzsteuer",
                    "2024",
                    dec!(190.00),
                )
                .status(Status::UnderReview)
                .reason("test fixture: postponed for review"),
            );

            // A further drift away from the postponed value fails again,
            // rather than silently accepting an unrelated second
            // discrepancy under the same postponed reason.
            let result = std::panic::catch_unwind(|| {
                assert_csv_value(
                    export_root,
                    &Case::new(
                        "test/under-review-further-drift",
                        "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                        "Kennzahl",
                        "Vereinnahmte Umsatzsteuer",
                        "2024",
                        dec!(1.00),
                    )
                    .status(Status::UnderReview)
                    .reason("test fixture: postponed for review"),
                );
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn under_review_status_prints_a_marker_line_for_listing_cases_still_open() {
        // check_to doesn't itself read a CSV -- only assert_csv_value's
        // caller does -- so this exercises the print directly, without
        // needing a real export.
        let mut out = Vec::new();
        check_to(
            &mut out,
            "test/under-review-marker",
            "some.csv",
            "SomeRow",
            "2024",
            Status::UnderReview,
            dec!(100.00),
            None,
            None,
            Some("waiting on a hledger-elster bug report"),
            dec!(100.00),
        );
        let printed = String::from_utf8(out).unwrap();
        assert!(
            printed.starts_with("UNDER REVIEW: test/under-review-marker: under review, "),
            "unexpected output: {printed}"
        );
        assert!(
            printed.contains("reason=waiting on a hledger-elster bug report"),
            "unexpected output: {printed}"
        );
    }

    #[test]
    fn assert_csv_sum_adds_up_matching_rows_across_a_key() {
        with_example_export(|export_root| {
            assert_csv_sum(
                export_root,
                &SumCase::new(
                    "test/sum",
                    "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
                    "Kennzahl",
                    &["Entnahmen", "Einlagen"],
                    "2024",
                    dec!(500.00),
                ),
            );
        });
    }
}
