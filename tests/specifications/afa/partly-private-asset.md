# A partly private asset depreciates only its business share

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

The phone account is used 20% for business. A phone above `800.00 EUR` net
switches to AfA, but the switch only changes when the cost is deducted, not how
much of it: each year deducts 20% of that year's depreciation.

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:phone  ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.20, elster_vat:not_applicable, elster_item:Mobiltelefon, elster_section:Arbeitsmittel
> account expenses:phone:hardware  ; elster_afa_years:5, elster_item:Handykauf
> 
> 2024-01-01 Phone
>     expenses:phone:hardware   1200.00 EUR
>     assets:bank:business     -1200.00 EUR
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
AfA Handykauf                                               | 48,00
Summe Arbeitsmittel                                         | 48,00
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 48,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -48,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-handykauf.csv
Beschreibung | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 | Anteil | Abziehbar
Phone        | 2024-01-01 | 1200,00           | 5         | 20,00      | 240,00   | 0,20   | 48,00
GESAMT       |            |                   |           |            | 240,00   |        | 48,00
```
