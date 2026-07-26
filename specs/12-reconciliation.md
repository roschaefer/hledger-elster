# Reconciliation library

A filed tax return is a permanent record: once ELSTER has accepted it, this
year's `Umsatzsteuerpflichtige Betriebseinnahmen` must keep reading `1000,00`
forever, even after next month's refactor, a config change, or a correction
to an account tag two years from now nudges the calculation. Nothing about
`cargo test` on this repository catches that on its own -- it only proves the
*current* ledger produces a *self-consistent* export, not that the export
still matches what was actually filed.

`hledger_elster::reconciliation` is a small library for writing that second
kind of test in your own ledger repository: pin specific cells of a
generated export to their previously verified values, and let `cargo test`
fail loudly the moment one of them drifts.

- `Case` pins one CSV cell: the single row found by `key_field == key_value`
  (more than one match is ambiguous and panics -- use a `SumCase` instead),
  the `value_field` column of that row, compared against `expected`.
- `SumCase` does the same but sums `value_field` across every row matching
  each of `key_values` -- for figures only meaningful once multiple rows
  (e.g. several bank sub-accounts, or two sections sharing one Kennzahl)
  are added together. `key_values` must be non-empty and free of duplicates --
  an empty list would sum to zero without ever reading the export, and a
  repeated key would count its rows twice.
- `case_test!`/`case_sum_test!` expand a `Case`/`SumCase` into an independently
  runnable `#[test]` fn, the same way `pytest.mark.parametrize` would.
- `export_dir()` resolves the export the same way `hledger elster` resolves
  where to *write* one: `FINANCES_TAX_DATA_DIR`, defaulting to
  `./data/exports`. Point both at the same directory and no further wiring is
  needed.
- Every `hledger elster` run writes a manifest of the files it touched, and a
  `Case`/`SumCase` refuses to read a CSV that isn't in it. This catches a
  year or form that's been removed from the journal but whose old CSV is
  still sitting in the export directory -- without the check, that stale
  file would be read as if it were current.

Amounts in the export are German-formatted (`,` decimal, per
[CSV/xlsx equivalence](./01-csv-xlsx-equivalence.md)); `reconciliation` parses
them back into `Decimal` for you, so a `Case`'s `expected` is an ordinary
`rust_decimal_macros::dec!(...)` value.

## What a drift means, and what to do about it

A drift is a disagreement between two things that are each supposed to be
correct: the figure that was actually filed (pinned as `expected`) and the
figure the export produces from today's ledger (`actual`). Only one of them
can be right -- either the filed figure was wrong (a bug in an old
calculation, an old tag, an old version of the tool), or the export is wrong
right now (a bug in the ledger, or in hledger-elster itself). A failing
`Case` doesn't tell you which; that takes a human looking at both numbers.

### The procedure

1. **A `Case` fails because of an unreviewed drift.** This is the default,
   unannotated state (`Status::Expected`): `expected` and `actual` disagree,
   and nobody has looked into why yet.
2. **The export was wrong; fixing the ledger turns the case green.** Once you
   find and fix the actual problem -- a missing posting, a wrong tag, a
   config mistake -- rerunning `hledger elster` produces the correct figure
   again, `actual` goes back to matching the already-pinned `expected`, and
   the case passes with no change to the test itself.
3. **The filed figure was wrong; the case is reconciled and turned green
   deliberately.** Sometimes the ledger and the tool are both right *now*,
   and it's the number that was actually filed that was wrong -- reviewing
   the drift reveals an old bug that produced it. Update `expected` to what
   the export correctly produces, move `.previous(...)` to the old (wrong)
   filed figure, set `.status(Status::ConfirmedDrift)`, and `.reason(...)`
   to explain the old bug. The case still asserts exact equality (against
   the corrected `expected`) -- `previous`/`reason` just keep the old figure
   and its explanation visible in the test source for whoever reads it next.

### Postponing a review (`UnderReview`)

Sometimes you can't immediately tell which of the above applies. Maybe
there's no time to dig into a given case this week. Maybe the drift looks
like a hledger-elster bug rather than a ledger mistake -- in which case,
please open one at
[github.com/roschaefer/hledger-elster/issues](https://github.com/roschaefer/hledger-elster/issues)
rather than quietly changing `expected` to match.

Either way, set `.status(Status::UnderReview)`, pin `expected` to what the
export produces *right now*, and `.reason(...)` to record why review is
postponed:

1. **A `Case` fails because of a drift the user can't review yet, so they
   postpone it with `UnderReview` and a reason.** This stops the case from
   failing. But it behaves exactly like `ConfirmedDrift` underneath -- if
   the ledger changes again and produces a *further* drift, the case fails
   again rather than silently absorbing an unrelated second discrepancy
   under the same postponed reason.
2. **Listing every case still awaiting review.** Every `UnderReview` case
   prints a `"UNDER REVIEW: ..."` line when it runs, even when it passes.
   Libtest normally suppresses output from passing tests, so run
   `cargo test -- --show-output` and grep for that marker to list every case
   still open.

```gherkin
Feature: Reconciliation library

  Background:
    Given a file named "fixture/Cargo.toml" with content:
      """
      [package]
      name = "fixture-reconciliation"
      edition = "2021"
      publish = false

      [dependencies]
      hledger-elster = { path = "{{HLEDGER_ELSTER_MANIFEST_DIR}}" }
      rust_decimal = "1"
      rust_decimal_macros = "1"
      """

  Scenario: A Case fails because of an unreviewed drift
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::Case;
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(999.00),
          )
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output" and it fails
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... FAILED

      successes:

      successes:

      failures:

      ---- umsatzsteuerpflichtige_betriebseinnahmen stdout ----

      ...

      assertion `left == right` failed: income:business: value mismatch, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=Expected, expected=999.00, actual=100.00, delta=-899.00

      If the historical (filed) figure was wrong and the export above is now correct, see scenario (1)/(3) in the `reconciliation` module docs: mark this case `.status(Status::ConfirmedDrift)` with `.previous(...)`/`.reason(...)` explaining why. If the export is wrong and the historical figure was correct, that's a hledger-elster bug -- please open one at https://github.com/roschaefer/hledger-elster/issues
        left: 100.00
       right: 999.00
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


      failures:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;
      """

  Scenario: Fixing the ledger turns a failing Case green
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -95.20 EUR
          assets:bank:business    95.20 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::Case;
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(100.00),
          )
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output" and it fails
    Then stdout should contain:
      """
      assertion `left == right` failed: income:business: value mismatch, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=Expected, expected=100.00, actual=80.00, delta=-20.00

      If the historical (filed) figure was wrong and the export above is now correct, see scenario (1)/(3) in the `reconciliation` module docs: mark this case `.status(Status::ConfirmedDrift)` with `.previous(...)`/`.reason(...)` explaining why. If the export is wrong and the historical figure was correct, that's a hledger-elster bug -- please open one at https://github.com/roschaefer/hledger-elster/issues
        left: 80.00
       right: 100.00
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


      failures:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;
      """
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output"
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... ok

      successes:

      successes:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
      """

  Scenario: Confirming the historical figure was wrong keeps the Case green and documented
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::{Case, Status};
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(100.00),
          )
          .status(Status::ConfirmedDrift)
          .previous(dec!(90.00))
          .reason("Historical figure missed a client invoice; corrected once the ledger was fixed")
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output"
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... ok

      successes:

      successes:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
      """

  Scenario: Postponing a review with UnderReview still catches a further drift
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::{Case, Status};
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(100.00),
          )
          .status(Status::UnderReview)
          .reason("No time to review this week")
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output"
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... ok

      successes:

      ---- umsatzsteuerpflichtige_betriebseinnahmen stdout ----
      UNDER REVIEW: income:business: under review, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=UnderReview, expected=100.00, actual=100.00, delta=+0.00, reason=No time to review this week


      successes:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
      """
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -178.50 EUR
          assets:bank:business   178.50 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output" and it fails
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... FAILED

      successes:

      successes:

      failures:

      ---- umsatzsteuerpflichtige_betriebseinnahmen stdout ----
      UNDER REVIEW: income:business: under review, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=UnderReview, expected=100.00, actual=150.00, delta=+50.00, reason=No time to review this week

      ...

      assertion `left == right` failed: income:business: value mismatch, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=UnderReview, expected=100.00, actual=150.00, delta=+50.00, reason=No time to review this week

      If the historical (filed) figure was wrong and the export above is now correct, see scenario (1)/(3) in the `reconciliation` module docs: mark this case `.status(Status::ConfirmedDrift)` with `.previous(...)`/`.reason(...)` explaining why. If the export is wrong and the historical figure was correct, that's a hledger-elster bug -- please open one at https://github.com/roschaefer/hledger-elster/issues
        left: 150.00
       right: 100.00
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


      failures:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;
      """

  Scenario: Listing every Case still awaiting review
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::{Case, Status};
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(100.00),
          )
          .status(Status::UnderReview)
          .reason("No time to review this week")
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output"
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... ok

      successes:

      ---- umsatzsteuerpflichtige_betriebseinnahmen stdout ----
      UNDER REVIEW: income:business: under review, path=2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv, row=Umsatzsteuerpflichtige Betriebseinnahmen, column=2024, status=UnderReview, expected=100.00, actual=100.00, delta=+0.00, reason=No time to review this week


      successes:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
      """

  Scenario: A Case refuses to read a CSV left over from a removed year
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "fixture/tests/reconciliation.rs" with content:
      """
      use hledger_elster::case_test;
      use hledger_elster::reconciliation::Case;
      use rust_decimal_macros::dec;

      case_test!(
          umsatzsteuerpflichtige_betriebseinnahmen,
          Case::new(
              "income:business",
              "2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv",
              "Kennzahl",
              "Umsatzsteuerpflichtige Betriebseinnahmen",
              "2024",
              dec!(100.00),
          )
      );
      """
    When I run "hledger elster -f journal.journal -o export"
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2023-01-15 Client invoice
          income:business       -50.00 EUR
          assets:bank:business    50.00 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    And I run "cargo test --manifest-path fixture/Cargo.toml -- --show-output" and it fails
    Then stdout should contain:
      """
      running 1 test
      test umsatzsteuerpflichtige_betriebseinnahmen ... FAILED

      successes:

      successes:

      failures:

      ---- umsatzsteuerpflichtige_betriebseinnahmen stdout ----

      ...

      2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv was not (re)generated by the most recent `hledger elster` run in:

      ...

      This is stale data left over from an earlier export (e.g. a year or form that no longer appears in the journal). Re-run `hledger elster` and check your journal includes cover this year/form before trusting this figure.
      note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


      failures:
          umsatzsteuerpflichtige_betriebseinnahmen

      test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;
      """
```
