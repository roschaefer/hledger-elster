# EÜR follows the booking year while USt settles against the tax period

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The EÜR and USt views use different time axes for VAT cash movements:

- `Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer` and `An das
  Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer` are EÜR cash-flow rows:
  they follow the booking date and therefore the year in which the payment or
  refund happened.
- `Vorauszahlungssoll` in the USt export is the VAT amount calculated for the
  tax period.
- `Bereits Entrichtet` in the USt export is settled against the tax period from
  `elster_period`. A payment booked in the following year can therefore settle
  the previous year's USt. Ideally, `Vorauszahlungssoll` and `Bereits Entrichtet`
  match for the settled year.

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_vat
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance
> account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
> 
> 2024-12-20 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> 
> 2025-01-10 Late VAT advance for 2024
>     expenses:taxes:umsatzsteuer:vorauszahlung:2024   19.00 EUR
>     assets:bank:business                             -19.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
  2025
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
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebskosten;0,00
Summe Betriebsausgaben;0,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;119,00
;
# Zusätzliche Angaben bei Einzelunternehmen;
Entnahmen;0,00
Einlagen;0,00
```

```scrut
$ cat export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv
Kennzahl;2025
# Betriebseinnahmen;
Umsatzsteuerpflichtige Betriebseinnahmen;0,00
Vereinnahmte Umsatzsteuer;0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebseinnahmen;0,00
;
# Betriebsausgaben;
;
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;19,00
Summe Betriebskosten;0,00
Summe Betriebsausgaben;19,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;-19,00
;
# Zusätzliche Angaben bei Einzelunternehmen;
Entnahmen;0,00
Einlagen;0,00
```

```scrut
$ cat export/2024/steuererklaerung/umsatzsteuer.csv
Zeitraum;Einnahme (Netto);Vereinnahmte Umsatzsteuer;Abziehbare Vorsteuerbeträge;Vorauszahlungssoll;Bereits Entrichtet
2024-01;0,00;0,00;0,00;0,00;
2024-02;0,00;0,00;0,00;0,00;
2024-03;0,00;0,00;0,00;0,00;
2024-04;0,00;0,00;0,00;0,00;
2024-05;0,00;0,00;0,00;0,00;
2024-06;0,00;0,00;0,00;0,00;
2024-07;0,00;0,00;0,00;0,00;
2024-08;0,00;0,00;0,00;0,00;
2024-09;0,00;0,00;0,00;0,00;
2024-10;0,00;0,00;0,00;0,00;
2024-11;0,00;0,00;0,00;0,00;
2024-12;100,00;19,00;0,00;19,00;
2024 Q1;0,00;0,00;0,00;0,00;
2024 Q2;0,00;0,00;0,00;0,00;
2024 Q3;0,00;0,00;0,00;0,00;
2024 Q4;100,00;19,00;0,00;19,00;
2024;100,00;19,00;0,00;19,00;19,00
```
