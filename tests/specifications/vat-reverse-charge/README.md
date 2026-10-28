# VAT Reverse Charge

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Reverse charge covers foreign supplier invoices where the vendor does not charge
German VAT, but the German business recipient has to declare German VAT.

For this tool, the reverse-charge values of `elster_vat` deliberately model
services, not goods:

- `elster_vat:reverse_charge_eu` is for B2B services from another EU country. In the
  annual USt form this belongs to the `§13b` bucket for services under
  `§3a Abs. 2 UStG` from another EU member state.
- `elster_vat:reverse_charge_non_eu` is for foreign services or work supplies that
  are not part of the EU service bucket.
- Physical goods bought from another EU country are usually
  `innergemeinschaftliche Erwerbe`. They use different ELSTER lines and are not
  part of this feature.

Reverse-charge invoices are booked net. `elster_vat_rate` is the German VAT rate
that is calculated on top of the booked amount. The rate can be inherited from a
parent account, so a child account can switch from `elster_vat:contains_vat` to
reverse charge by overriding only `elster_vat`.

Reverse charge has no special EÜR effect. The invoice amount stays a regular net
business expense under its `elster_item`; the calculated reverse-charge VAT is
only visible in the USt form. For a fully deductible business expense, it
increases both `§13b` VAT owed and `Abziehbare Vorsteuerbeträge` by the same
amount. Because no VAT was paid to the supplier or tax office as part of the
invoice transaction, the EÜR VAT cash-flow rows remain unchanged.

### Foreign service invoices create reverse-charge VAT and matching input VAT

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
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:hosting  ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:contains_vat, elster_vat_rate:0.19, elster_section:Bezogene Fremdleistungen
> account expenses:business:hosting:eu  ; elster_item:EU Hosting, elster_vat:reverse_charge_eu
> account expenses:business:hosting:non_eu  ; elster_item:Non-EU Hosting, elster_vat:reverse_charge_non_eu
> 
> 2026-03-15 EU SaaS invoice
>     expenses:business:hosting:eu   100.00 EUR
>     assets:bank:business          -100.00 EUR
> 
> 2026-03-16 US SaaS invoice
>     expenses:business:hosting:non_eu    50.00 EUR
>     assets:bank:business              -50.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2026
```

```scrut
$ csv_table export/2026/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl                                                    | 2026
# Betriebseinnahmen                                         |
Umsatzsteuerpflichtige Betriebseinnahmen                    | 0,00
Vereinnahmte Umsatzsteuer                                   | 0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0,00
Summe Betriebseinnahmen                                     | 0,00
                                                            |
# Betriebsausgaben                                          |
# Bezogene Fremdleistungen                                  |
EU Hosting                                                  | 100,00
Non-EU Hosting                                              | 50,00
Summe Bezogene Fremdleistungen                              | 150,00
                                                            |
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0,00
Summe Betriebsausgaben                                      | 150,00
                                                            |
# Ermittlung des Gewinns                                    |
Steuerpflichtiger Gewinn/Verlust                            | -150,00
                                                            |
# Zusätzliche Angaben bei Einzelunternehmen                 |
Entnahmen                                                   | 0,00
Einlagen                                                    | 0,00
```

```scrut
$ csv_table export/2026/steuererklaerung/umsatzsteuer.csv
Kennzahl                                             | 2026-01 | 2026-02 | 2026-03 | 2026 Q1 | 2026-04 | 2026-05 | 2026-06 | 2026 Q2 | 2026-07 | 2026-08 | 2026-09 | 2026 Q3 | 2026-10 | 2026-11 | 2026-12 | 2026 Q4 | 2026
# Lieferungen und sonstige Leistungen                |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
Einnahme (Netto)                                     | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00
Vereinnahmte Umsatzsteuer                            | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00
                                                     |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
# Leistungsempfänger als Steuerschuldner (§13b UStG) |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
§13b EU Leistung (Netto)                             | 0,00    | 0,00    | 100,00  | 100,00  | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 100,00
§13b EU Umsatzsteuer                                 | 0,00    | 0,00    | 19,00   | 19,00   | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 19,00
§13b Non-EU Leistung (Netto)                         | 0,00    | 0,00    | 50,00   | 50,00   | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 50,00
§13b Non-EU Umsatzsteuer                             | 0,00    | 0,00    | 9,50    | 9,50    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 9,50
                                                     |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
Abziehbare Vorsteuerbeträge                          | 0,00    | 0,00    | 28,50   | 28,50   | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 28,50
                                                     |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
# Berechnung der Umsatzsteuer                        |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |
Vorauszahlungssoll                                   | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00    | 0,00
Bereits Entrichtet                                   |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         |         | 0,00
```
