# A non-deductible asset stays non-deductible when it is depreciated

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

`elster_afa_years` on a `non_deductible` account switches the asset to AfA, but
none of the depreciation is deductible.

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:television  ; elster_form:einnahmenueberschussrechnung, elster_deduction:non_deductible, elster_vat:not_applicable, elster_afa_years:3, elster_item:Fernseher, elster_section:Arbeitsmittel
> 
> 2024-01-01 Television
>     expenses:business:television   1200.00 EUR
>     assets:bank:business           -1200.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
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
AfA Fernseher                                               | 0,00
Summe Arbeitsmittel                                         | 0,00
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 0,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | 0,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-fernseher.csv
Beschreibung | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 | Anteil | Abziehbar
Television   | 2024-01-01 | 1200,00           | 3         | 33,33      | 400,00   | 0,00   | 0,00
GESAMT       |            |                   |           |            | 400,00   |        | 0,00
```
