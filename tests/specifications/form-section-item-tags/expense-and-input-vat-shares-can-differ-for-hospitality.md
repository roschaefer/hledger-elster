# Expense and input VAT shares can differ for hospitality

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
$ build_scenario scenario_form_section_item_tags
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:hospitality  ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.70, elster_vat:contains_vat, elster_vat_rate:0.19, elster_input_vat_share:1.00, elster_item:Bewirtung, elster_section:Bewirtungskosten
> 
> 2024-04-10 Business dinner
>     expenses:business:hospitality   119.00 EUR
>     assets:bank:business           -119.00 EUR
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
Umsatzsteuerpflichtige Betriebseinnahmen;0,00
Vereinnahmte Umsatzsteuer;0,00
Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebseinnahmen;0,00
;
# Betriebsausgaben;
# Bewirtungskosten;
Bewirtung;70,00
;
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebskosten;70,00
Summe Betriebsausgaben;70,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;-70,00
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
2024-04;0,00;0,00;19,00;-19,00;
2024-05;0,00;0,00;0,00;0,00;
2024-06;0,00;0,00;0,00;0,00;
2024-07;0,00;0,00;0,00;0,00;
2024-08;0,00;0,00;0,00;0,00;
2024-09;0,00;0,00;0,00;0,00;
2024-10;0,00;0,00;0,00;0,00;
2024-11;0,00;0,00;0,00;0,00;
2024-12;0,00;0,00;0,00;0,00;
2024 Q1;0,00;0,00;0,00;0,00;
2024 Q2;0,00;0,00;19,00;-19,00;
2024 Q3;0,00;0,00;0,00;0,00;
2024 Q4;0,00;0,00;0,00;0,00;
2024;0,00;0,00;19,00;-19,00;0,00
```
