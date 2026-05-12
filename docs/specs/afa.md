# GWG And AfA

Business assets tagged with `elster_afa_years` are classified by net acquisition
cost:

- net cost up to and including `800.00 EUR` is treated as a GWG and deducted in
  full in the purchase year
- net cost above `800.00 EUR` is depreciated with straight-line AfA over
  `elster_afa_years`

The optional German `Sammelposten` treatment for assets between `250.00 EUR` and
`1000.00 EUR` is not implemented. The tool intentionally keeps the rule simple:
GWG up to `800.00 EUR`, AfA above `800.00 EUR`.

Since 2021, computers, laptops, and software can be modeled as digital AfA with a
useful life of one year. In this tool that means `elster_afa_years:1`. AfA is
calculated monthly, so a purchase on `2024-01-01` is fully depreciated in 2024.

```gherkin
Feature: GWG and AfA

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Digital AfA deducts a laptop over one year
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:business:hardware:laptop  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_afa_years:1, elster_item:Laptop, elster_section:Arbeitsmittel

      2024-01-01 Laptop digital AfA
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
      | AfA Laptop                                                  | 1200.00  |
      |                                                             |          |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00     |
      | Summe Betriebskosten                                        | 0.00     |
      | Summe Betriebsausgaben                                      | 1200.00  |
      |                                                             |          |
      | # Ermittlung des Gewinns                                    |          |
      | Steuerpflichtiger Gewinn/Verlust                            | -1200.00 |
      |                                                             |          |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |          |
      | Entnahmen                                                   | 0.00     |
      | Einlagen                                                    | 0.00     |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-laptop.csv" should contain exactly:
      | Beschreibung       | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 |
      | Laptop digital AfA | 2024-01-01 | 1200.00           | 1         | 100.00     | 1200.00  |
      | GESAMT             |            |                   |           |            | 1200.00  |

  Scenario: A larger non-digital business asset is depreciated over several years
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:business:hardware:camera  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_afa_years:7, elster_item:Kamera, elster_section:Arbeitsmittel

      2024-01-01 Camera AfA
          expenses:business:hardware:camera   1400.00 EUR
          assets:bank:business               -1400.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      | Kennzahl                                                    | 2024    |
      | # Betriebseinnahmen                                         |         |
      | Umsatzsteuerpflichtige Betriebseinnahmen                    | 0.00    |
      | Vereinnahmte Umsatzsteuer                                   | 0.00    |
      | Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer  | 0.00    |
      | Summe Betriebseinnahmen                                     | 0.00    |
      |                                                            |         |
      | # Betriebsausgaben                                          |         |
      | # Arbeitsmittel                                             |         |
      | AfA Kamera                                                  | 200.00  |
      |                                                            |         |
      | An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer | 0.00    |
      | Summe Betriebskosten                                        | 0.00    |
      | Summe Betriebsausgaben                                      | 200.00  |
      |                                                            |         |
      | # Ermittlung des Gewinns                                    |         |
      | Steuerpflichtiger Gewinn/Verlust                            | -200.00 |
      |                                                            |         |
      | # Zusätzliche Angaben bei Einzelunternehmen                 |         |
      | Entnahmen                                                   | 0.00    |
      | Einlagen                                                    | 0.00    |
    And the CSV file "export/2024/herleitung/einnahmen-ueberschuss-rechnung/afa-kamera.csv" should contain exactly:
      | Beschreibung | Kaufdatum  | Kaufpreis (Netto) | AfA-Jahre | Monat. AfA | AfA 2024 |
      | Camera AfA   | 2024-01-01 | 1400.00           | 7         | 16.67      | 200.00   |
      | GESAMT       |            |                   |           |            | 200.00   |
```
