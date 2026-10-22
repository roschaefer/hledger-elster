# Configuration

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

`hledger-elster` uses a TOML config file for user-specific tax assumptions that
are not ledger transactions. The tool defaults the Home-Office-Pauschale to the
maximum legal amount for the year, because human users are likely to forget this
adjustment.

### Running without a config file uses human-friendly defaults

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> 
> 2024-01-10 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal -o export
  2024
```

```scrut
$ cat export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl;2024
# Betriebseinnahmen;
Umsatzsteuerpflichtige Betriebseinnahmen;100,00
Vereinnahmte Umsatzsteuer;19,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebseinnahmen;119,00
;
# Betriebsausgaben;
;
Home-Office-Pauschale;1260,00
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebsausgaben;1260,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;-1141,00
;
# Zusätzliche Angaben bei Einzelunternehmen;
Entnahmen;0,00
Einlagen;0,00
```

### The default config can be generated

```scrut
$ new_scenario
```

```scrut
$ hledger-elster init-config --output elster.toml
```

```scrut
$ cat elster.toml
[euer.home_office_pauschale]
enabled = true
default_days = "max"
# Set per-year days when the default does not match your situation.
# 2020-2022: 5 EUR/day, capped at 600 EUR.
# 2023+: 6 EUR/day, capped at 1260 EUR.

[euer.home_office_pauschale.days]
# 2024 = 210
```

### --help lists the init-config subcommand

```scrut
$ new_scenario
```

```scrut
$ hledger-elster --help
Generate ELSTER-oriented tax exports from an hledger journal.

Usage: hledger-elster [OPTIONS] [COMMAND]

Commands:
  init-config             Write a default hledger-elster TOML config file
  export-commit-evidence  Write a PDF identifying the current clean Git commit
  help                    Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>              
  -o, --output-dir <OUTPUT_DIR>  
      --config <CONFIG>          
  -h, --help                     Print help
  -V, --version                  Print version
```

The Home-Office-Pauschale config stores days, not amounts. The tool applies the
calendar-year policy:

- 2020-2022: 5 EUR per day, capped at 600 EUR.
- 2023 and later: 6 EUR per day, capped at 1260 EUR.

Acceptance tests that focus on journal-derived behavior pass an explicit config
file with the adjustment disabled.

### A custom config changes the Home-Office-Pauschale

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> 
> 2024-01-10 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = true
> default_days = "max"
> 
> [euer.home_office_pauschale.days]
> 2024 = 10
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
```

```scrut
$ cat export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl;2024
# Betriebseinnahmen;
Umsatzsteuerpflichtige Betriebseinnahmen;100,00
Vereinnahmte Umsatzsteuer;19,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebseinnahmen;119,00
;
# Betriebsausgaben;
;
Home-Office-Pauschale;60,00
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebsausgaben;60,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;59,00
;
# Zusätzliche Angaben bei Einzelunternehmen;
Entnahmen;0,00
Einlagen;0,00
```
