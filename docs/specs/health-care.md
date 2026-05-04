# Health Care And Insurance

Private health care and insurance payments are Einkommensteuer rows. The account
contract is:

- `elster_form:einkommensteuer` places the account in the ESt export.
- `elster_section:Vorsorgeaufwand` groups the labels under the Vorsorgeaufwand
  section.
- `elster_label` becomes the visible row name in the CSV.
- `elster_deduction:nicht_abzugsfaehig` keeps the booked payments visible while
  excluding them from the calculated deductible totals.

```gherkin
Feature: Health care and insurance

  Scenario: Non-deductible Vorsorgeaufwand rows are listed but not included in deductible totals
    Given a file named "journal.journal" with content:
      """
      account assets:bank:checking  ; elster_account:private, elster_label:Girokonto
      account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Krankenversicherung, elster_section:Vorsorgeaufwand
      account expenses:private:health-care:pv  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Pflegeversicherung, elster_section:Vorsorgeaufwand
      account expenses:private:health-care:zb  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Zusatzbeitrag, elster_section:Vorsorgeaufwand
      account expenses:insurance:travel:long-term-health-care  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Langzeit-Auslandskrankenversicherung, elster_section:Vorsorgeaufwand
      account expenses:insurance:travel:short-term-health-care  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Kurzzeit-Auslandskrankenversicherung, elster_section:Vorsorgeaufwand
      account expenses:insurance:liability:haftpflicht  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_label:Haftpflichtversicherung, elster_section:Vorsorgeaufwand

      2024-06-01 Health care contribution
          expenses:private:health-care:kv   840.00 EUR
          expenses:private:health-care:pv   240.00 EUR
          expenses:private:health-care:zb   120.00 EUR
          assets:bank:checking            -1200.00 EUR

      2024-06-08 Long-term travel health care
          expenses:insurance:travel:long-term-health-care   343.50 EUR
          assets:bank:checking                              -343.50 EUR

      2024-06-10 Short-term travel health care
          expenses:insurance:travel:short-term-health-care   9.50 EUR
          assets:bank:checking                               -9.50 EUR

      2024-06-15 Liability insurance
          expenses:insurance:liability:haftpflicht   57.88 EUR
          assets:bank:checking                       -57.88 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    Then the file "export/2024/steuererklaerung/einkommensteuer.csv" should contain exactly:
      """
      Kennzahl,2024
      # Vorsorgeaufwand,
      Haftpflichtversicherung,57.88
      Krankenversicherung,840.00
      Kurzzeit-Auslandskrankenversicherung,9.50
      Langzeit-Auslandskrankenversicherung,343.50
      Pflegeversicherung,240.00
      Zusatzbeitrag,120.00
      ,
      Summe privat gezahlt,1610.88
      Abziehbar (Netto),0.00
      Gezahlte Vorsteuer,0.00
      Abziehbare Vorsteuer,0.00
      Summe abziehbar,0.00
      ,
      """
```
