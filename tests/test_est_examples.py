from __future__ import annotations

from decimal import Decimal

from calculate import aggregates
from domain.dataset import TaxDataset


def _gross_for_label(dataset: TaxDataset, year: int, label: str) -> Decimal:
    labeled = TaxDataset(
        [p for p in dataset if p.year == year and p.tax_form == "einkommensteuer" and p.label == label]
    )
    return aggregates.gross_amount(labeled)


def test_est_2024_private_insurance_and_advance(dataset: TaxDataset) -> None:
    year = 2024
    assert _gross_for_label(dataset, year, "Krankenversicherung") == Decimal("840.00")
    assert _gross_for_label(dataset, year, "Pflegeversicherung") == Decimal("240.00")
    assert _gross_for_label(dataset, year, "Zusatzbeitrag") == Decimal("120.00")
    assert _gross_for_label(dataset, year, "Langzeit-Auslandskrankenversicherung") == Decimal("343.50")
    assert _gross_for_label(dataset, year, "Kurzzeit-Auslandskrankenversicherung") == Decimal("9.50")
    assert _gross_for_label(dataset, year, "Haftpflichtversicherung") == Decimal("57.88")
    assert aggregates.gross_amount(dataset.for_role("income_tax_advance").for_year(year)) == Decimal("400.00")


def test_est_2025_final_payment_and_health_insurance(dataset: TaxDataset) -> None:
    year = 2025
    assert _gross_for_label(dataset, year, "Krankenversicherung") == Decimal("910.00")
    assert _gross_for_label(dataset, year, "Pflegeversicherung") == Decimal("260.00")
    assert _gross_for_label(dataset, year, "Zusatzbeitrag") == Decimal("130.00")
    assert aggregates.gross_amount(dataset.for_role("income_tax_final").for_year(year)) == Decimal("50.00")
