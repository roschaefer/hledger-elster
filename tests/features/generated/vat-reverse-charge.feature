# Generated from docs/specs/vat-reverse-charge.md
# Run: python scripts/generate_features.py

Feature: VAT reverse charge

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Foreign service invoices create reverse-charge VAT and matching input VAT
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:business:hosting  ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:contains_vat, elster_vat_rate:0.19, elster_section:Bezogene Fremdleistungen
      account expenses:business:hosting:eu  ; elster_item:EU Hosting, elster_vat:reverse_charge_eu
      account expenses:business:hosting:non_eu  ; elster_item:Non-EU Hosting, elster_vat:reverse_charge_non_eu

      2026-03-15 EU SaaS invoice
          expenses:business:hosting:eu   100.00 EUR
          assets:bank:business          -100.00 EUR

      2026-03-16 US SaaS invoice
          expenses:business:hosting:non_eu    50.00 EUR
          assets:bank:business              -50.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2026/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2026    |
      | # Betriebseinnahmen                                         |         |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 0.00    |
      | Vereinnahmte Umsatzsteuer                                   | 0.00    |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00    |
      | Summe Betriebseinnahmen                                     | 0.00    |
      |                                                             |         |
      | # Betriebsausgaben                                          |         |
      | # Bezogene Fremdleistungen                                  |         |
      | EU Hosting                                                  | 100.00  |
      | Non-EU Hosting                                              | 50.00   |
      |                                                             |         |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00    |
      | Summe Betriebskosten                                        | 150.00  |
      | Summe Betriebsausgaben                                      | 150.00  |
      |                                                             |         |
      | # Ermittlung des Gewinns                                    |         |
      | Steuerpflichtiger Gewinn/Verlust                            | -150.00 |
      |                                                             |         |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |         |
      | Entnahmen                                                   | 0.00    |
      | Einlagen                                                    | 0.00    |
    And the CSV file "export/2026/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      | Zeitraum | Einnahme (Netto) | Vereinnahmte Umsatzsteuer | §13b EU Leistung (Netto) | §13b EU Umsatzsteuer | §13b Non-EU Leistung (Netto) | §13b Non-EU Umsatzsteuer | Abziehbare Vorsteuerbeträge | Vorauszahlungssoll | Bereits Entrichtet |
      | 2026-01  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-02  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-03  | 0.00             | 0.00                      | 100.00                   | 19.00                | 50.00                        | 9.50                     | 28.50                       | 0.00               |                    |
      | 2026-04  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-05  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-06  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-07  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-08  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-09  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-10  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-11  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026-12  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026 Q1  | 0.00             | 0.00                      | 100.00                   | 19.00                | 50.00                        | 9.50                     | 28.50                       | 0.00               |                    |
      | 2026 Q2  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026 Q3  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026 Q4  | 0.00             | 0.00                      | 0.00                     | 0.00                 | 0.00                         | 0.00                     | 0.00                        | 0.00               |                    |
      | 2026     | 0.00             | 0.00                      | 100.00                   | 19.00                | 50.00                        | 9.50                     | 28.50                       | 0.00               | 0.00               |
