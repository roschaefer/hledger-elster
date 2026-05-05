# Form, Section, And Item Tags

The three core routing tags define where a posting appears in the generated
ELSTER-oriented CSV files:

- `elster_form:einnahmenueberschussrechnung` routes business income and expenses
  into the EÜR export. The USt export is derived from these EÜR postings when
  they carry VAT metadata such as `elster_vat_rate`, plus VAT payment roles.
- `elster_form:einkommensteuer` routes private tax-relevant expenses into the
  ESt export.
- `elster_section` is a user-defined heading inside a form. The tool preserves
  the section name but does not interpret it.
- `elster_item` is the report row. It can translate a technical account name into
  a user-facing row name and it also controls aggregation: child accounts inherit
  the closest parent item unless they define their own item.

```gherkin
Feature: Form, section, and item tags

  Scenario: Form tags select exports and item tags translate or aggregate accounts
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account assets:bank:private   ; elster_account:private, elster_item:Girokonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
      account expenses:charity      ; elster_form:einkommensteuer, elster_item:Spenden, elster_section:Sonderausgaben
      account expenses:charity:local
      account expenses:charity:international  ; elster_item:Internationale Hilfe

      2024-01-10 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2024-02-01 Local donation
          expenses:charity:local   50.00 EUR
          assets:bank:private     -50.00 EUR

      2024-02-02 International donation
          expenses:charity:international   30.00 EUR
          assets:bank:private             -30.00 EUR
      """
    And a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      """
      Kennzahl,2024
      # Betriebseinnahmen,
      Umsatzsteuerpflichtige Betriebseinnahmen,100.00
      Vereinnahmte Umsatzsteuer,19.00
      Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebseinnahmen,119.00
      ,
      # Betriebsausgaben,
      ,
      An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebskosten,0.00
      Summe Betriebsausgaben,0.00
      ,
      # Ermittlung des Gewinns,
      Steuerpflichtiger Gewinn/Verlust,119.00
      ,
      # Zusätzliche Angaben bei Einzelunternehmen,
      Entnahmen,0.00
      Einlagen,0.00
      """
    And the file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      """
      Zeitraum,Einnahme (Netto),Vereinnahmte Umsatzsteuer,Abziehbare Vorsteuerbeträge,Vorauszahlungssoll,Bereits Entrichtet
      2024-01,100.00,19.00,0.00,19.00,
      2024-02,0.00,0.00,0.00,0.00,
      2024-03,0.00,0.00,0.00,0.00,
      2024-04,0.00,0.00,0.00,0.00,
      2024-05,0.00,0.00,0.00,0.00,
      2024-06,0.00,0.00,0.00,0.00,
      2024-07,0.00,0.00,0.00,0.00,
      2024-08,0.00,0.00,0.00,0.00,
      2024-09,0.00,0.00,0.00,0.00,
      2024-10,0.00,0.00,0.00,0.00,
      2024-11,0.00,0.00,0.00,0.00,
      2024-12,0.00,0.00,0.00,0.00,
      2024 Q1,100.00,19.00,0.00,19.00,
      2024 Q2,0.00,0.00,0.00,0.00,
      2024 Q3,0.00,0.00,0.00,0.00,
      2024 Q4,0.00,0.00,0.00,0.00,
      2024,100.00,19.00,0.00,19.00,0.00
      """
    And the file "export/2024/steuererklaerung/einkommensteuer.csv" should contain exactly:
      """
      Kennzahl,2024
      # Sonderausgaben,
      Internationale Hilfe,30.00
      Spenden,50.00
      ,
      Summe privat gezahlt,80.00
      Abziehbar (Netto),80.00
      Gezahlte Vorsteuer,0.00
      Abziehbare Vorsteuer,0.00
      Summe abziehbar,80.00
      ,
      """
```
