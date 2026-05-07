# Generated from docs/specs/afa.md
# Run: python scripts/generate_features.py

Feature: GWG and AfA

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Low-value assets are deducted immediately and larger assets are depreciated
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:business:hardware:monitor  ; elster_form:einnahmenueberschussrechnung, elster_afa_years:3, elster_item:Monitor, elster_section:Arbeitsmittel
      account expenses:business:hardware:laptop  ; elster_form:einnahmenueberschussrechnung, elster_afa_years:3, elster_item:Laptop, elster_section:Arbeitsmittel

      2024-01-10 Office monitor GWG
          expenses:business:hardware:monitor   800.00 EUR
          assets:bank:business                -800.00 EUR

      2024-07-01 Laptop AfA
          expenses:business:hardware:laptop   1200.00 EUR
          assets:bank:business               -1200.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2024     |
      | # Betriebseinnahmen                                         |          |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 0.00     |
      | Vereinnahmte Umsatzsteuer                                   | 0.00     |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00     |
      | Summe Betriebseinnahmen                                     | 0.00     |
      |                                                             |          |
      | # Betriebsausgaben                                          |          |
      | # Arbeitsmittel                                             |          |
      | Monitor                                                     | 800.00   |
      | # Arbeitsmittel                                             |          |
      | AfA Laptop                                                  | 200.00   |
      |                                                             |          |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00     |
      | Summe Betriebskosten                                        | 800.00   |
      | Summe Betriebsausgaben                                      | 1000.00  |
      |                                                             |          |
      | # Ermittlung des Gewinns                                    |          |
      | Steuerpflichtiger Gewinn/Verlust                            | -1000.00 |
      |                                                             |          |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |          |
      | Entnahmen                                                   | 0.00     |
      | Einlagen                                                    | 0.00     |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/monitor.csv" should contain exactly:
      | Konto             | Datum      | Beschreibung       | Brutto | Netto  | Anteil | Abziehbar |
      | Geschäftskonto    | 2024-01-10 | Office monitor GWG | 800.00 | 800.00 | 1.00   | 800.00    |
      | Σ Geschäftskonto  |            |                    | 800.00 | 800.00 |        | 800.00    |
      | GESAMT            |            |                    | 800.00 | 800.00 |        | 800.00    |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-laptop.csv" should contain exactly:
      | Beschreibung | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 |
      | Laptop AfA   | 2024-07-01 | 1200.00           | 3         | 33.33      | 200.00   |
      | GESAMT       |            |                   |           |            | 200.00   |
