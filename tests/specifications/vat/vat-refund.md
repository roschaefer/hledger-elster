# VAT refund from the Finanzamt is part of the profit

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
$ build_scenario scenario_vat
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> account expenses:taxes:umsatzsteuer:erstattung  ; elster_role:vat_payment
> 
> 2025-02-01 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> 
> 2025-03-01 VAT refund for 2024
>     expenses:taxes:umsatzsteuer:erstattung  -10.00 EUR
>     assets:bank:business                     10.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2025
```

```scrut
$ csv_table export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl                                                    | 2025
# Betriebseinnahmen                                         |
Umsatzsteuerpflichtige Betriebseinnahmen                    | 100,00
Vereinnahmte Umsatzsteuer                                   | 19,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 10,00
Summe Betriebseinnahmen                                     | 129,00
                                                            |
# Betriebsausgaben                                          |
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 0,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | 129,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```
