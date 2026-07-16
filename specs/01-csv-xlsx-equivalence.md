# CSV/xlsx equivalence

A German tax audit (`Steuerprüfung`) expects to see the calculation, not just
the filed number. In my own audit, the tax office asked for a file they could
open in LibreOffice or Microsoft Office — a spreadsheet, not a printout — so
they could follow how each figure was derived. That's why this tool produces
`.xlsx` workbooks.

An `.xlsx` file is a binary zip archive, which makes it a poor fit for a
journal kept under version control: you can't meaningfully `git diff` it, and
small changes produce large, opaque commits. So every workbook has a
plain-text CSV sibling generated from the exact same computed values. This
follows the same reasoning as [Plain Text
Accounting](https://plaintextaccounting.org/What-is-Plain-Text-Accounting):
data you can diff, grep, and version is easier to trust than data you can't.
Together the two formats satisfy both requirements — auditable by the tax
office, diffable by you:

- `steuererklaerung.xlsx` has one tab per form (`EÜR`, `USt`, `ESt`); each tab
  has a matching CSV under `steuererklaerung/`.
- Each `herleitung/<form>.xlsx` workbook has one tab per Herleitung sheet;
  each tab has a matching CSV under `herleitung/<form>/`.

There is intentionally only one computation path to get there, so the two
representations cannot drift apart. One documented exception: a
`"# "`-prefixed `Kennzahl` marks a section header. The CSV keeps the literal
`"# "` prefix as plain text; the xlsx strips it and renders the row bold
instead, since it has actual styling available to convey that meaning.
`"GESAMT"`- and `"Σ "`-prefixed rows carry no such exception — their text is
identical in both formats, only the fill/bold formatting differs.

```gherkin
Feature: CSV/xlsx equivalence

  Background:
    Given a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """

  Scenario: Every form tab matches its CSV export, across EÜR, USt, and ESt
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account assets:bank:private   ; elster_account:private, elster_item:Girokonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
      account expenses:business:hosting  ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:contains_vat, elster_vat_rate:0.19, elster_input_vat_share:1.00, elster_item:Serverkosten, elster_section:Bezogene Fremdleistungen
      account expenses:taxes:umsatzsteuer:vorauszahlung       ; elster_role:vat_advance
      account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
      account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_item:Krankenversicherung, elster_section:Vorsorgeaufwand

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2024-02-01 Hosting
          expenses:business:hosting   23.80 EUR
          assets:bank:business       -23.80 EUR

      2024-03-01 VAT advance
          expenses:taxes:umsatzsteuer:vorauszahlung:2024   19.00 EUR
          assets:bank:business                            -19.00 EUR

      2024-06-01 Health care contribution
          expenses:private:health-care:kv   200.00 EUR
          assets:bank:private              -200.00 EUR
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then the xlsx file "export/2024/steuererklaerung.xlsx" tab "EÜR" should equal the CSV file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv"
    And the xlsx file "export/2024/steuererklaerung.xlsx" tab "USt" should equal the CSV file "export/2024/steuererklaerung/umsatzsteuer.csv"
    And the xlsx file "export/2024/steuererklaerung.xlsx" tab "ESt" should equal the CSV file "export/2024/steuererklaerung/einkommensteuer.csv"
```
