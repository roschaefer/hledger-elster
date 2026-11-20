# Income Tax Payments

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Income tax paid during the year is listed at the top of the ESt export, one row
per account:

- `elster_role:income_tax_advance` marks Vorauszahlungen.
- `elster_role:income_tax_final` marks the Abschlusszahlung for an earlier year.
- `elster_item` becomes the visible row name in the CSV.

Each row counts only the postings that carry its role, so an account with a
different role nested below it gets a row of its own and is not counted twice.

### A final payment below an advance account is counted once

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:private      ; elster_account:private, elster_item:Girokonto
> account expenses:est:va          ; elster_role:income_tax_advance, elster_item:ESt-VZ
> account expenses:est:va:final    ; elster_role:income_tax_final, elster_item:ESt-Abschluss
> 
> 2024-03-10 ESt-VZ
>     expenses:est:va         300.00 EUR
>     assets:bank:private
> 
> 2024-09-10 ESt-Abschluss
>     expenses:est:va:final    50.00 EUR
>     assets:bank:private
> EOF
```

```scrut
$ hledger elster -f journal.journal -o export
  2024
```

```scrut
$ csv_table export/2024/steuererklaerung/einkommensteuer.csv
Kennzahl      | 2024
ESt-VZ        | 300,00
ESt-Abschluss | 50,00
              |
```

### Sub-accounts with the same role add up to their parent's row

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:private      ; elster_account:private, elster_item:Girokonto
> account expenses:est:va          ; elster_role:income_tax_advance, elster_item:ESt-Vorauszahlung
> account expenses:est:va:q2
> 
> 2024-03-10 ESt-VZ Q1
>     expenses:est:va         200.00 EUR
>     assets:bank:private
> 
> 2024-06-10 ESt-VZ Q2
>     expenses:est:va:q2      200.00 EUR
>     assets:bank:private
> EOF
```

```scrut
$ hledger elster -f journal.journal -o export
  2024
```

```scrut
$ csv_table export/2024/steuererklaerung/einkommensteuer.csv
Kennzahl          | 2024
ESt-Vorauszahlung | 400,00
                  |
```
