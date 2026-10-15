# VAT Payments And Settlements

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

VAT payment accounts use role metadata instead of form metadata:

- `elster_role:vat_advance` marks VAT advance payments and reversals.
- `elster_period` assigns an advance payment to the VAT period it settles, which
  can differ from the booking year. Supported values are `YYYY`, `YYYY-Qn`, and
  `YYYY-MM`.
- `elster_role:vat_payment` marks final VAT payments or refunds. These affect the
  EÜR in the booking year, but they are not part of the "Bereits Entrichtet" sum
  for the USt year being settled unless they are tagged as an advance for that
  period.

## Shared setup

Every scenario in this chapter disables the Home-Office-Pauschale, so the
figures show only what the journal contributes. [`setup.bash`](setup.bash) builds this state:

```scrut
$ build_scenario scenario_vat
```

```scrut
$ cat elster.toml
[euer.home_office_pauschale]
enabled = false
```

## VAT advance payments can be assigned by year, quarter, or month cadence

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance, elster_item:USt-Vorauszahlung
> account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
> account expenses:taxes:umsatzsteuer:vorauszahlung:2025:q1  ; elster_period:2025-Q1
> account expenses:taxes:umsatzsteuer:vorauszahlung:2026:march  ; elster_period:2026-03
> 
> 2024-02-15 VAT advance for 2024
>     expenses:taxes:umsatzsteuer:vorauszahlung:2024  100.00 EUR
>     assets:bank:business                           -100.00 EUR
> 
> 2024-03-01 VAT advance reversal for 2024
>     expenses:taxes:umsatzsteuer:vorauszahlung:2024  -40.00 EUR
>     assets:bank:business                             40.00 EUR
> 
> 2025-04-10 VAT advance for 2025 Q1
>     expenses:taxes:umsatzsteuer:vorauszahlung:2025:q1   30.00 EUR
>     assets:bank:business                              -30.00 EUR
> 
> 2026-04-11 VAT advance for March 2026
>     expenses:taxes:umsatzsteuer:vorauszahlung:2026:march   20.00 EUR
>     assets:bank:business                                 -20.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
  2025
  2026
```

```scrut
$ cat export/2024/steuererklaerung/umsatzsteuer.csv
Zeitraum;Einnahme (Netto);Vereinnahmte Umsatzsteuer;Abziehbare Vorsteuerbeträge;Vorauszahlungssoll;Bereits Entrichtet
2024-01;0,00;0,00;0,00;0,00;
2024-02;0,00;0,00;0,00;0,00;
2024-03;0,00;0,00;0,00;0,00;
2024-04;0,00;0,00;0,00;0,00;
2024-05;0,00;0,00;0,00;0,00;
2024-06;0,00;0,00;0,00;0,00;
2024-07;0,00;0,00;0,00;0,00;
2024-08;0,00;0,00;0,00;0,00;
2024-09;0,00;0,00;0,00;0,00;
2024-10;0,00;0,00;0,00;0,00;
2024-11;0,00;0,00;0,00;0,00;
2024-12;0,00;0,00;0,00;0,00;
2024 Q1;0,00;0,00;0,00;0,00;
2024 Q2;0,00;0,00;0,00;0,00;
2024 Q3;0,00;0,00;0,00;0,00;
2024 Q4;0,00;0,00;0,00;0,00;
2024;0,00;0,00;0,00;0,00;60,00
```

```scrut
$ cat export/2025/steuererklaerung/umsatzsteuer.csv
Zeitraum;Einnahme (Netto);Vereinnahmte Umsatzsteuer;Abziehbare Vorsteuerbeträge;Vorauszahlungssoll;Bereits Entrichtet
2025-01;0,00;0,00;0,00;0,00;
2025-02;0,00;0,00;0,00;0,00;
2025-03;0,00;0,00;0,00;0,00;
2025-04;0,00;0,00;0,00;0,00;
2025-05;0,00;0,00;0,00;0,00;
2025-06;0,00;0,00;0,00;0,00;
2025-07;0,00;0,00;0,00;0,00;
2025-08;0,00;0,00;0,00;0,00;
2025-09;0,00;0,00;0,00;0,00;
2025-10;0,00;0,00;0,00;0,00;
2025-11;0,00;0,00;0,00;0,00;
2025-12;0,00;0,00;0,00;0,00;
2025 Q1;0,00;0,00;0,00;0,00;30,00
2025 Q2;0,00;0,00;0,00;0,00;
2025 Q3;0,00;0,00;0,00;0,00;
2025 Q4;0,00;0,00;0,00;0,00;
2025;0,00;0,00;0,00;0,00;30,00
```

```scrut
$ cat export/2026/steuererklaerung/umsatzsteuer.csv
Zeitraum;Einnahme (Netto);Vereinnahmte Umsatzsteuer;Abziehbare Vorsteuerbeträge;Vorauszahlungssoll;Bereits Entrichtet
2026-01;0,00;0,00;0,00;0,00;
2026-02;0,00;0,00;0,00;0,00;
2026-03;0,00;0,00;0,00;0,00;20,00
2026-04;0,00;0,00;0,00;0,00;
2026-05;0,00;0,00;0,00;0,00;
2026-06;0,00;0,00;0,00;0,00;
2026-07;0,00;0,00;0,00;0,00;
2026-08;0,00;0,00;0,00;0,00;
2026-09;0,00;0,00;0,00;0,00;
2026-10;0,00;0,00;0,00;0,00;
2026-11;0,00;0,00;0,00;0,00;
2026-12;0,00;0,00;0,00;0,00;
2026 Q1;0,00;0,00;0,00;0,00;
2026 Q2;0,00;0,00;0,00;0,00;
2026 Q3;0,00;0,00;0,00;0,00;
2026 Q4;0,00;0,00;0,00;0,00;
2026;0,00;0,00;0,00;0,00;20,00
```

## More scenarios

These start from the same shared setup:

- [EÜR follows the booking year while USt settles against the tax period](booking-year-and-tax-period.md)
- [VAT refund from the Finanzamt is part of the profit](vat-refund.md)
