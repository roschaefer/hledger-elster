from __future__ import annotations

from collections import defaultdict
from decimal import Decimal

from calculate import aggregates
from calculate.report.periods import aggregate_periods, annual_labels, blank_row, fmt, section_row
from domain.dataset import TaxDataset

ZERO = Decimal("0.00")
MANUAL_PLACEHOLDER = "MANUAL"


def _est_section_ds(dataset: TaxDataset, year: int) -> TaxDataset:
    return TaxDataset([p for p in dataset if p.tax_form == "einkommensteuer" and p.section and p.year == year])


def _account_label(dataset: TaxDataset, account: str, year: int) -> str:
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.label if p and p.label else "") or account.split(":")[-1]


def _account_section(dataset: TaxDataset, account: str, year: int) -> str:
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.section if p else "") or ""


def _posting_label(p) -> str:
    return p.label or p.counter_account.split(":")[-1]


def _requires_manual_calculation(dataset: TaxDataset) -> bool:
    return any(p.calculation == "manual" for p in dataset)


def est_rows(dataset: TaxDataset, year: int) -> list[dict[str, str]]:
    labels = annual_labels(year)

    # ── income tax payments ───────────────────────────────────────────────
    advance_ds = dataset.for_role("income_tax_advance")
    final_ds = dataset.for_role("income_tax_final")

    tax_rows: list[dict[str, str]] = []
    for role_ds in (advance_ds, final_ds):
        accounts = sorted({p.counter_account for p in role_ds})
        for account in accounts:
            acc_ds = dataset.for_account_prefix(account)
            totals = aggregate_periods(acc_ds, year, aggregates.signed_total, labels)
            if all(v == ZERO for v in totals.values()):
                continue
            label = _account_label(dataset, account, year)
            row = {"Kennzahl": label}
            for lbl in labels:
                row[lbl] = fmt(totals[lbl])
            tax_rows.append(row)

    # ── ESt account sections ──────────────────────────────────────────────
    section_ds = _est_section_ds(dataset, year)

    by_section: dict[str, dict[str, list]] = defaultdict(lambda: defaultdict(list))
    for p in section_ds:
        by_section[p.section][_posting_label(p)].append(p)

    section_rows: list[dict[str, str]] = []

    for sec_name in sorted(by_section.keys()):
        if sec_name:
            section_rows.append(section_row(sec_name, labels))
        for label in sorted(by_section[sec_name].keys()):
            acc_ds = TaxDataset(by_section[sec_name][label])
            totals = aggregate_periods(acc_ds, year, aggregates.signed_total, labels)
            row = {"Kennzahl": label}
            manual = _requires_manual_calculation(acc_ds)
            for lbl in labels:
                if manual:
                    row[lbl] = MANUAL_PLACEHOLDER
                    continue
                v = totals[lbl]
                row[lbl] = fmt(v)
            section_rows.append(row)

    rows: list[dict[str, str]] = []
    if tax_rows:
        rows.extend(tax_rows)
        rows.append(blank_row(labels))
    rows.extend(section_rows)
    return rows
