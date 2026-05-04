# AGENTS.md

## Scope

`hledger-elster/` contains the publishable ELSTER tool:

- `src/`: tax calculation and export code
- `tests/`: public tests
- `examples/`: sanitized example journals
- `docs/`: ELSTER-specific design and migration notes

Private data and private tests stay outside this repository.

## Commands

- `just export`
- `just test`
- `./hledger-elster -f <journal>`

## Boundaries

- Do not move private journals, private tax artifacts, or private verification tests into this directory.
- Keep ELSTER-facing output labels in German.
- Keep user-specific verification tests outside this repository.
- Keep workbook export structure consistent: `name.xlsx` has a sibling `name/` directory, and each workbook tab is also exported there as a CSV.

## Traceability invariant

Every numeric line in the main output forms (EÜR, ESt, USt) must be backed by a
corresponding Herleitung sheet that lists the individual transactions contributing
to that figure. A reader must be able to verify any form total by summing the rows
of the matching Herleitung sheet — no form value may be opaque.

Consequence: whenever a calculation is added or changed in `euer.py`, `est.py`, or
`ust.py`, the matching sheet in `herleitung.py` must be updated in the same change
so that EÜR ↔ Herleitung, ESt ↔ Herleitung, and USt ↔ Herleitung remain consistent.
Entnahmen and Einlagen follow the same rule: both the EÜR totals and the Herleitung
sheets use the shared `is_drawing` / `is_contribution` predicates from
`calculate/drawing.py` so they can never diverge.
