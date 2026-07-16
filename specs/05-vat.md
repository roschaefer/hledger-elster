# VAT Payments And Settlements

VAT payment accounts use role metadata instead of form metadata:

- `elster_role:vat_advance` marks VAT advance payments and reversals.
- `elster_period` assigns an advance payment to the VAT period it settles, which
  can differ from the booking year. Supported values are `YYYY`, `YYYY-Qn`, and
  `YYYY-MM`.
- `elster_role:vat_payment` marks final VAT payments or refunds. These affect the
  EÜR in the booking year, but they are not part of the "Bereits Entrichtet" sum
  for the USt year being settled unless they are tagged as an advance for that
  period.

```gherkin
Feature: VAT advance payment periods

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: VAT advance payments can be assigned by year, quarter, or month cadence
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance, elster_item:USt-Vorauszahlung
      account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
      account expenses:taxes:umsatzsteuer:vorauszahlung:2025:q1  ; elster_period:2025-Q1
      account expenses:taxes:umsatzsteuer:vorauszahlung:2026:march  ; elster_period:2026-03

      2024-02-15 VAT advance for 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  100.00 EUR
          assets:bank:business                           -100.00 EUR

      2024-03-01 VAT advance reversal for 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  -40.00 EUR
          assets:bank:business                             40.00 EUR

      2025-04-10 VAT advance for 2025 Q1
          expenses:taxes:umsatzsteuer:vorauszahlung:2025:q1   30.00 EUR
          assets:bank:business                              -30.00 EUR

      2026-04-11 VAT advance for March 2026
          expenses:taxes:umsatzsteuer:vorauszahlung:2026:march   20.00 EUR
          assets:bank:business                                 -20.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2024-01  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-02  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-03  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-04  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-05  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-06  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-07  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-08  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-09  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-10  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-11  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-12  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q1  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q2  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q4  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024     | 0.00             | 0.00                      | 0.00                        | 0.00               | 60.00              |
    And the CSV file "export/2025/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2025-01  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-02  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-03  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-04  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-05  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-06  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-07  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-08  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-09  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-10  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-11  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025-12  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025 Q1  | 0.00             | 0.00                      | 0.00                        | 0.00               | 30.00              |
      | 2025 Q2  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025 Q4  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2025     | 0.00             | 0.00                      | 0.00                        | 0.00               | 30.00              |
    And the CSV file "export/2026/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2026-01  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-02  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-03  | 0.00             | 0.00                      | 0.00                        | 0.00               | 20.00              |
      | 2026-04  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-05  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-06  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-07  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-08  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-09  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-10  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-11  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026-12  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026 Q1  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026 Q2  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026 Q4  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2026     | 0.00             | 0.00                      | 0.00                        | 0.00               | 20.00              |
```

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

```gherkin
Feature: VAT cash settlement

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: EÜR follows the booking year while USt settles against the tax period
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
      account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance
      account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024

      2024-12-20 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2025-01-10 Late VAT advance for 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024   19.00 EUR
          assets:bank:business                             -19.00 EUR
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
      | Entnahmen                                                   | 0.00   |
      | Einlagen                                                    | 0.00   |
    And the CSV file "export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2025   |
      | # Betriebseinnahmen                                         |        |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 0.00   |
      | Vereinnahmte Umsatzsteuer                                   | 0.00   |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00   |
      | Summe Betriebseinnahmen                                     | 0.00   |
      |                                                             |        |
      | # Betriebsausgaben                                          |        |
      |                                                             |        |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 19.00  |
      | Summe Betriebskosten                                        | 0.00   |
      | Summe Betriebsausgaben                                      | 19.00  |
      |                                                             |        |
      | # Ermittlung des Gewinns                                    |        |
      | Steuerpflichtiger Gewinn/Verlust                            | -19.00 |
      |                                                             |        |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |        |
      | Entnahmen                                                   | 0.00   |
      | Einlagen                                                    | 0.00   |
    And the CSV file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2024-01  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-02  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-03  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-04  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-05  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-06  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-07  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-08  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-09  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-10  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-11  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024-12  | 100.00           | 19.00                     | 0.00                        | 19.00               |                    |
      | 2024 Q1  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024 Q2  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00                |                    |
      | 2024 Q4  | 100.00           | 19.00                     | 0.00                        | 19.00               |                    |
      | 2024     | 100.00           | 19.00                     | 0.00                        | 19.00               | 19.00              |
```
