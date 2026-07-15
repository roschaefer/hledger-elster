# hledger-elster

`hledger-elster` generates German tax exports for [ELSTER.de](https://www.elster.de) from an `hledger` journal.

## Installation

Required tools:

- [`hledger`](https://hledger.org/) 1.52.1

Download the standalone executable for your platform from the
[latest release](https://github.com/roschaefer/hledger-elster/releases/tag/latest)
and put it on `PATH`:

```bash
curl -L \
  -o /tmp/hledger-elster \
  https://github.com/roschaefer/hledger-elster/releases/download/latest/hledger-elster-linux-x86_64
install -m 0755 /tmp/hledger-elster ~/.local/bin/hledger-elster
```

For macOS on Apple Silicon, use the `hledger-elster-macos-arm64` asset instead.

## Usage

Outputs are written under `data/exports/<year>/` by default:

- `steuererklaerung/`: summary CSV files plus `steuererklaerung.xlsx`
- `herleitung/`: detailed audit trails plus per-form `.xlsx`

Workbook export convention:

- every `name.xlsx` workbook has a sibling `name/` directory
- each workbook tab is also exported there as a CSV with the corresponding derived filename

```bash
hledger-elster
hledger-elster -f examples/ledger/hledger.journal
hledger-elster -f examples/ledger/hledger.journal --config elster.toml -o /tmp/elster-out
```

Arguments:

- `-f`, `--file`: input journal, with the same meaning as `hledger -f`
- `-o`, `--output-dir`: output directory for generated tax artifacts
- `--config`: TOML config file for user-specific tax adjustments

Sanitized public fixtures live under [`examples/`](./examples). Keep real journals,
tax filings, and verification data outside this repository.

## Audience

This tool currently targets my own situation: I am a software developer working
remotely as a freelancer (`selbstständig`) and as a `Freiberufler`. I do not pay
Gewerbesteuer, but I do need to file EÜR, USt, and ESt.

I am not an expert in German tax law, and calculations may contain errors. Bug
fixes and contributions are very welcome, especially for additional tax
scenarios. The goal is to cover more cases over time through executable examples.

## Specification By Example

Executable specifications in [`specs/`](./specs/) — run them with:

```bash
cargo test --test cucumber
```

Specs:

- [Configuration](./specs/00-configuration.md)
- [Export hygiene](./specs/01-export-hygiene.md)
- [Business expenses and income](./specs/02-form-section-item-tags.md)
- [VAT payments and settlements](./specs/03-vat.md)
- [VAT reverse charge](./specs/04-vat-reverse-charge.md)
- [Business vs. private accounts](./specs/05-business-accounts.md)
- [Health care and insurance](./specs/06-health-care.md)
- [GWG and AfA](./specs/07-afa.md)
- [Donations](./specs/08-donations.md)

## Development

```bash
git clone git@github.com:roschaefer/hledger-elster.git
cd hledger-elster
cargo build --release
cargo test
```

`just check` runs the same formatting, lint, and test gates as CI.

## Documentation

The tool reads account-level metadata from hledger account directives (`;` comments).
All tags are prefixed with `elster_`. Tags fall into three categories:

- **Routing tags** — which tax form an account belongs to and how it is grouped there
- **Calculation tags** — how amounts are transformed before they appear in the output
- **Infrastructure tags** — account classification used by the ingestion layer

### Example `accounts.journal`

```hledger
; ── Payment accounts ──────────────────────────────────────────────────────────
account assets:bank:business          ; elster_account:business, elster_item:Geschäftskonto
account assets:bank:private           ; elster_account:private,  elster_item:Girokonto

; ── Business income (EÜR + USt) ───────────────────────────────────────────────
account income:business               ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

; ── Business expenses (EÜR) ───────────────────────────────────────────────────
; Base account sets defaults; sub-accounts inherit and override.
account expenses:business             ; elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat:contains_vat, elster_vat_rate:0.19, elster_input_vat_share:1.00
account expenses:business:hosting     ; elster_item:Serverkosten, elster_section:Bezogene Fremdleistungen
account expenses:business:education   ; elster_item:Fortbildung,  elster_section:Fortbildungskosten

; Proportional deduction (e.g. phone: 20 % business use)
account expenses:phone                ; elster_form:einnahmenueberschussrechnung, elster_deduction:proportional, elster_expense_share:0.20, elster_vat:contains_vat, elster_input_vat_share:0.20, elster_vat_rate:0.19, elster_item:Mobiltelefon, elster_section:Arbeitsmittel

; Depreciable asset (AfA)
account expenses:hardware:computer          ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_afa_years:3, elster_item:Computer-Kauf, elster_section:Arbeitsmittel

; ── Private expenses (ESt) ────────────────────────────────────────────────────
account expenses:insurance            ; elster_form:einkommensteuer
account expenses:insurance:health:kv  ; elster_item:Krankenversicherung,  elster_section:Vorsorgeaufwand
account expenses:insurance:health:pv  ; elster_item:Pflegeversicherung,   elster_section:Vorsorgeaufwand

; Donations are ordinary ESt accounts grouped by a user-defined section.
; Child accounts without their own item inherit this item and are summed into one row.
account expenses:charity              ; elster_form:einkommensteuer, elster_item:Spenden, elster_section:Sonderausgaben
account expenses:charity:drk
account expenses:charity:unicef

; Use manual calculation for cases the tool should list but not calculate.
account expenses:politics:party       ; elster_form:einkommensteuer, elster_item:Parteispende - §34g/§10b manuell berechnen, elster_section:Sonderausgaben, elster_calculation:manual

; ── Tax payments ──────────────────────────────────────────────────────────────
account expenses:taxes:einkommensteuer:vorauszahlung    ; elster_role:income_tax_advance, elster_item:ESt-Vorauszahlung
account expenses:taxes:einkommensteuer:abschlusszahlung ; elster_role:income_tax_final,   elster_item:ESt-Abschlusszahlung
account expenses:taxes:umsatzsteuer:vorauszahlung       ; elster_role:vat_advance
account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
account expenses:taxes:umsatzsteuer:vorauszahlung:2025  ; elster_period:2025
```

---

### Config file

User-specific tax assumptions that are not ledger transactions live in a TOML
config file. Generate the default config with:

```bash
hledger-elster init-config --output elster.toml
```

The default config enables the Home-Office-Pauschale and uses the maximum number
of days for each supported year, because this adjustment is easy to forget:

```toml
[euer.home_office_pauschale]
enabled = true
default_days = "max"
# Set per-year days when the default does not match your situation.
# 2020-2022: 5 EUR/day, capped at 600 EUR.
# 2023+: 6 EUR/day, capped at 1260 EUR.

[euer.home_office_pauschale.days]
# 2024 = 210
```

Use `enabled = false` for journal-only exports, or set per-year days when the
maximum does not match your situation.

---

### Routing tags

| Tag | Values | Meaning |
|-----|--------|---------|
| `elster_account` | `business` \| `private` | Marks a payment account as belonging to the business or private sphere. Drives the drawing/contribution fallback: any unclassified outflow from a `business` account is counted as an Entnahme in the EÜR. |
| `elster_role` | `income_tax_advance` \| `income_tax_final` | Marks ESt payment accounts. Postings appear in the ESt summary, separated by advance vs. final settlement. |
| `elster_role` | `tax_payment` | Generic parent role for all tax payments. Prevents tax outflows from being counted as Entnahmen in the EÜR. |
| `elster_role` | `vat_payment` | Marks USt Abschlusszahlung accounts. Postings flow into the EÜR (line 57: gezahlte Umsatzsteuer) and the USt report. |
| `elster_role` | `vat_advance` | Marks USt Vorauszahlung accounts. Requires `elster_period` on sub-accounts for correct fiscal-year attribution. |
| `elster_form` | `einnahmenueberschussrechnung` | Marks an account as belonging to the EÜR. Income accounts flow into Betriebseinnahmen; expense accounts flow into Betriebsausgaben. VAT handling is controlled by `elster_vat` and `elster_vat_rate`; deduction treatment by the calculation tags below. The USt export is derived from these EÜR VAT fields and VAT payment roles. |
| `elster_form` | `einkommensteuer` | Marks an account as belonging to the ESt. The account appears under the user-defined `elster_section`; postings from a `business` source account are additionally counted as Entnahmen in the EÜR. |
| `elster_section` | free text | User-defined grouping within a form (for example `Sonderausgaben` for donations or `Arbeitsmittel` for EÜR expenses). The code does not interpret specific section names. |
| `elster_item` | free text | Report item shown as an output row. Use it to translate technical account names and to define aggregation boundaries: child accounts without their own `elster_item` are summed into the inherited parent item; child accounts with their own `elster_item` appear separately. |
| `elster_period` | `YYYY` | On USt Vorauszahlung sub-accounts: the fiscal year the payment belongs to, regardless of when the transaction occurred. Required on every `vat_advance` sub-account. |

---

### Calculation tags

| Tag | Values | Meaning |
|-----|--------|---------|
| `elster_vat` | `contains_vat` | The booked amount is gross and contains VAT. `elster_vat_rate` splits the amount into net and VAT. |
| `elster_vat` | `reverse_charge_eu` \| `reverse_charge_non_eu` | The booked amount is net. `elster_vat_rate` calculates German VAT on top for the USt reverse-charge rows. |
| `elster_vat` | `not_applicable` | No VAT calculation applies. Do not set `elster_vat_rate` or `elster_input_vat_share`. |
| `elster_vat_rate` | `0.19` \| `0.07` | German VAT rate used by `elster_vat:contains_vat` and reverse-charge modes. |
| `elster_deduction` | `full` | The full net amount is deductible as a business expense. |
| `elster_deduction` | `proportional` | Only a fraction is deductible. Set `elster_expense_share`; set `elster_input_vat_share` too when the deductible input VAT share differs. |
| `elster_deduction` | `non_deductible` | Show an EÜR expense row but count `0.00` toward deductible business expenses and input VAT. |
| `elster_deduction` | `afa` | Triggers straight-line depreciation. Set `elster_afa_years` to the useful life. Net cost above €800 is depreciated; at or below €800 it is treated as `full` (GWG). |
| `elster_expense_share` | decimal (e.g. `0.20`) | Fraction of the net amount that is a deductible business expense. Only effective when `elster_deduction:proportional`. |
| `elster_input_vat_share` | decimal (e.g. `0.20`) | Fraction of input VAT that is deductible. Defaults to `elster_expense_share` when omitted. Hospitality can use `elster_expense_share:0.70` with `elster_input_vat_share:1.00`. |
| `elster_afa_years` | integer (e.g. `3`) | Useful life in years for straight-line depreciation. Only effective when `elster_deduction:afa`. |
| `elster_calculation` | `manual` | Lists the account in the form but writes `MANUAL` instead of a calculated amount. The Herleitung still shows the booked payment amount. Use this for unimplemented or externally calculated cases such as political party donations. |

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
