# VAT advance reversals are refunds in the booking year

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

A refunded advance belongs to EÜR income in its booking year, just like a final
VAT refund. The USt export instead subtracts it from the tax period settled.
The Herleitung lists both cash movements with their original signs.

```scrut
$ build_scenario scenario_vat
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:vat:advance  ; elster_role:vat_advance, elster_period:2024-03
> account expenses:vat:final  ; elster_role:vat_payment
>
> 2024-04-10 VAT advance for March
>     expenses:vat:advance  100.00 EUR
>     assets:bank:business
>
> 2024-06-10 VAT advance refund
>     expenses:vat:advance  -40.00 EUR
>     assets:bank:business
>
> 2025-01-10 Later advance refund for March 2024
>     expenses:vat:advance  -10.00 EUR
>     assets:bank:business
>
> 2025-02-10 Final VAT refund
>     expenses:vat:final  -20.00 EUR
>     assets:bank:business
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
  2025
```

```scrut
$ csv_table export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl                                                    | 2024
# Betriebseinnahmen                                         |
Umsatzsteuerpflichtige Betriebseinnahmen                    | 0,00
Vereinnahmte Umsatzsteuer                                   | 0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 40,00
Summe Betriebseinnahmen                                     | 40,00
                                                            |
# Betriebsausgaben                                          |
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 100,00
Summe Betriebsausgaben                                      | 100,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -60,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2024/herleitung/einnahmen-ueberschuss-rechnung/vom-finanzamt-erstattete-und-gg.csv
Konto            | Datum      | Beschreibung       | Betrag
Geschäftskonto   | 2024-06-10 | VAT advance refund | -40,00
Σ Geschäftskonto |            |                    | -40,00
GESAMT           |            |                    | -40,00
```

The following year includes both kinds of refund in EÜR income and its trail.

```scrut
$ csv_table export/2025/herleitung/einnahmen-ueberschuss-rechnung/vom-finanzamt-erstattete-und-gg.csv
Konto            | Datum      | Beschreibung                        | Betrag
Geschäftskonto   | 2025-01-10 | Later advance refund for March 2024 | -10,00
Geschäftskonto   | 2025-02-10 | Final VAT refund                    | -20,00
Σ Geschäftskonto |            |                                     | -30,00
GESAMT           |            |                                     | -30,00
```

The advance refunds reduce both the monthly and annual USt settlement,
including the refund booked in 2025. The final refund is not an advance settlement.

```scrut
$ python3 - <<'PYTHON'
> import csv
> with open('export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv') as source:
>     rows = {row['Kennzahl']: row['2025'] for row in csv.DictReader(source, delimiter=';')}
> for label in ['Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer', 'Steuerpflichtiger Gewinn/Verlust']:
>     print(label + ':', rows[label])
> with open('export/2024/steuererklaerung/umsatzsteuer.csv') as source:
>     row = next(row for row in csv.DictReader(source, delimiter=';') if row['Kennzahl'] == 'Bereits Entrichtet')
> print('2024-03:', row['2024-03'])
> print('2024:', row['2024'])
> PYTHON
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer: 30,00
Steuerpflichtiger Gewinn/Verlust: 30,00
2024-03: 50,00
2024: 50,00
```
