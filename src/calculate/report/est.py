from __future__ import annotations

from collections import defaultdict
from decimal import Decimal, ROUND_HALF_UP

from domain.dataset import TaxDataset
from calculate import aggregates
from calculate.report.periods import aggregate_periods, annual_labels, blank_row, fmt, section_row


TWOPLACES = Decimal("0.01")
ZERO = Decimal("0.00")
MANUAL_PLACEHOLDER = "MANUAL"

def _est_section_ds(dataset: TaxDataset, year: int) -> TaxDataset:
    return TaxDataset([
        p for p in dataset
        if p.tax_form == "einkommensteuer" and p.section and p.year == year
    ])


def _account_label(dataset: TaxDataset, account: str, year: int) -> str:
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.label if p and p.label else "") or account.split(":")[-1]


def _account_section(dataset: TaxDataset, account: str, year: int) -> str:
    p = next(iter(dataset.for_account_prefix(account).for_year(year)), None)
    return (p.section if p else "") or ""


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
    section_accounts = sorted({p.counter_account for p in section_ds})

    by_section: dict[str, list[str]] = defaultdict(list)
    for account in section_accounts:
        sec = _account_section(section_ds, account, year)
        by_section[sec].append(account)

    section_rows: list[dict[str, str]] = []
    summe_totals: dict[str, Decimal] = {lbl: ZERO for lbl in labels}
    abziehbar_totals: dict[str, Decimal] = {lbl: ZERO for lbl in labels}

    for sec_name in sorted(by_section.keys()):
        if sec_name:
            section_rows.append(section_row(sec_name, labels))
        for account in sorted(by_section[sec_name]):
            acc_ds = section_ds.for_account_prefix(account)
            totals = aggregate_periods(acc_ds, year, aggregates.signed_total, labels)
            deductible_totals = aggregate_periods(acc_ds, year, aggregates.deductible_net, labels)
            label = _account_label(section_ds, account, year)
            row = {"Kennzahl": label}
            manual = _requires_manual_calculation(acc_ds)
            for lbl in labels:
                if manual:
                    row[lbl] = MANUAL_PLACEHOLDER
                    continue
                v = totals[lbl]
                row[lbl] = fmt(v)
                summe_totals[lbl] += v
                abziehbar_totals[lbl] += deductible_totals[lbl]
            section_rows.append(row)

    for lbl in labels:
        summe_totals[lbl] = summe_totals[lbl].quantize(TWOPLACES, rounding=ROUND_HALF_UP)
        abziehbar_totals[lbl] = abziehbar_totals[lbl].quantize(TWOPLACES, rounding=ROUND_HALF_UP)

    # ── summary ───────────────────────────────────────────────────────────
    summe_row = {"Kennzahl": "Summe privat gezahlt"}
    abziehbar_row = {"Kennzahl": "Abziehbar (Netto)"}
    vorsteuer_row = {"Kennzahl": "Gezahlte Vorsteuer"}
    abziehbare_vorsteuer_row = {"Kennzahl": "Abziehbare Vorsteuer"}
    summe_abziehbar_row = {"Kennzahl": "Summe abziehbar"}

    for lbl in labels:
        summe_row[lbl] = fmt(summe_totals[lbl])
        abziehbar_row[lbl] = fmt(abziehbar_totals[lbl])
        vorsteuer_row[lbl] = fmt(ZERO)
        abziehbare_vorsteuer_row[lbl] = fmt(ZERO)
        summe_abziehbar_row[lbl] = fmt(abziehbar_totals[lbl])

    rows: list[dict[str, str]] = []
    if tax_rows:
        rows.extend(tax_rows)
        rows.append(blank_row(labels))
    rows.extend(section_rows)
    rows.extend([
        blank_row(labels),
        summe_row,
        abziehbar_row,
        vorsteuer_row,
        abziehbare_vorsteuer_row,
        summe_abziehbar_row,
        blank_row(labels),
    ])
    return rows
