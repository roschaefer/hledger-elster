# A dirty repository cannot export commit evidence

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

Starts from the chapter's [shared setup](README.md#shared-setup), which
[`setup.bash`](setup.bash) builds:

```scrut
$ build_scenario scenario_commit_evidence
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> 
> 2024-01-10 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> EOF
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf
Error: working tree has uncommitted changes; commit or stash them before exporting commit evidence
[1]
```

```scrut
$ test ! -e ../commit-evidence.pdf
```
