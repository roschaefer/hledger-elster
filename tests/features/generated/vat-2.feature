# Generated from docs/specs/vat.md
# Run: python scripts/generate_features.py

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
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
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
