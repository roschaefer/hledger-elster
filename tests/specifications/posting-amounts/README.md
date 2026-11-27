# Posting amounts must be in EUR

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The export accepts one EUR amount per posting, also written as `€` or without
a commodity. Foreign amounts must be recorded in EUR before exporting; a
conversion price on a foreign amount does not change its commodity.

Foreign currencies and mixed-commodity postings are rejected with their source
location. Unsupported decimal precision also returns an error.

## A foreign amount with a EUR price is rejected

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> 2024-01-10 Hosting invoice
>     expenses:hosting  100 USD @ 0.92 EUR
>     assets:bank
> EOF
```

```scrut
$ hledger elster -f journal.journal -o export
Error: Unsupported amount for account "expenses:hosting" on 2024-01-10 ("Hosting invoice", *journal.journal:1): commodity "USD" is not EUR; record the posting in EUR before exporting (glob)
[1]
```

```scrut
$ test ! -d export
```

## A mixed balancing amount cannot be partly dropped

The inferred bank posting contains both commodities. Even when its first
amount is EUR, the export rejects the posting rather than dropping the rest.

```scrut
$ new_scenario
```

```scrut
$ write_file journal.journal <<'EOF'
> 2024-01-10 Mixed payment
>     assets:bank
>     expenses:hosting  10 EUR
>     expenses:other  20 USD
> EOF
```

```scrut
$ hledger elster -f journal.journal -o export
Error: Unsupported amount for account "assets:bank" on 2024-01-10 ("Mixed payment", *journal.journal:1): expected a single EUR amount, found 2 amounts; split the posting into EUR postings (glob)
[1]
```

```scrut
$ test ! -d export
```
