# A larger non-digital business asset is depreciated over several years

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

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:business:hardware:camera  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_afa_years:7, elster_item:Kamera, elster_section:Arbeitsmittel
> 
> 2024-01-01 Camera AfA
>     expenses:business:hardware:camera   1400.00 EUR
>     assets:bank:business               -1400.00 EUR
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
# Arbeitsmittel;
AfA Kamera;200,00
Summe Arbeitsmittel;200,00
;
An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer;0,00
Summe Betriebsausgaben;200,00
;
# Ermittlung des Gewinns;
Steuerpflichtiger Gewinn/Verlust;-200,00
;
# Zusätzliche Angaben bei Einzelunternehmen;
Entnahmen;0,00
Einlagen;0,00
```

```scrut
$ cat export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-kamera.csv
Beschreibung;Kaufdatum;Kaufpreis (Netto);AfA-Jahre;Monat. AfA;AfA 2024
Camera AfA;2024-01-01;1400,00;7;16,67;200,00
GESAMT;;;;;200,00
```
