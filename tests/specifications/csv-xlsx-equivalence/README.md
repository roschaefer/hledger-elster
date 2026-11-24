# CSV/xlsx equivalence

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The `scrut` blocks below use helpers from [`readme-setup.sh`](../readme-setup.sh),
such as `new_scenario`, which starts each scenario in a new empty directory.

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

Both formats also render amounts with German number conventions -- `,` as the
decimal separator (`1190,00`, not `1190.00`) -- the same convention ELSTER's
own web forms use, so a figure can be copied straight out of either file and
pasted into ELSTER without manually swapping the separator first. A `,`
decimal collides with a `,` field delimiter, so every CSV in this project uses
`;` to separate fields instead. This is the standard German locale convention
for CSV: [Unicode CLDR](https://github.com/unicode-org/cldr-json/blob/main/cldr-json/cldr-numbers-full/main/de/numbers.json)
-- the reference locale-data repository underlying ICU, Windows, and most
platform number formatting -- defines, for `de`, `decimal: ","`, `group: "."`,
and `list: ";"`; that `list` symbol is what Excel and LibreOffice use as the
default CSV field separator, so a German-locale install of either already
writes and expects `;`-delimited, `,`-decimal CSVs.

There is intentionally only one computation path to get there, so the two
representations cannot drift apart. One documented exception: a
`"# "`-prefixed `Kennzahl` marks a section header. The CSV keeps the literal
`"# "` prefix as plain text; the xlsx strips it and renders the row bold
instead, since it has actual styling available to convey that meaning.
`"GESAMT"`- and `"Σ "`-prefixed rows carry no such exception — their text is
identical in both formats, only the fill/bold formatting differs.

### Every form tab matches its CSV export, across EÜR, USt, and ESt

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
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account assets:bank:private   ; elster_account:private, elster_item:Girokonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> account expenses:business:hosting  ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:contains_vat, elster_vat_rate:0.19, elster_input_vat_share:1.00, elster_item:Serverkosten, elster_section:Bezogene Fremdleistungen
> account expenses:taxes:umsatzsteuer:vorauszahlung       ; elster_role:vat_advance
> account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
> account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_item:Krankenversicherung, elster_section:Vorsorgeaufwand
> 
> 2024-01-15 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> 
> 2024-02-01 Hosting
>     expenses:business:hosting   23.80 EUR
>     assets:bank:business       -23.80 EUR
> 
> 2024-03-01 VAT advance
>     expenses:taxes:umsatzsteuer:vorauszahlung:2024   19.00 EUR
>     assets:bank:business                            -19.00 EUR
> 
> 2024-06-01 Health care contribution
>     expenses:private:health-care:kv   200.00 EUR
>     assets:bank:private              -200.00 EUR
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
```

```scrut
$ xlsx_sheet export/2024/steuererklaerung.xlsx EÜR | diff - <(strip_section_markers export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv)
```

```scrut
$ xlsx_sheet export/2024/steuererklaerung.xlsx USt | diff - <(strip_section_markers export/2024/steuererklaerung/umsatzsteuer.csv)
```

```scrut
$ xlsx_sheet export/2024/steuererklaerung.xlsx ESt | diff - <(strip_section_markers export/2024/steuererklaerung/einkommensteuer.csv)
```

### Colliding labels keep a separate workbook tab and CSV for every item

Each `elster_item` becomes one Herleitung tab in the workbook and one CSV
file next to it. Neither name can be the label verbatim: Excel forbids some
characters in tab names and compares them case-insensitively, and CSV file
names are lowercase ASCII slugs. So distinct labels can collapse into the same
name. This scenario books one expense on each of three such pairs:

| `elster_item` | Amount | Collides with   | Workbook tab  | CSV file        |
|---------------|-------:|-----------------|---------------|-----------------|
| `A-B`         |  60,00 |                 | `A-B`         | `a-b.csv`       |
| `A/B`         |  50,00 | `A-B` (`/`)     | `A-B (2)`     | `a-b-2.csv`     |
| `Buro`        |  20,00 |                 | `Buro`        | `buro.csv`      |
| `Büro`        |  10,00 | `Buro` (accent) | `Büro`        | `buro-2.csv`    |
| `Telefon`     |  30,00 |                 | `Telefon`     | `telefon.csv`   |
| `telefon`     |  40,00 | `Telefon` (case)| `telefon (2)` | `telefon-2.csv` |

Labels are taken in sorted order; the first keeps the plain name and each
later one gets a numeric suffix. Tab and CSV names are made unique
separately, so `Büro` needs a suffix only for its CSV.

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
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account expenses:a  ; elster_item:Büro
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
> account expenses:b  ; elster_item:Buro
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
> account expenses:c  ; elster_item:Telefon
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
> account expenses:d  ; elster_item:telefon
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
> account expenses:e  ; elster_item:A/B
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
> account expenses:f  ; elster_item:A-B
>   ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:not_applicable
>
> 2024-01-10 Distinct expenses
>     expenses:a  10.00 EUR
>     expenses:b  20.00 EUR
>     expenses:c  30.00 EUR
>     expenses:d  40.00 EUR
>     expenses:e  50.00 EUR
>     expenses:f  60.00 EUR
>     assets:bank:business
> EOF
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
```

All six items get their own CSV. Before this was fixed, `Büro` and `Buro`
both wrote `buro.csv`, and the second silently overwrote the first.

```scrut
$ ls export/2024/herleitung/einnahmen-ueberschuss-rechnung
a-b-2.csv
a-b.csv
buro-2.csv
buro.csv
telefon-2.csv
telefon.csv
```

The `GESAMT` row of each CSV shows which item landed in which file, matching
the amounts in the table above:

```scrut
$ (cd export/2024/herleitung/einnahmen-ueberschuss-rechnung && grep ^GESAMT *.csv)
a-b-2.csv:GESAMT;;;50,00;50,00;;50,00
a-b.csv:GESAMT;;;60,00;60,00;;60,00
buro-2.csv:GESAMT;;;10,00;10,00;;10,00
buro.csv:GESAMT;;;20,00;20,00;;20,00
telefon-2.csv:GESAMT;;;40,00;40,00;;40,00
telefon.csv:GESAMT;;;30,00;30,00;;30,00
```

```scrut
$ workbook=export/2024/herleitung/einnahmen-ueberschuss-rechnung.xlsx
```

```scrut
$ csv_dir=export/2024/herleitung/einnahmen-ueberschuss-rechnung
```

Next, each workbook tab is paired with its CSV. `xlsx_sheet` prints a tab
as `;`-separated text, and `diff -` compares that against the CSV file; no
output means they're identical. Since every item has a different amount, a
tab paired with the wrong CSV would show up as a diff. Each block below
therefore checks one row of the table: the item's tab and its CSV hold the
same data.

```scrut
$ xlsx_sheet "$workbook" A-B | diff - "$csv_dir/a-b.csv"
```

```scrut
$ xlsx_sheet "$workbook" 'A-B (2)' | diff - "$csv_dir/a-b-2.csv"
```

```scrut
$ xlsx_sheet "$workbook" Buro | diff - "$csv_dir/buro.csv"
```

```scrut
$ xlsx_sheet "$workbook" Büro | diff - "$csv_dir/buro-2.csv"
```

```scrut
$ xlsx_sheet "$workbook" Telefon | diff - "$csv_dir/telefon.csv"
```

```scrut
$ xlsx_sheet "$workbook" 'telefon (2)' | diff - "$csv_dir/telefon-2.csv"
```

Finally, the suffixes are deterministic: exporting the same journal again
assigns every item the same tab and CSV name, so the export doesn't churn
under version control. `diff -r` prints nothing when both exports are
identical.

```scrut
$ cp -r export previous-export
```

```scrut
$ hledger elster -f journal.journal --config elster.toml -o export
  2024
```

```scrut
$ diff -r previous-export export
```
