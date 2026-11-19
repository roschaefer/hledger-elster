# A disposed asset writes off its remaining book value

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

The laptop is stolen in March 2025. AfA runs through March, and what is not
depreciated by then is written off in 2025 as `Restbuchwert`. Nothing is left for
2026. The journal has no posting in 2025, but the export still covers that year,
because the asset writes something off in it.

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:hardware:laptop  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_afa_years:3, elster_item:Laptop, elster_section:Arbeitsmittel
> 
> 2024-01-10 Laptop
>     expenses:business:hardware:laptop   3600.00 EUR  ; elster_disposal:2025-03-20
>     assets:bank:business               -3600.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
  2025
```

2024 depreciates twelve of 36 months:

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
AfA Laptop                                                  | 1200,00
Summe Arbeitsmittel                                         | 1200,00
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 1200,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -1200,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

2025 depreciates January to March and writes off the rest:

```scrut
$ csv_table export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl                                                    | 2025
# Betriebseinnahmen                                         |
Umsatzsteuerpflichtige Betriebseinnahmen                    | 0,00
Vereinnahmte Umsatzsteuer                                   | 0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0,00
Summe Betriebseinnahmen                                     | 0,00
                                                            |
# Betriebsausgaben                                          |
# Arbeitsmittel                                             |
AfA Laptop                                                  | 300,00
Restbuchwert Laptop                                         | 2100,00
Summe Arbeitsmittel                                         | 2400,00
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 2400,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -2400,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2025/herleitung/einnahmen-ueberschuss-rechnung/afa-laptop.csv
Beschreibung | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2025 | Abgang     | Restbuchwert | Anteil | Abziehbar
Laptop       | 2024-01-10 | 3600,00           | 3         | 100,00     | 300,00   | 2025-03-20 | 2100,00      | 1,00   | 2400,00
GESAMT       |            |                   |           |            | 300,00   |            | 2100,00      |        | 2400,00
```
