# ADR 0005: Rust rewrite

## Status

Accepted

## Context

`hledger-elster` was a ~2400-line Python CLI (`uv`, `ruff`, `ty`, `pytest`,
`behave`, `openpyxl`) distributed by cloning the repository and running it
through `uv run`. The sibling tool `hledger-document-check` had already gone
through the same kind of rewrite (see its `rust-rewrite` branch), replacing a
Python interpreter + virtualenv + `openpyxl` with a single dependency-free
binary, installed with a curl one-liner against a rolling GitHub release.

The motivation for doing the same here is **distribution**, not performance:
this tool is not compute-bound, and a personal tax journal parses and renders
in well under a second in either language. What Rust buys is a static binary
with no runtime dependency beyond `hledger` itself.

Unlike `hledger-document-check`, this repository already has an established
ADR practice (0001-0004) and a domain-docs practice (`docs/domain/`,
`docs/migration/`). This ADR exists because that practice is worth
continuing here, not because the reference project had one to copy from — it
didn't; its own rewrite was recorded as a single well-described commit with
no ADR trail.

## Decisions

### 1. Flat single-binary crate, no workspace, no `lib.rs`

`src/*.rs` mirrors the reference project's convention: one file per concern,
no nested module directories, `main.rs` declares every module and stays a
thin CLI dispatcher. Unit tests live inline as `#[cfg(test)] mod tests` at
the bottom of the module they test, which works for a binary-only crate
without needing a library target.

The Python package layout (`domain/`, `ingest/`, `calculate/`,
`calculate/report/`) collapses into flat modules: `posting.rs`, `dataset.rs`,
`journal.rs`, `enrich.rs`, `afa.rs`, `aggregates.rs`, `drawing.rs`,
`classification.rs`, `periods.rs`, `euer.rs`, `est.rs`, `ust.rs`,
`herleitung.rs`, `report_writer.rs`, `config.rs`, `paths.rs`.

### 2. Exact decimal arithmetic, not `f64`

`rust_decimal::Decimal` with `RoundingStrategy::MidpointAwayFromZero`
reproduces Python's `Decimal.quantize(..., ROUND_HALF_UP)` exactly (verified
across 20 half-cent boundary cases before any business logic was ported).

This is a deliberate divergence from `hledger-document-check`, which
represents money as `f64` with a fuzzy-match tolerance — acceptable there
because it only compares extracted PDF amounts against booked amounts, not
because it computes filed tax figures. A tax tool has no such tolerance:
every output cell must reproduce the filed value to the cent.

Postings are also rebuilt from hledger's exact `decimalMantissa`/
`decimalPlaces` fields rather than its lossy `floatingPoint` convenience
field, which the Python implementation used (`Decimal(str(quantity))`,
inheriting Python's incidental `ROUND_HALF_EVEN`-via-string-formatting
behavior for the case where a journal amount has more than two decimal
places). This is bit-identical to the Python output for every real EUR
amount actually used in practice.

### 3. Build-time spec generation, nothing committed

`specs/*.md` (moved from `docs/specs/`, numbered `00`-`08` per the reference
project's convention) contain fenced ` ```gherkin ` blocks exactly as
before. `build.rs` extracts them into `$OUT_DIR/features/*.feature` at
compile time; `tests/cucumber.rs` runs them against the compiled binary via
`cucumber`. This replaces the Python setup's committed
`tests/features/generated/*.feature` plus the separate
`just check-generated-features` drift-check test — drift between docs and
executed specs is now structurally impossible rather than merely checked.

### 4. xlsx determinism is generated in memory, not patched on disk

The Python tool wrote the `.xlsx` to disk with `openpyxl`, then reopened and
rewrote the zip to force sorted entries, fixed per-entry timestamps, and
fixed `docProps/core.xml` content. The Rust port generates each workbook to
an in-memory buffer (`Workbook::new_from_buffer` + `close_to_buffer`), then
re-packs that buffer the same way before ever touching disk. Verified by a
unit test that stabilizes the same raw bytes twice and asserts byte
equality, and manually by running the compiled binary twice into separate
directories and diffing every generated file.

### 5. Business-logic and BDD parity, not a blind port

Every Python module's tests were treated as a literal porting checklist:
`test_sign_regressions.py`'s ~40 scenarios became inline `#[cfg(test)]` unit
tests colocated with the module that owns that rule; `test_euer_examples.py`
/ `test_est_examples.py` / `test_generate_report_examples.py`'s golden
numeric values (2024 net income 1000.00, collected VAT 190.00, Gewinn
-824.22, AfA 222.22/333.33, USt "Bereits Entrichtet" 190.00) are asserted
verbatim against the same `examples/ledger/hledger.journal` fixture; all 9
`specs/*.md` files (14 scenarios, 67 steps) pass unchanged against the
compiled binary.

Two pieces of genuinely dead Python code were dropped rather than ported:
`_account_label`/`_account_section` (defined in both `euer.py` and
`est.py`, never called), and `herleitung.py`'s non-signed `_deductible_net`/
`_deductible_vat` (only the `_signed_` variants were ever called).

## Consequences

- `src/**/*.py`, `tests/**/*.py`, `pyproject.toml`, and the `uv`/`ruff`/`ty`/
  `pytest`/`behave` toolchain are removed once the Rust port reaches parity
  (tracked as the final cutover step of the rewrite).
- `docs/adr/0001`-`0004` and `docs/domain/*`/`docs/migration/*` are
  unchanged — they document business rules and prior architecture decisions
  that remain valid regardless of implementation language.
- `docs/domain/verification.md` describes a `confirmed`/`needs_review`
  pytest-marker mechanism (`src/test_support.py`) consumed by **private**
  verification tests living outside this repository. No public test uses it
  today, and this rewrite does not build a Rust equivalent — that remains an
  open item for whoever maintains the private test suite (either keep it as
  a black-box Python subprocess suite against the compiled binary, or port
  it to Rust separately).
- The `elster_*` tag contract in `README.md` — the language-agnostic
  interface between a journal author and the tool — carries over verbatim.
- License changes to MIT (matching `hledger-document-check`), enabling the
  same curl-installable rolling-release distribution model.
