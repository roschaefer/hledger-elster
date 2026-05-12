# Generated from docs/specs/form-section-item-tags.md
# Run: python scripts/generate_features.py

Feature: Business expenses and income

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Form tags select exports and item tags translate or aggregate accounts
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account assets:bank:private   ; elster_account:private, elster_item:Girokonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
      account expenses:business:penalty  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_deduction:non_deductible, elster_item:Nicht abzugsfähige Betriebsausgabe, elster_section:Arbeitsmittel
      account expenses:charity      ; elster_form:einkommensteuer, elster_item:Spenden, elster_section:Sonderausgaben
      account expenses:charity:local
      account expenses:charity:international  ; elster_item:Internationale Hilfe

      2024-01-10 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2024-01-20 Non-deductible business penalty
          expenses:business:penalty   40.00 EUR
          assets:bank:business       -40.00 EUR

      2024-02-01 Local donation
          expenses:charity:local   50.00 EUR
          assets:bank:private     -50.00 EUR

      2024-02-02 International donation
          expenses:charity:international   30.00 EUR
          assets:bank:private             -30.00 EUR
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
      | # Arbeitsmittel                                             |        |
      | Nicht abzugsfähige Betriebsausgabe                          | 0.00   |
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
    And the CSV file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2024-01  | 100.00           | 19.00                     | 0.00                        | 19.00              |                    |
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
      | 2024 Q1  | 100.00           | 19.00                     | 0.00                        | 19.00              |                    |
      | 2024 Q2  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q4  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024     | 100.00           | 19.00                     | 0.00                        | 19.00              | 0.00               |
    And the CSV file "export/2024/steuererklaerung/einkommensteuer.csv" should contain exactly:
      | Kennzahl             | 2024  |
      | # Sonderausgaben     |       |
      | Internationale Hilfe | 30.00 |
      | Spenden              | 50.00 |

  Scenario: Expense and input VAT shares can differ for hospitality
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:business:hospitality  ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.70, elster_vat:contains_vat, elster_vat_rate:0.19, elster_input_vat_share:1.00, elster_item:Bewirtung, elster_section:Bewirtungskosten

      2024-04-10 Business dinner
          expenses:business:hospitality   119.00 EUR
          assets:bank:business           -119.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2024   |
      | # Betriebseinnahmen                                         |        |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 0.00   |
      | Vereinnahmte Umsatzsteuer                                   | 0.00   |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00   |
      | Summe Betriebseinnahmen                                     | 0.00   |
      |                                                             |        |
      | # Betriebsausgaben                                          |        |
      | # Bewirtungskosten                                          |        |
      | Bewirtung                                                   | 70.00  |
      |                                                             |        |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00   |
      | Summe Betriebskosten                                        | 70.00  |
      | Summe Betriebsausgaben                                      | 70.00  |
      |                                                             |        |
      | # Ermittlung des Gewinns                                    |        |
      | Steuerpflichtiger Gewinn/Verlust                            | -70.00 |
      |                                                             |        |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |        |
      | Entnahmen                                                   | 0.00   |
      | Einlagen                                                    | 0.00   |
    And the CSV file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2024-01  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-02  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-03  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-04  | 0.00             | 0.00                      | 19.00                       | -19.00             |                    |
      | 2024-05  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-06  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-07  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-08  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-09  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-10  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-11  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024-12  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q1  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q2  | 0.00             | 0.00                      | 19.00                       | -19.00             |                    |
      | 2024 Q3  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024 Q4  | 0.00             | 0.00                      | 0.00                        | 0.00               |                    |
      | 2024     | 0.00             | 0.00                      | 19.00                       | -19.00             | 0.00               |
