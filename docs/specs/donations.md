# Donations

Donation accounts are Einkommensteuer rows grouped by user-defined section
metadata. The account contract is:

- `elster_form:einkommensteuer` places the account in the ESt export.
- `elster_section:Sonderausgaben` groups the rows under the Sonderausgaben
  heading. The section name is user-defined; the tool does not hard-code a
  donation category.
- `elster_item` becomes the visible row name.
- Child accounts inherit the parent account metadata, so multiple charity
  subaccounts roll up to one `Spenden` row.
- `elster_calculation:manual` keeps a row visible as `MANUAL` and writes an audit
  trail, but excludes the amount from calculated ESt totals. This is used for
  cases such as political party donations where the tax treatment must be
  calculated outside the tool.

```gherkin
Feature: Donations

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Donations are exported to Einkommensteuer
    Given a file named "journal.journal" with content:
      """
      account assets:bank:checking  ; elster_account:private, elster_item:Girokonto
      account expenses:charity  ; elster_form:einkommensteuer, elster_item:Spenden, elster_section:Sonderausgaben
      account expenses:charity:example
      account expenses:charity:another
      account expenses:politics:party  ; elster_form:einkommensteuer, elster_item:Parteispende (§34g/§10b manuell berechnen), elster_section:Sonderausgaben, elster_calculation:manual

      2024-12-01 Example charity donation
          expenses:charity:example    50.00 EUR
          assets:bank:checking       -50.00 EUR

      2024-12-02 Another charity donation
          expenses:charity:another    25.00 EUR
          assets:bank:checking       -25.00 EUR

      2024-12-02 Example political party donation
          expenses:politics:party    100.00 EUR
          assets:bank:checking      -100.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the CSV file "export/2024/steuererklaerung/einkommensteuer.csv" should contain exactly:
      | Kennzahl                                   | 2024   |
      | # Sonderausgaben                           |        |
      | Parteispende (§34g/§10b manuell berechnen) | MANUAL |
      | Spenden                                    | 75.00  |
    And the CSV file "export/2024/herleitung/einkommensteuer/spenden.csv" should contain exactly:
      | Konto       | Datum      | Beschreibung             | Betrag |
      | Girokonto   | 2024-12-01 | Example charity donation | 50.00  |
      | Girokonto   | 2024-12-02 | Another charity donation | 25.00  |
      | Σ Girokonto |            |                          | 75.00  |
      | GESAMT      |            |                          | 75.00  |
    And the CSV file "export/2024/herleitung/einkommensteuer/parteispende-(§34g-§10b-manuell.csv" should contain exactly:
      | Konto       | Datum      | Beschreibung                     | Betrag |
      | Girokonto   | 2024-12-02 | Example political party donation | 100.00 |
      | Σ Girokonto |            |                                  | 100.00 |
      | GESAMT      |            |                                  | 100.00 |
```
