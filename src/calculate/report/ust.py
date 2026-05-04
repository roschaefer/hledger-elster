from __future__ import annotations

from decimal import Decimal, ROUND_HALF_UP

from domain.dataset import TaxDataset
from calculate import aggregates
from calculate.report.classification import euer_expenses, euer_income
from calculate.report.periods import filter_period, fmt


TWOPLACES = Decimal("0.01")
ZERO = Decimal("0.00")


def _vat_advance_year(dataset: TaxDataset, year: int) -> TaxDataset:
    invalid = [
        p for p in dataset.for_role("vat_advance")
        if p.amount != ZERO and p.tax_period_year == 0
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
    return TaxDataset([
        p for p in dataset.for_role("vat_advance")
        if p.amount != ZERO and p.tax_period_year == year
    ])


def ust_rows(dataset: TaxDataset, year: int) -> list[dict[str, str]]:
    income_ds = euer_income(dataset)
    euer_ds = euer_expenses(dataset)
    vat_advance_year = _vat_advance_year(dataset, year)

    col_vorauszahlungssoll = "Bereits Entrichtet"

    def make_row(lbl: str, for_vorauszahlung: TaxDataset | None) -> dict[str, str]:
        income_p = filter_period(income_ds, year, lbl)
        euer_p = filter_period(euer_ds, year, lbl)
        net = aggregates.net_amount(income_p)
        collected = aggregates.collected_vat(income_p)
        vorsteuer = aggregates.deductible_vat(euer_p)
        uberschuss = (collected - vorsteuer).quantize(TWOPLACES, rounding=ROUND_HALF_UP)
        return {
            "Zeitraum": lbl,
            "Einnahme (Netto)": fmt(net),
            "Vereinnahmte Umsatzsteuer": fmt(collected),
            "Abziehbare Vorsteuerbeträge": fmt(vorsteuer),
            "Vorauszahlungssoll": fmt(uberschuss),
            col_vorauszahlungssoll: (
                fmt(aggregates.signed_total(for_vorauszahlung))
                if for_vorauszahlung is not None
                else ""
            ),
        }

    rows: list[dict[str, str]] = []

    for m in range(1, 13):
        lbl = f"{year}-{m:02d}"
        rows.append(make_row(lbl, None))

    for q in range(1, 5):
        lbl = f"{year} Q{q}"
        rows.append(make_row(lbl, None))

    rows.append(make_row(str(year), vat_advance_year))

    return rows
