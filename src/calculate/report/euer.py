from __future__ import annotations

from collections import defaultdict
from decimal import ROUND_HALF_UP, Decimal

from calculate import afa, aggregates
from calculate.drawing import is_contribution, is_drawing
from calculate.report.classification import euer_expenses, euer_income
from calculate.report.periods import (
    aggregate_periods,
    annual_labels,
    blank_row,
    fmt,
    section_row,
)
from domain.dataset import TaxDataset
from domain.posting import TaxPosting

TWOPLACES = Decimal("0.01")
ZERO = Decimal("0.00")

# Section ordering for EÜR Betriebsausgaben.
_SECTION_ORDER = [
    "Bezogene Fremdleistungen",
    "Fortbildungskosten",
    "Rechts- und Steuerberatung",
    "Arbeitsmittel",
]


def _section_sort_key(name: str) -> int:
    try:
        return _SECTION_ORDER.index(name)
    except ValueError:
        return len(_SECTION_ORDER)


def _account_label(dataset: TaxDataset, account: str, year: int) -> str:
    """Human label for an account, falling back to the last path component."""
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.label if p and p.label else "") or account.split(":")[-1]


def _account_section(dataset: TaxDataset, account: str, year: int) -> str:
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.section if p else "") or ""


def _posting_label(p: TaxPosting) -> str:
    return p.label or p.counter_account.split(":")[-1]


def _row_from_values(name: str, values: dict[str, Decimal]) -> dict[str, str]:
    row = {"Kennzahl": name}
    for label, value in values.items():
        row[label] = fmt(value)
    return row


def _home_office_pauschale(year: int) -> Decimal:
    if 2020 <= year <= 2022:
        return Decimal("600.00")
    if year >= 2023:
        return Decimal("1260.00")
    return ZERO


def _afa_for_label(postings: list[TaxPosting], year: int, label: str) -> Decimal:
    """Distribute annual AfA across periods by counting active months in each period."""
    total = ZERO
    for p in postings:
        total += _afa_posting_for_label(p, year, label)
    return total.quantize(TWOPLACES, rounding=ROUND_HALF_UP)


def _afa_posting_for_label(p: TaxPosting, year: int, label: str) -> Decimal:
    purchase = p.posting_date
    total_months = p.afa_years * 12
    if total_months <= 0:
        return ZERO

    cost = afa.net_cost(p)
    monthly = cost / Decimal(total_months)

    window_end_month_abs = (purchase.year - 1) * 12 + purchase.month - 1 + total_months
    window_end_year = (window_end_month_abs - 1) // 12 + 1
    window_end_cal_month = ((window_end_month_abs - 1) % 12) + 1

    def month_in_window(y: int, m: int) -> bool:
        if y < purchase.year or (y == purchase.year and m < purchase.month):
            return False
        if y > window_end_year or (y == window_end_year and m > window_end_cal_month):
            return False
        return True

    if label == str(year):
        months = [m for m in range(1, 13) if month_in_window(year, m)]
    elif "Q" in label:
        q = int(label.split("Q")[1])
        start = (q - 1) * 3 + 1
        months = [m for m in range(start, start + 3) if month_in_window(year, m)]
    else:
        m = int(label.split("-")[1])
        months = [m] if month_in_window(year, m) else []

    return monthly * len(months)


def euer_rows(dataset: TaxDataset, year: int) -> list[dict[str, str]]:
    labels = annual_labels(year)
    euer_ds = euer_expenses(dataset)

    # ── Betriebseinnahmen ─────────────────────────────────────────────────
    income_ds = euer_income(dataset)
    net_totals = aggregate_periods(income_ds, year, aggregates.net_amount, labels)
    collected_totals = aggregate_periods(income_ds, year, aggregates.collected_vat, labels)

    einnahmen_net_row = {"Kennzahl": "Umsatzsteuerpflichtige Betriebseinnahmen"}
    einnahmen_ust_row = {"Kennzahl": "Vereinnahmte Umsatzsteuer"}
    einnahmen_total: dict[str, Decimal] = {}

    for lbl in labels:
        einnahmen_net_row[lbl] = fmt(net_totals[lbl])
        einnahmen_ust_row[lbl] = fmt(collected_totals[lbl])
        einnahmen_total[lbl] = (net_totals[lbl] + collected_totals[lbl]).quantize(TWOPLACES, rounding=ROUND_HALF_UP)

    # ── regular expense accounts, grouped by section ──────────────────────
    regular_ds = euer_ds.exclude_deduction("afa").for_year(year)

    by_section: dict[str, dict[str, list[TaxPosting]]] = defaultdict(lambda: defaultdict(list))
    for p in regular_ds:
        by_section[p.section][_posting_label(p)].append(p)

    expense_sections: list[tuple[str, list[dict[str, str]]]] = []
    summe_betriebskosten: dict[str, Decimal] = {lbl: ZERO for lbl in labels}

    for section_name in sorted(by_section.keys(), key=_section_sort_key):
        section_rows: list[dict[str, str]] = []
        for label in sorted(by_section[section_name]):
            acc_ds = TaxDataset(by_section[section_name][label])
            row: dict[str, str] = {"Kennzahl": label}
            for lbl in labels:
                value = aggregate_periods(acc_ds, year, aggregates.deductible_net, labels)[lbl]
                row[lbl] = fmt(value)
                summe_betriebskosten[lbl] += value
            section_rows.append(row)
        if section_rows:
            expense_sections.append((section_name, section_rows))

    for lbl in labels:
        summe_betriebskosten[lbl] = summe_betriebskosten[lbl].quantize(TWOPLACES, rounding=ROUND_HALF_UP)

    # ── AfA accounts ──────────────────────────────────────────────────────
    afa_postings = list(dataset.for_deduction("afa"))
    afa_accounts = sorted({p.counter_account for p in afa_postings})
    afa_sections: dict[str, list[dict[str, str]]] = defaultdict(list)
    afa_totals: dict[str, Decimal] = {lbl: ZERO for lbl in labels}

    for account in afa_accounts:
        acc_postings = [p for p in afa_postings if p.counter_account == account]
        label = next((p.label for p in acc_postings if p.label), None) or account.split(":")[-1]
        section_name = next((p.section for p in acc_postings if p.section), "") or "AfA"
        row = {"Kennzahl": f"AfA {label}"}
        for lbl in labels:
            value = _afa_for_label(acc_postings, year, lbl)
            row[lbl] = fmt(value)
            afa_totals[lbl] += value
        afa_sections[section_name].append(row)

    for lbl in labels:
        afa_totals[lbl] = afa_totals[lbl].quantize(TWOPLACES, rounding=ROUND_HALF_UP)

    # ── Home-Office-Pauschale ─────────────────────────────────────────────
    annual_hop = _home_office_pauschale(year)

    # ── UStVA payments (ELSTER EÜR line 57) — advance + final settlements ──
    vat_paid_ds = TaxDataset(
        [p for p in dataset if p.tax_role in ("vat_payment", "vat_advance") and p.amount > Decimal("0")]
    )
    vat_refund_ds = TaxDataset([p for p in dataset if p.tax_role == "vat_payment" and p.amount < Decimal("0")])
    ust_paid_totals = aggregate_periods(vat_paid_ds, year, aggregates.gross_amount, labels)
    ust_refund_totals = aggregate_periods(vat_refund_ds, year, aggregates.gross_amount, labels)

    # ── Entnahmen / Einlagen from business accounts ──────────────────────
    drawing_ds = TaxDataset([p for p in dataset if is_drawing(p)])
    drawing_totals = aggregate_periods(drawing_ds, year, aggregates.gross_amount, labels)
    contribution_ds = TaxDataset([p for p in dataset if is_contribution(p)])
    contribution_totals = aggregate_periods(contribution_ds, year, aggregates.gross_amount, labels)

    # ── summary rows ─────────────────────────────────────────────────────
    summe_betriebseinnahmen: dict[str, Decimal] = {}
    summe_betriebsausgaben: dict[str, Decimal] = {}
    gewinn_totals: dict[str, Decimal] = {}

    for lbl in labels:
        summe_betriebseinnahmen[lbl] = (einnahmen_total[lbl] + ust_refund_totals[lbl]).quantize(
            TWOPLACES, rounding=ROUND_HALF_UP
        )
        betriebsausgaben = (summe_betriebskosten[lbl] + afa_totals[lbl] + annual_hop + ust_paid_totals[lbl]).quantize(
            TWOPLACES, rounding=ROUND_HALF_UP
        )
        summe_betriebsausgaben[lbl] = betriebsausgaben
        gewinn = (einnahmen_total[lbl] - betriebsausgaben).quantize(TWOPLACES, rounding=ROUND_HALF_UP)
        gewinn_totals[lbl] = gewinn

    rows: list[dict[str, str]] = [
        section_row("Betriebseinnahmen", labels),
        einnahmen_net_row,
        einnahmen_ust_row,
        _row_from_values(
            "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer",
            ust_refund_totals,
        ),
        _row_from_values("Summe Betriebseinnahmen", summe_betriebseinnahmen),
        blank_row(labels),
        section_row("Betriebsausgaben", labels),
    ]

    for section_name, section_rows in expense_sections:
        if section_name:
            rows.append(section_row(section_name, labels))
        rows.extend(section_rows)

    for section_name in sorted(afa_sections.keys(), key=_section_sort_key):
        rows.append(section_row(section_name, labels))
        rows.extend(afa_sections[section_name])

    rows.extend(
        [
            blank_row(labels),
            _row_from_values("Home-Office-Pauschale", {lbl: annual_hop for lbl in labels}),
            _row_from_values(
                "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer",
                ust_paid_totals,
            ),
            _row_from_values("Summe Betriebskosten", summe_betriebskosten),
            _row_from_values("Summe Betriebsausgaben", summe_betriebsausgaben),
            blank_row(labels),
            section_row("Ermittlung des Gewinns", labels),
            _row_from_values("Steuerpflichtiger Gewinn/Verlust", gewinn_totals),
            blank_row(labels),
            section_row("Zusätzliche Angaben bei Einzelunternehmen", labels),
            _row_from_values("Entnahmen", drawing_totals),
            _row_from_values("Einlagen", contribution_totals),
        ]
    )

    return rows
