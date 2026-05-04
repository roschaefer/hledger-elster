from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal, ROUND_HALF_UP

from calculate import afa as afa_module
from calculate.drawing import is_drawing, is_contribution
from calculate.report.classification import euer_expenses, euer_income
from domain.dataset import TaxDataset
from domain.posting import TaxPosting


TWOPLACES = Decimal("0.01")
ZERO = Decimal("0.00")


@dataclass
class TrailRow:
    cells: list[str]
    outline_level: int   # 0=total, 1=bank subtotal, 2=transaction (used for styling only)
    bold: bool = False
    fill: str = ""       # "" | "subtotal" | "total"


@dataclass
class TrailSheet:
    name: str
    headers: list[str]
    rows: list[TrailRow]


# ── helpers ───────────────────────────────────────────────────────────────────

def _q(v: Decimal) -> Decimal:
    return v.quantize(TWOPLACES, rounding=ROUND_HALF_UP)


def _fmt(v: Decimal) -> str:
    return f"{v:.2f}"


def _gross(p: TaxPosting) -> Decimal:
    return _q(abs(p.amount))


def _signed_gross(p: TaxPosting) -> Decimal:
    return _q(p.amount)


def _net(p: TaxPosting) -> Decimal:
    g = abs(p.amount)
    if p.vat_rate > ZERO:
        return _q(g / (1 + p.vat_rate))
    return _q(g)


def _vat_amount(p: TaxPosting) -> Decimal:
    return _q(_gross(p) - _net(p))


def _signed_net(p: TaxPosting) -> Decimal:
    if p.vat_rate > ZERO:
        return _q(p.amount / (1 + p.vat_rate))
    return _q(p.amount)


def _signed_vat_amount(p: TaxPosting) -> Decimal:
    return _q(_signed_gross(p) - _signed_net(p))


def _deductible_net(p: TaxPosting) -> Decimal:
    return _q(_net(p) * p.expense_share)


def _deductible_vat(p: TaxPosting) -> Decimal:
    return _q(_vat_amount(p) * p.vat_share)


def _signed_deductible_net(p: TaxPosting) -> Decimal:
    return _q(_signed_net(p) * p.expense_share)


def _signed_deductible_vat(p: TaxPosting) -> Decimal:
    return _q(_signed_vat_amount(p) * p.vat_share)


def _short(account: str) -> str:
    return account.split(":")[-1]


def _bank_label(p: TaxPosting) -> str:
    """Human-readable label for the source (bank) account."""
    return p.source_label or _short(p.source_account)


def _sheet_label(ds: TaxDataset, account: str) -> str:
    """Use elster_label from first matching posting if available, else last account component."""
    return next((p.label for p in ds if p.label), None) or _short(account)


def _posting_label(p: TaxPosting) -> str:
    return p.label or _short(p.counter_account)


def _est_section_ds(dataset: TaxDataset, year: int) -> TaxDataset:
    return TaxDataset([
        p for p in dataset
        if p.tax_form == "einkommensteuer" and p.section and p.year == year
    ])


def _by_bank(postings: list[TaxPosting]) -> dict[str, list[TaxPosting]]:
    """Group postings by source account, sorted by account then date."""
    groups: dict[str, list[TaxPosting]] = {}
    for p in sorted(postings, key=lambda x: (x.source_account, x.posting_date)):
        groups.setdefault(p.source_account, []).append(p)
    return groups


def _bank_subtotal_label(ps: list[TaxPosting]) -> str:
    """Short label for a bank subtotal row: 'Σ <bank_label>'."""
    return f"Σ {_bank_label(ps[0])}" if ps else "Σ"


def _unique_name(name: str, seen: set[str]) -> str:
    candidate = name[:31]
    if candidate not in seen:
        seen.add(candidate)
        return candidate
    i = 2
    while True:
        suffix = f" ({i})"
        candidate = name[: 31 - len(suffix)] + suffix
        if candidate not in seen:
            seen.add(candidate)
            return candidate
        i += 1


# ── sheet builders ────────────────────────────────────────────────────────────

def _expense_sheet(ds: TaxDataset, account: str) -> TrailSheet:
    headers = ["Konto", "Datum", "Beschreibung", "Brutto", "Netto", "Anteil", "Abziehbar"]
    rows: list[TrailRow] = []
    t_gross = t_net = t_ded = ZERO

    for _bank_acct, ps in _by_bank(list(ds)).items():
        b_gross = b_net = b_ded = ZERO
        for p in ps:
            g, n, d = _signed_gross(p), _signed_net(p), _signed_deductible_net(p)
            rows.append(TrailRow([_bank_label(p), str(p.posting_date), p.description, _fmt(g), _fmt(n), _fmt(p.expense_share), _fmt(d)], 2))
            b_gross += g; b_net += n; b_ded += d
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", "", _fmt(_q(b_gross)), _fmt(_q(b_net)), "", _fmt(_q(b_ded))],
            1, bold=True, fill="subtotal",
        ))
        t_gross += b_gross; t_net += b_net; t_ded += b_ded

    rows.append(TrailRow(
        ["GESAMT", "", "", _fmt(_q(t_gross)), _fmt(_q(t_net)), "", _fmt(_q(t_ded))],
        0, bold=True, fill="total",
    ))
    return TrailSheet(_sheet_label(ds, account), headers, rows)


def _income_sheet(ds: TaxDataset) -> TrailSheet:
    headers = ["Konto", "Datum", "Beschreibung", "Brutto", "Netto", "USt-Betrag"]
    rows: list[TrailRow] = []
    t_gross = t_net = t_vat = ZERO

    for _bank_acct, ps in _by_bank(list(ds)).items():
        b_gross = b_net = b_vat = ZERO
        for p in ps:
            g, n, v = _gross(p), _net(p), _vat_amount(p)
            rows.append(TrailRow([_bank_label(p), str(p.posting_date), p.description, _fmt(g), _fmt(n), _fmt(v)], 2))
            b_gross += g; b_net += n; b_vat += v
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", "", _fmt(_q(b_gross)), _fmt(_q(b_net)), _fmt(_q(b_vat))],
            1, bold=True, fill="subtotal",
        ))
        t_gross += b_gross; t_net += b_net; t_vat += b_vat

    rows.append(TrailRow(
        ["GESAMT", "", "", _fmt(_q(t_gross)), _fmt(_q(t_net)), _fmt(_q(t_vat))],
        0, bold=True, fill="total",
    ))
    return TrailSheet("Einnahmen", headers, rows)


def _input_vat_sheet(ds: TaxDataset) -> TrailSheet:
    """Abziehbare Vorsteuer for EÜR expense postings."""
    headers = ["Konto", "Datum", "Beschreibung", "Brutto", "USt-Betrag", "USt-Anteil", "Abziehbar"]
    rows: list[TrailRow] = []
    postings = [p for p in ds if p.vat_rate > ZERO]
    t_gross = t_vat = t_ded = ZERO

    for _bank_acct, ps in _by_bank(postings).items():
        b_gross = b_vat = b_ded = ZERO
        for p in ps:
            g, v, d = _signed_gross(p), _signed_vat_amount(p), _signed_deductible_vat(p)
            rows.append(TrailRow([_bank_label(p), str(p.posting_date), p.description, _fmt(g), _fmt(v), _fmt(p.vat_share), _fmt(d)], 2))
            b_gross += g; b_vat += v; b_ded += d
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", "", _fmt(_q(b_gross)), _fmt(_q(b_vat)), "", _fmt(_q(b_ded))],
            1, bold=True, fill="subtotal",
        ))
        t_gross += b_gross; t_vat += b_vat; t_ded += b_ded

    rows.append(TrailRow(
        ["GESAMT", "", "", _fmt(_q(t_gross)), _fmt(_q(t_vat)), "", _fmt(_q(t_ded))],
        0, bold=True, fill="total",
    ))
    return TrailSheet("Vorsteuer", headers, rows)


def _afa_sheet(postings: list[TaxPosting], account: str, year: int) -> TrailSheet:
    headers = ["Beschreibung", "Kaufdatum", "Kaufpreis (Netto)", "AfA-Jahre", "Monat. AfA", f"AfA {year}"]
    rows: list[TrailRow] = []
    total = ZERO

    for p in sorted(postings, key=lambda x: x.posting_date):
        if p.afa_years <= 0:
            continue
        cost = afa_module.net_cost(p)
        monthly = _q(cost / Decimal(p.afa_years * 12))
        annual = afa_module.depreciation_for_year(p, year)
        rows.append(TrailRow([p.description, str(p.posting_date), _fmt(cost), str(p.afa_years), _fmt(monthly), _fmt(annual)], 1))
        total += annual

    rows.append(TrailRow(["GESAMT", "", "", "", "", _fmt(_q(total))], 0, bold=True, fill="total"))
    label = next((p.label for p in postings if p.label), None) or _short(account)
    return TrailSheet(f"AfA {label}", headers, rows)


def _gross_sheet(ds: TaxDataset, name: str, signed: bool = False) -> TrailSheet:
    headers = ["Konto", "Datum", "Beschreibung", "Betrag"]
    rows: list[TrailRow] = []
    total = ZERO

    for _bank_acct, ps in _by_bank(list(ds)).items():
        b_total = ZERO
        for p in ps:
            amount = p.amount if signed else _gross(p)
            rows.append(TrailRow([_bank_label(p), str(p.posting_date), p.description, _fmt(_q(amount))], 2))
            b_total += amount
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", "", _fmt(_q(b_total))],
            1, bold=True, fill="subtotal",
        ))
        total += b_total

    rows.append(TrailRow(["GESAMT", "", "", _fmt(_q(total))], 0, bold=True, fill="total"))
    return TrailSheet(name, headers, rows)


def _vat_outflows_sheet(ds: TaxDataset) -> TrailSheet:
    return _gross_sheet(ds, "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer", signed=True)


def _vat_refunds_sheet(ds: TaxDataset) -> TrailSheet:
    return _gross_sheet(ds, "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer", signed=True)


def _ignored_sheet(ds: TaxDataset, year: int) -> TrailSheet:
    headers = ["Konto", "Datum", "Beschreibung", "Gegenkonto", "Betrag"]
    rows: list[TrailRow] = []
    total = ZERO

    for _bank_acct, ps in _by_bank(list(ds.for_role("ignore").for_year(year))).items():
        b_total = ZERO
        for p in ps:
            amount = _q(p.amount)
            rows.append(TrailRow([
                _bank_label(p),
                str(p.posting_date),
                p.description,
                p.counter_account,
                _fmt(amount),
            ], 2))
            b_total += amount
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", "", "", _fmt(_q(b_total))],
            1, bold=True, fill="subtotal",
        ))
        total += b_total

    rows.append(TrailRow(["GESAMT", "", "", "", _fmt(_q(total))], 0, bold=True, fill="total"))
    return TrailSheet("Ignoriert", headers, rows)


def _vat_advance_sheet(ds: TaxDataset, year: int) -> TrailSheet:
    invalid = [
        p for p in ds
        if p.tax_role == "vat_advance" and p.amount != ZERO and p.tax_period_year == 0
    ]
    if invalid:
        examples = ", ".join(
            f"{p.posting_date} {p.description} ({p.source_account} -> {p.counter_account})"
            for p in invalid[:3]
        )
        raise ValueError(
            "vat_advance postings require tax_period. "
            f"Missing tax_period for {len(invalid)} posting(s): {examples}"
        )

    headers = ["Konto", "Datum", "Steuerperiode", "Beschreibung", "Betrag"]
    rows: list[TrailRow] = []
    total = ZERO
    by_bank = _by_bank([p for p in ds if p.amount != ZERO and p.tax_period_year == year])

    for _bank_acct, ps in by_bank.items():
        b_total = ZERO
        for p in ps:
            amount = _q(p.amount)
            rows.append(TrailRow([
                _bank_label(p),
                str(p.posting_date),
                str(p.tax_period_year),
                p.description,
                _fmt(_q(amount)),
            ], 2))
            b_total += amount
        rows.append(TrailRow(
            [_bank_subtotal_label(ps), "", str(year), "", _fmt(_q(b_total))],
            1, bold=True, fill="subtotal",
        ))
        total += b_total

    rows.append(TrailRow(["GESAMT", "", str(year), "", _fmt(_q(total))], 0, bold=True, fill="total"))
    return TrailSheet("Bereits Entrichtet", headers, rows)


# ── per-form sheet lists ──────────────────────────────────────────────────────

def _collect(sheets: list[TrailSheet], seen: set[str], sheet: TrailSheet) -> None:
    if len(sheet.rows) <= 1:  # only a GESAMT row = no data
        return
    sheet.name = _unique_name(sheet.name, seen)
    sheets.append(sheet)


def _euer_sheets(dataset: TaxDataset, year: int) -> list[TrailSheet]:
    result: list[TrailSheet] = []
    seen: set[str] = set()
    add = lambda s: _collect(result, seen, s)  # noqa: E731

    income_ds = euer_income(dataset).for_year(year)
    add(_income_sheet(income_ds))

    euer_ds = euer_expenses(dataset)
    by_label: dict[str, list[TaxPosting]] = {}
    for p in euer_ds.exclude_deduction("afa").for_year(year):
        by_label.setdefault(_posting_label(p), []).append(p)
    for label in sorted(by_label.keys()):
        add(_expense_sheet(TaxDataset(by_label[label]), label))

    afa_postings = list(dataset.for_deduction("afa"))
    for account in sorted({p.counter_account for p in afa_postings}):
        acc_ps = [p for p in afa_postings if p.counter_account == account]
        add(_afa_sheet(acc_ps, account, year))

    add(_input_vat_sheet(euer_ds.for_year(year)))

    vat_payment_ds = TaxDataset([
        p for p in dataset.for_year(year)
        if p.tax_role == "vat_payment" and p.amount > ZERO
    ])
    vat_outflows_ds = TaxDataset([
        p for p in dataset.for_year(year)
        if p.tax_role in ("vat_payment", "vat_advance") and p.amount > ZERO
    ])
    vat_refund_ds = TaxDataset([
        p for p in dataset.for_year(year)
        if p.tax_role == "vat_payment" and p.amount < ZERO
    ])
    add(_vat_outflows_sheet(vat_outflows_ds))
    add(_vat_refunds_sheet(vat_refund_ds))
    add(_gross_sheet(vat_payment_ds, "USt-Abschlusszahlungen"))
    add(_vat_advance_sheet(dataset.for_role("vat_advance"), year))

    drawing_ds = TaxDataset([p for p in dataset.for_year(year) if is_drawing(p)])
    add(_gross_sheet(drawing_ds, "Entnahmen"))
    contribution_ds = TaxDataset([p for p in dataset.for_year(year) if is_contribution(p)])
    add(_gross_sheet(contribution_ds, "Einlagen"))

    return result


def _ust_sheets(dataset: TaxDataset, year: int) -> list[TrailSheet]:
    result: list[TrailSheet] = []
    seen: set[str] = set()
    add = lambda s: _collect(result, seen, s)  # noqa: E731

    income_ds = euer_income(dataset).for_year(year)
    add(_income_sheet(income_ds))

    vat_payment_ds = TaxDataset([
        p for p in dataset.for_year(year)
        if p.tax_role == "vat_payment" and p.amount > ZERO
    ])
    add(_gross_sheet(vat_payment_ds, "USt-Abschlusszahlungen"))
    add(_vat_advance_sheet(dataset.for_role("vat_advance"), year))

    euer_ds = euer_expenses(dataset)
    add(_input_vat_sheet(euer_ds.for_year(year)))

    return result


def _est_sheets(dataset: TaxDataset, year: int) -> list[TrailSheet]:
    result: list[TrailSheet] = []
    seen: set[str] = set()
    add = lambda s: _collect(result, seen, s)  # noqa: E731

    section_ds = _est_section_ds(dataset, year)
    by_label: dict[str, list[TaxPosting]] = {}
    for p in section_ds:
        by_label.setdefault(_posting_label(p), []).append(p)
    for label in sorted(by_label.keys()):
        add(_gross_sheet(TaxDataset(by_label[label]), label, signed=True))

    add(_gross_sheet(dataset.for_role("income_tax_advance").for_year(year), "ESt Vorauszahlung"))
    add(_gross_sheet(dataset.for_role("income_tax_final").for_year(year), "ESt Abschluss"))

    return result


# ── public entry point ────────────────────────────────────────────────────────

FORM_KEYS = [
    "einnahmen-ueberschuss-rechnung",
    "umsatzsteuer",
    "einkommensteuer",
]


def herleitung_sheets(dataset: TaxDataset, year: int) -> dict[str, list[TrailSheet]]:
    ignored = _ignored_sheet(dataset, year)
    return {
        "einnahmen-ueberschuss-rechnung": _euer_sheets(dataset, year),
        "umsatzsteuer":                   _ust_sheets(dataset, year),
        "einkommensteuer":                _est_sheets(dataset, year),
        "ignoriert":                     [ignored] if len(ignored.rows) > 1 else [],
    }
