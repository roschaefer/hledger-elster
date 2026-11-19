# Several assets on one account depreciate to the cent

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_afa
```

Two laptops on one account each depreciate `1000.00 EUR / 36 * 7 = 194.444...`
in 2024. Each asset is rounded on its own, so the EÜR and the Herleitung both
show `388,88`. Over the full period each asset adds up to its net cost: the last
year takes the rounding remainder, `138,90` instead of `138,89`.

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:hardware:laptop  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_afa_years:3, elster_item:Laptop, elster_section:Arbeitsmittel
> 
> 2024-06-01 First laptop
>     expenses:business:hardware:laptop   1000.00 EUR
>     assets:bank:business               -1000.00 EUR
> 
> 2024-06-01 Second laptop
>     expenses:business:hardware:laptop   1000.00 EUR
>     assets:bank:business               -1000.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
  2025
  2026
  2027
```

```scrut
$ csv_table export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl                                                    | 2024
# Betriebseinnahmen                                         |
Umsatzsteuerpflichtige Betriebseinnahmen                    | 0,00
Vereinnahmte Umsatzsteuer                                   | 0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0,00
Summe Betriebseinnahmen                                     | 0,00
                                                            |
# Betriebsausgaben                                          |
# Arbeitsmittel                                             |
AfA Laptop                                                  | 388,88
Summe Arbeitsmittel                                         | 388,88
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 388,88
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -388,88
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-laptop.csv
Beschreibung  | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 | Anteil | Abziehbar
First laptop  | 2024-06-01 | 1000,00           | 3         | 27,78      | 194,44   | 1,00   | 194,44
Second laptop | 2024-06-01 | 1000,00           | 3         | 27,78      | 194,44   | 1,00   | 194,44
GESAMT        |            |                   |           |            | 388,88   |        | 388,88
```

The last year takes the remainder:

```scrut
$ csv_table export/2027/herleitung/einnahmen-ueberschuss-rechnung/afa-laptop.csv
Beschreibung  | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2027 | Anteil | Abziehbar
First laptop  | 2024-06-01 | 1000,00           | 3         | 27,78      | 138,90   | 1,00   | 138,90
Second laptop | 2024-06-01 | 1000,00           | 3         | 27,78      | 138,90   | 1,00   | 138,90
GESAMT        |            |                   |           |            | 277,80   |        | 277,80
```
