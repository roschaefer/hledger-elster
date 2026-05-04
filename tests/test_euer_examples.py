from __future__ import annotations

from decimal import Decimal

from calculate import afa, aggregates
from domain.dataset import TaxDataset


def _postings_for_label(dataset: TaxDataset, label: str) -> TaxDataset:
    return TaxDataset([p for p in dataset if p.label == label])


def test_euer_2024_income_and_expenses(dataset: TaxDataset) -> None:
    year = 2024
    income_ds = TaxDataset(
        [p for p in dataset.for_year(year) if p.tax_form == "einnahmenueberschussrechnung" and p.amount < Decimal("0")]
    )
    assert aggregates.gross_amount(income_ds) == Decimal("1190.00")
    assert aggregates.net_amount(income_ds) == Decimal("1000.00")
    assert aggregates.collected_vat(income_ds) == Decimal("190.00")
    assert aggregates.deductible_net(_postings_for_label(dataset, "Serverkosten Wasabi").for_year(year)) == Decimal(
        "20.00"
    )
    assert aggregates.deductible_net(_postings_for_label(dataset, "Mobiltelefon").for_year(year)) == Decimal("2.00")
    assert aggregates.deductible_net(_postings_for_label(dataset, "Steuerberatung").for_year(year)) == Decimal("100.00")


def test_euer_afa_spans_multiple_years(dataset: TaxDataset) -> None:
    afa_postings = list(_postings_for_label(dataset.for_deduction("afa"), "Computer-Kauf"))
    assert len(afa_postings) == 1
    assert afa.depreciation_for_year(afa_postings[0], 2024) == Decimal("222.22")
    assert afa.depreciation_for_year(afa_postings[0], 2025) == Decimal("333.33")
