# Traceability

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
- An outflow from a `business` account with no matching `elster_form` or
  `elster_role` is an Entnahme; a matching inflow is an Einlage.
- The EÜR `Entnahmen`/`Einlagen` rows and the `entnahmen`/`einlagen`
  Herleitung sheets are derived from the same underlying transactions, so
  they can never diverge.

```gherkin
Feature: Traceability

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Betriebseinnahmen, Entnahmen, and Einlagen all trace back to a Herleitung sheet
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2024-09-01 Owner draw
          liabilities:owner       50.00 EUR
          assets:bank:business   -50.00 EUR

      2024-09-02 Owner contribution
          liabilities:owner      -40.00 EUR
          assets:bank:business    40.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2024   |
      | # Betriebseinnahmen                                         |        |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 100.00 |
      | Vereinnahmte Umsatzsteuer                                   | 19.00  |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00   |
      | Summe Betriebseinnahmen                                     | 119.00 |
      |                                                             |        |
      | # Betriebsausgaben                                          |        |
      |                                                             |        |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00   |
      | Summe Betriebskosten                                        | 0.00   |
      | Summe Betriebsausgaben                                      | 0.00   |
      |                                                             |        |
      | # Ermittlung des Gewinns                                    |        |
      | Steuerpflichtiger Gewinn/Verlust                            | 119.00 |
      |                                                             |        |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |        |
      | Entnahmen                                                   | 50.00  |
      | Einlagen                                                    | 40.00  |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/einnahmen.csv" should contain exactly:
      | Konto            | Datum      | Beschreibung   | Brutto | Netto  | USt-Betrag |
      | Geschäftskonto   | 2024-01-15 | Client invoice | 119.00 | 100.00 | 19.00      |
      | Σ Geschäftskonto |            |                | 119.00 | 100.00 | 19.00      |
      | GESAMT           |            |                | 119.00 | 100.00 | 19.00      |
    And the CSV file "export/2024/herleitung/umsatzsteuer/einnahmen.csv" should contain exactly:
      | Konto            | Datum      | Beschreibung   | Brutto | Netto  | USt-Betrag |
      | Geschäftskonto   | 2024-01-15 | Client invoice | 119.00 | 100.00 | 19.00      |
      | Σ Geschäftskonto |            |                | 119.00 | 100.00 | 19.00      |
      | GESAMT           |            |                | 119.00 | 100.00 | 19.00      |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/entnahmen.csv" should contain exactly:
      | Konto            | Datum      | Beschreibung | Betrag |
      | Geschäftskonto   | 2024-09-01 | Owner draw   | 50.00  |
      | Σ Geschäftskonto |            |              | 50.00  |
      | GESAMT           |            |              | 50.00  |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/einlagen.csv" should contain exactly:
      | Konto            | Datum      | Beschreibung        | Betrag |
      | Geschäftskonto   | 2024-09-02 | Owner contribution  | 40.00  |
      | Σ Geschäftskonto |            |                     | 40.00  |
      | GESAMT           |            |                     | 40.00  |
```
