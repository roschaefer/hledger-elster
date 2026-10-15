# Health Care And Insurance

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

Private health care and insurance payments are Einkommensteuer rows. The account
contract is:

- `elster_form:einkommensteuer` places the account in the ESt export.
- `elster_section:Vorsorgeaufwand` groups the labels under the Vorsorgeaufwand
  section.
- `elster_item` becomes the visible row name in the CSV.

### Vorsorgeaufwand rows are listed for the income tax declaration

```scrut
$ new_scenario
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:checking  ; elster_account:private, elster_item:Girokonto
> account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_item:Krankenversicherung, elster_section:Vorsorgeaufwand
> account expenses:private:health-care:pv  ; elster_form:einkommensteuer, elster_item:Pflegeversicherung, elster_section:Vorsorgeaufwand
> account expenses:private:health-care:zb  ; elster_form:einkommensteuer, elster_item:Zusatzbeitrag, elster_section:Vorsorgeaufwand
> account expenses:insurance:travel:long-term-health-care  ; elster_form:einkommensteuer, elster_item:Langzeit-Auslandskrankenversicherung, elster_section:Vorsorgeaufwand
> account expenses:insurance:travel:short-term-health-care  ; elster_form:einkommensteuer, elster_item:Kurzzeit-Auslandskrankenversicherung, elster_section:Vorsorgeaufwand
> account expenses:insurance:liability:haftpflicht  ; elster_form:einkommensteuer, elster_item:Haftpflichtversicherung, elster_section:Vorsorgeaufwand
> 
> 2024-06-01 Health care contribution
>     expenses:private:health-care:kv   840.00 EUR
>     expenses:private:health-care:pv   240.00 EUR
>     expenses:private:health-care:zb   120.00 EUR
>     assets:bank:checking            -1200.00 EUR
> 
> 2024-06-08 Long-term travel health care
>     expenses:insurance:travel:long-term-health-care   343.50 EUR
>     assets:bank:checking                              -343.50 EUR
> 
> 2024-06-10 Short-term travel health care
>     expenses:insurance:travel:short-term-health-care   9.50 EUR
>     assets:bank:checking                               -9.50 EUR
> 
> 2024-06-15 Liability insurance
>     expenses:insurance:liability:haftpflicht   57.88 EUR
>     assets:bank:checking                       -57.88 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
```

```scrut
$ cat export/2024/steuererklaerung/einkommensteuer.csv
Kennzahl;2024
# Vorsorgeaufwand;
Haftpflichtversicherung;57,88
Krankenversicherung;840,00
Kurzzeit-Auslandskrankenversicherung;9,50
Langzeit-Auslandskrankenversicherung;343,50
Pflegeversicherung;240,00
Zusatzbeitrag;120,00
```
