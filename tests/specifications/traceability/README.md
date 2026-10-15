# Traceability

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

This is for you, the person who has to stand behind these numbers — not just
the software. When the tax office asks how a figure was arrived at, you
should be able to point at a row in a spreadsheet and add it up yourself,
rather than trusting the tool's arithmetic on faith. Every numeric line in
the main output forms (EÜR, ESt, USt) is backed by a corresponding Herleitung
("derivation") sheet that lists the individual transactions contributing to
that figure — no form value is opaque.

Take `Umsatzsteuerpflichtige Betriebseinnahmen` (VAT-liable business income)
in the EÜR export below: it shows `100.00`. That figure is not just asserted
— it comes from `herleitung/einnahmen-ueberschuss-rechnung/einnahmen.csv`,
which lists the one contributing invoice and sums to the same `100.00` in its
`Netto` column.

Traceability is not limited to the EÜR. The same business income also drives
the USt export's `Einnahme (Netto)` column, and USt ships its own audit
trail: `herleitung/umsatzsteuer/einnahmen.csv`. Both sheets are built from the
exact same underlying postings, so verifying the USt figures never requires
trusting a cross-form reference — the derivation is right there, in the USt
form's own Herleitung.

`Entnahmen` (owner draws) and `Einlagen` (owner contributions) illustrate the
same guarantee from a different angle, because they are computed twice from
the same underlying rule — once for the EÜR summary line, once for the
Herleitung sheet — and both computations must agree:

- `elster_account:business` marks the bank account that makes unclassified
  owner draws and contributions visible.
- An outflow from a `business` account is an Entnahme unless it is EÜR
  income or expense (`elster_form:einnahmenueberschussrechnung`) or a tax
  payment (an `elster_role` listed in `NON_DRAWING_ROLES` in
  [`src/drawing.rs`](../../../src/drawing.rs), such as `vat_advance`); a
  matching inflow is an Einlage. Private expenses for other forms, such as
  health insurance for the ESt paid from the business account, are
  Entnahmen too.
- The EÜR `Entnahmen`/`Einlagen` rows and the `entnahmen`/`einlagen`
  Herleitung sheets are derived from the same underlying transactions, so
  they can never diverge.

### Betriebseinnahmen, Entnahmen, and Einlagen all trace back to a Herleitung sheet

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
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> 
> 2024-01-15 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> 
> 2024-09-01 Owner draw
>     liabilities:owner       50.00 EUR
>     assets:bank:business   -50.00 EUR
> 
> 2024-09-02 Owner contribution
>     liabilities:owner      -40.00 EUR
>     assets:bank:business    40.00 EUR
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
Entnahmen;50,00
Einlagen;40,00
```

```scrut
$ cat export/2024/herleitung/einnahmen-ueberschuss-rechnung/einnahmen.csv
Konto;Datum;Beschreibung;Brutto;Netto;USt-Betrag
Geschäftskonto;2024-01-15;Client invoice;119,00;100,00;19,00
Σ Geschäftskonto;;;119,00;100,00;19,00
GESAMT;;;119,00;100,00;19,00
```

```scrut
$ cat export/2024/herleitung/umsatzsteuer/einnahmen.csv
Konto;Datum;Beschreibung;Brutto;Netto;USt-Betrag
Geschäftskonto;2024-01-15;Client invoice;119,00;100,00;19,00
Σ Geschäftskonto;;;119,00;100,00;19,00
GESAMT;;;119,00;100,00;19,00
```

```scrut
$ cat export/2024/herleitung/einnahmen-ueberschuss-rechnung/entnahmen.csv
Konto;Datum;Beschreibung;Betrag
Geschäftskonto;2024-09-01;Owner draw;50,00
Σ Geschäftskonto;;;50,00
GESAMT;;;50,00
```

```scrut
$ cat export/2024/herleitung/einnahmen-ueberschuss-rechnung/einlagen.csv
Konto;Datum;Beschreibung;Betrag
Geschäftskonto;2024-09-02;Owner contribution;40,00
Σ Geschäftskonto;;;40,00
GESAMT;;;40,00
```
