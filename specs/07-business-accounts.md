# Business Vs. Private Accounts

Business income is identified by the combination of account metadata and the
counterposting account:

- `elster_form:einnahmenueberschussrechnung` marks business income or expenses
  for the EÜR.
- `elster_vat:contains_vat, elster_vat_rate:0.19` splits gross income
  into net income and collected VAT.
- `elster_account:business` identifies the bank account that makes owner draws
  and contributions visible even when the other posting uses an untagged owner
  equity account.
- Unclassified outflows from a business account are treated as owner draws. Use
  `elster_role:ignore` on internal funding movements when the actual business
  expense is booked in a separate transaction.

In the EÜR output, `Entnahmen` are owner draws from the business sphere into the
private sphere. `Einlagen` are owner contributions into the business sphere.

```gherkin
Feature: Business vs. private accounts

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Business account postings classify income, owner draws, and owner contributions
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account assets:bank:private   ; elster_account:private, elster_item:Girokonto
      account transfers:clearing
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

      2024-09-03 Internal transfer  ; elster_role:ignore
          transfers:clearing      75.00 EUR
          assets:bank:business   -75.00 EUR
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
```
