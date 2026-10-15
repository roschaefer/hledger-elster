# Export Hygiene

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

The generator writes deterministic ELSTER artifacts into the selected output
directory. It also reports files it did not touch, because stale CSVs in the
export tree can be mistaken for current tax artifacts.

### Stale files in the export directory are reported

```scrut
$ new_scenario
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:checking  ; elster_account:private, elster_item:Girokonto
> account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_item:Krankenversicherung, elster_section:Vorsorgeaufwand
> 
> 2024-06-01 Health care contribution
>     expenses:private:health-care:kv   840.00 EUR
>     assets:bank:checking             -840.00 EUR
> EOF
```

```scrut
$ write_file export/2024/steuererklaerung/stale.csv <<'EOF'
> stale
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024

Warning: untouched files remain in ELSTER export directory. Consider emptying the export directory and running the tool again.
  */work/export/2024/steuererklaerung/stale.csv (glob)

```
