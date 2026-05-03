# hledger-elster

`hledger-elster` generates German tax exports for `ELSTER.de` from an `hledger` journal.

Outputs are written under `data/exports/<year>/` by default:

- `steuererklaerung/`: summary CSV files plus `steuererklaerung.xlsx`
- `herleitung/`: detailed audit trails plus per-form `.xlsx`

Workbook export convention:

- every `name.xlsx` workbook has a sibling `name/` directory
- each workbook tab is also exported there as a CSV with the corresponding derived filename

Usage:

```bash
./hledger-elster
./hledger-elster -f examples/ledger/hledger.journal
./hledger-elster -f examples/ledger/hledger.journal -o /tmp/elster-out
```

Arguments:

- `-f`, `--file`: input journal, with the same meaning as `hledger -f`
- `-o`, `--output-dir`: output directory for generated tax artifacts

Typical development commands:

```bash
just test
```

Sanitized public fixtures live under [`examples/`](./examples). Keep real journals,
tax filings, and verification data outside this repository.
