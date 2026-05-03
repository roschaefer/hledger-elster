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
