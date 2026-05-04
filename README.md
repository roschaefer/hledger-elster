# hledger-elster

`hledger-elster` generates German tax exports for `ELSTER.de` from an `hledger` journal.

Outputs are written under `data/exports/<year>/` by default:

- `steuererklaerung/`: summary CSV files plus `steuererklaerung.xlsx`
- `herleitung/`: detailed audit trails plus per-form `.xlsx`

Workbook export convention:

- every `name.xlsx` workbook has a sibling `name/` directory
- each workbook tab is also exported there as a CSV with the corresponding derived filename

Usage:

```bash
./hledger-elster
./hledger-elster -f examples/ledger/hledger.journal
./hledger-elster -f examples/ledger/hledger.journal -o /tmp/elster-out
```

Arguments:

- `-f`, `--file`: input journal, with the same meaning as `hledger -f`
- `-o`, `--output-dir`: output directory for generated tax artifacts

Typical development commands:

```bash
just test
```

Sanitized public fixtures live under [`examples/`](./examples). Keep real journals,
tax filings, and verification data outside this repository.

---

## Account tag reference

The tool reads account-level metadata from hledger account directives (`;` comments).
All tags are prefixed with `elster_`. Tags fall into three categories:

- **Routing tags** — which tax form an account belongs to and how it is grouped there
- **Calculation tags** — how amounts are transformed before they appear in the output
- **Infrastructure tags** — account classification used by the ingestion layer

### Example `accounts.journal`

```hledger
; ── Payment accounts ──────────────────────────────────────────────────────────
account assets:bank:business          ; elster_account:business, elster_label:Geschäftskonto
account assets:bank:private           ; elster_account:private,  elster_label:Girokonto

; ── Business income (EÜR + USt) ───────────────────────────────────────────────
account income:business               ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_label:Betriebseinnahmen

; ── Business expenses (EÜR) ───────────────────────────────────────────────────
; Base account sets defaults; sub-accounts inherit and override.
account expenses:business             ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat_rate:0.19, elster_vat_share:1.00
account expenses:business:hosting     ; elster_label:Serverkosten, elster_section:Bezogene Fremdleistungen
account expenses:business:education   ; elster_label:Fortbildung,  elster_section:Fortbildungskosten

; Proportional deduction (e.g. phone: 20 % business use)
account expenses:phone                ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.20, elster_vat_share:0.20, elster_vat_rate:0.19, elster_label:Mobiltelefon, elster_section:Arbeitsmittel

; Depreciable asset (AfA)
account expenses:business:hardware:computer ; elster_form:einnahmenueberschussrechnung, elster_afa_years:3, elster_label:Computer-Kauf, elster_section:Arbeitsmittel

; ── Private expenses (ESt) ────────────────────────────────────────────────────
account expenses:insurance            ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig
account expenses:insurance:health:kv  ; elster_label:Krankenversicherung,  elster_section:Vorsorgeaufwand
account expenses:insurance:health:pv  ; elster_label:Pflegeversicherung,   elster_section:Vorsorgeaufwand

; Donations are ordinary ESt accounts grouped by a user-defined section.
; Unlabelled child accounts inherit this label and are summed into one row.
account expenses:charity              ; elster_form:einkommensteuer, elster_label:Spenden, elster_section:Sonderausgaben
account expenses:charity:drk
account expenses:charity:unicef

; Use manual calculation for cases the tool should list but not calculate.
account expenses:politics:party       ; elster_form:einkommensteuer, elster_label:Parteispende - §34g/§10b manuell berechnen, elster_section:Sonderausgaben, elster_calculation:manual

; ── Tax payments ──────────────────────────────────────────────────────────────
account expenses:taxes:einkommensteuer:vorauszahlung    ; elster_role:income_tax_advance, elster_label:ESt-Vorauszahlung
account expenses:taxes:einkommensteuer:abschlusszahlung ; elster_role:income_tax_final,   elster_label:ESt-Abschlusszahlung
account expenses:taxes:umsatzsteuer:vorauszahlung       ; elster_role:vat_advance
account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
account expenses:taxes:umsatzsteuer:vorauszahlung:2025  ; elster_period:2025
```

---

### Routing tags

| Tag | Values | Meaning |
|-----|--------|---------|
| `elster_account` | `business` \| `private` | Marks a payment account as belonging to the business or private sphere. Drives the drawing/contribution fallback: any unclassified outflow from a `business` account is counted as an Entnahme in the EÜR. |
| `elster_role` | `income_tax_advance` \| `income_tax_final` | Marks ESt payment accounts. Postings appear in the ESt summary, separated by advance vs. final settlement. |
| `elster_role` | `tax_payment` | Generic parent role for all tax payments. Prevents tax outflows from being counted as Entnahmen in the EÜR. |
| `elster_role` | `vat_payment` | Marks USt Abschlusszahlung accounts. Postings flow into the EÜR (line 57: gezahlte Umsatzsteuer) and the USt report. |
| `elster_role` | `vat_advance` | Marks USt Vorauszahlung accounts. Requires `elster_period` on sub-accounts for correct fiscal-year attribution. |
| `elster_form` | `einnahmenueberschussrechnung` | Marks an account as belonging to the EÜR. Income accounts flow into Betriebseinnahmen; expense accounts flow into Betriebsausgaben. Net/VAT split is controlled by `elster_vat_rate`; deduction treatment by the calculation tags below. |
| `elster_form` | `einkommensteuer` | Marks an account as belonging to the ESt. The account appears under the user-defined `elster_section`; postings from a `business` source account are additionally counted as Entnahmen in the EÜR. |
| `elster_section` | free text | User-defined grouping within a form (for example `Sonderausgaben` for donations or `Arbeitsmittel` for EÜR expenses). The code does not interpret specific section names. |
| `elster_label` | free text | Human-readable label shown in the output instead of the account name. Inherited labels are used for grouping: child accounts without their own `elster_label` are summed into the parent label; child accounts with their own label appear separately. |
| `elster_period` | `YYYY` | On USt Vorauszahlung sub-accounts: the fiscal year the payment belongs to, regardless of when the transaction occurred. Required on every `vat_advance` sub-account. |

---

### Calculation tags

| Tag | Values | Meaning |
|-----|--------|---------|
| `elster_vat_rate` | `0.19` \| `0.07` \| `0.00` | VAT rate used to split gross amounts into net + VAT. On income accounts, determines collected VAT. On expense accounts, determines deductible input VAT (subject to `elster_vat_share`). |
| `elster_deduction` | `full` | The full net amount is deductible as a business expense. |
| `elster_deduction` | `proportional` | Only the business-use fraction is deductible. Set `elster_expense_share` and `elster_vat_share` to the business-use percentage. |
| `elster_deduction` | `nicht_abzugsfaehig` | Not deductible as a business expense (e.g. private insurance). The gross amount is still reported on the ESt form. |
| `elster_deduction` | `afa` | Triggers straight-line depreciation. Set `elster_afa_years` to the useful life. Net cost above €800 is depreciated; at or below €800 it is treated as `full` (GWG). |
| `elster_expense_share` | decimal (e.g. `0.20`) | Fraction of the net amount that is a deductible business expense. Only effective when `elster_deduction:proportional`. |
| `elster_vat_share` | decimal (e.g. `0.20`) | Fraction of input VAT that is deductible. Only effective when `elster_deduction:proportional`. |
| `elster_afa_years` | integer (e.g. `3`) | Useful life in years for straight-line depreciation. Only effective when `elster_deduction:afa`. |
| `elster_calculation` | `manual` | Lists the account in the form but writes `MANUAL` instead of a calculated amount. Manual rows are excluded from calculated ESt summary totals; the Herleitung still shows the booked payment amount. Use this for unimplemented or externally calculated cases such as political party donations. |

---

### Transaction-level overrides

These are set in transaction or posting comments, not in account directives.

| Tag | Value | Meaning |
|-----|-------|---------|
| `elster_role` | `ignore` | Excludes a transaction or posting from all tax calculations. Useful for internal transfers that would otherwise be misclassified. |
| `elster_period` | `YYYY` | On individual postings: overrides the fiscal year for period attribution (same semantics as the account-level tag). |

---

### Derived roles (not set by the user)

The ingestion layer assigns these roles automatically; they never appear in
`accounts.journal`.

| Role | Condition | Effect |
|------|-----------|--------|
| `drawing` | Outflow from a `business` account with no matching `elster_form` or `elster_role` | Counted as Entnahme in the EÜR. |
| `contribution` | Inflow to a `business` account with no matching `elster_form` or `elster_role` | Counted as Einlage in the EÜR. |

Private expenses tagged with `elster_form:einkommensteuer` and paid from a
`business` account are **also** counted as Entnahmen, even though they carry an
explicit form tag. The EÜR Entnahmen line therefore equals the sum of all
unclassified business-account outflows plus all private-form expenses paid from
the business account.
